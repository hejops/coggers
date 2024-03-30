//! TUI implementation for tagging files in the source directory. Heavily
//! borrowed from the [ratatui list example](https://docs.rs/ratatui/latest/src/list/list.rs.html).

use std::io;
use std::io::stdout;

use anyhow::Context;
use anyhow::Result;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::ExecutableCommand;
use id3::TagLike;
use itertools::Itertools;
use ratatui::prelude::*;
use ratatui::widgets::*;
use walkdir::DirEntry;

use crate::io::Walk;
use crate::io::SOURCE;
use crate::release::Release;
use crate::transcode::File;
use crate::transcode::SourceDir;
use crate::transcode::TagField;

pub struct TaggerApp {
    dir_state: ListState,
    items: Vec<DirEntry>,
}

impl TaggerApp {
    pub fn with_items(items: Vec<DirEntry>) -> Self {
        TaggerApp {
            dir_state: {
                // note: highlight only becomes visible when an item is selected
                let mut state = ListState::default();
                state.select(Some(0));
                state
            },
            items,
        }
    }

    /// `main` -> `run` -> loop{`draw` -> `render` -> `render` components...}
    pub fn main(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;

        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        self.run(terminal)?;

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    fn run(
        &mut self,
        mut terminal: Terminal<impl Backend>,
    ) -> io::Result<()> {
        loop {
            self.draw(&mut terminal)?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),

                        Char('J') | PageDown => self.next(5),
                        Char('K') | PageUp => self.previous(5),
                        Char('j') | Down => self.next(1),
                        Char('k') | Up => self.previous(1),

                        // Char('h') | Left => self.items.unselect(),
                        // Char('l') | Right | Enter => self.change_status(),
                        // Char('g') => self.go_top(),
                        // Char('G') => self.go_bottom(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(
        &mut self,
        terminal: &mut Terminal<impl Backend>,
    ) -> io::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    // state management

    /// Allows wrap-around
    fn next(
        &mut self,
        step: usize,
    ) {
        // an item will always be selected in this app
        let curr = self.dir_state.selected().unwrap();

        let new = curr
            .ge(&(self.items.len() - step)) // last item, wrap-around
            .then_some(0)
            .or(Some(curr + step));

        self.dir_state.select(new);
    }

    fn previous(
        &mut self,
        step: usize,
    ) {
        // let curr = self.dir_state.selected().unwrap();
        // let new = curr
        //     .eq(&0)
        //     .then_some(self.items.len() - 1) // first item, wrap-around
        //     .or(Some(curr - 1)); // curr-1 is an underflow, and i have no idea why

        let new = {
            let curr = self.dir_state.selected().unwrap();
            match curr {
                0 => self.items.len() - step,
                _ => curr - step,
            }
        };

        self.dir_state.select(Some(new));
    }
}

// rendering

impl Widget for &mut TaggerApp {
    fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
    ) where
        Self: Sized,
    {
        // Layout::vertical == horizontal split
        let hsplit = Layout::vertical([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Min(0),
        ]);
        let [upper, _, middle, _, lower] = hsplit.areas(area);

        self.render_dirs(upper, buf);
        self.render_summary(middle, buf);

        // let vsplit = Layout::horizontal(Constraint::from_percentages([32, 2, 32, 2,
        // 32])); let [left, _, middle, _, right] = vsplit.areas(lower);
        // self.render_files(left, buf);
        // self.render_tags(middle, buf);
        // self.render_discogs(right, buf);

        let vsplit = Layout::horizontal(Constraint::from_percentages([49, 2, 49]));
        let [left, _, right] = vsplit.areas(lower);
        self.render_tags(left, buf);
        self.render_discogs(right, buf);

        // TODO: footer with keybindings
    }
}

impl TaggerApp {
    pub fn render_dirs(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) {
        // TODO: basename
        let items: Vec<ListItem> = self.items.iter().map(|f| f.as_list_item()).collect();
        StatefulWidget::render(
            List::new(items).highlight_symbol("> "),
            area,
            buf,
            &mut self.dir_state,
        );
    }

    pub fn render_summary(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) -> Result<()> {
        let f = self.get_files().find(|f| f.file_type().is_file()).unwrap();
        let f = File::new(f.as_str())?;

        let summary = f.to_string();
        let list = List::new(summary.split('\n'))
            .block(Block::default().borders(Borders::ALL).title("summary"));
        Widget::render(list, area, buf);

        Ok(())
    }

    pub fn get_files(&self) -> impl Iterator<Item = DirEntry> + '_ {
        self.items
            .get(self.dir_state.selected().unwrap())
            .unwrap()
            .walk()
    }

    pub fn render_files(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) {
        // this has to be String, since file entries are not persistently stored by
        // TaggerApp
        let items: Vec<String> = self
            .get_files()
            // yeah...
            // TODO: add basename method in Walk
            .map(|f| f.path().file_name().unwrap().to_str().unwrap().to_owned())
            .sorted()
            .collect();
        Widget::render(List::new(items).dim(), area, buf);
    }

    pub fn render_tags(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let files = self
            .get_files()
            .filter(|f| f.file_type().is_file())
            .map(|f| f.as_str().to_string())
            .sorted()
            .map(|f| File::new(&f))
            .filter_map(|f| f.ok());
        let items = files
            .into_iter()
            .map(|f| f.tags.title().unwrap().to_string());
        Widget::render(
            List::new(items).block(Block::default().title("tags")),
            area,
            buf,
        );
    }

    pub fn render_discogs(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) -> Result<()> {
        let f = self.get_files().find(|f| f.file_type().is_file()).unwrap();
        let f = File::new(f.as_str())?;

        let results = Release::search(
            &f.get(TagField::Artist).unwrap(),
            &f.get(TagField::Album).unwrap(),
        )
        .results;

        // TODO: cache results into some hashmap
        // TODO: iterate through results (h/l); requires extra state in TaggerApp

        let block = Block::default().borders(Borders::LEFT);

        let list = match results.first() {
            Some(res) => {
                let rel = res.as_rel();
                let tracks = rel.tracklist();
                let items = tracks.iter().map(|t| t.to_string());
                // TODO: search url?
                List::new(items).block(block.title(rel.uri.clone()))
            }
            None => List::default().block(block.title("not found")),
        };
        Widget::render(list, area, buf);

        Ok(())
    }
}

pub fn main() {
    let dir = SourceDir::new(&SOURCE).unwrap();
    TaggerApp::with_items(dir.dirs()).main().unwrap();
}
