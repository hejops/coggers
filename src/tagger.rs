//! TUI implementation for tagging files in the source directory. Heavily
//! borrowed from the [ratatui list example](https://docs.rs/ratatui/latest/src/list/list.rs.html).

use std::io;
use std::io::stdout;

use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::*;
use walkdir::DirEntry;

use crate::io::Walk;

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
                        // Char('h') | Left => self.items.unselect(),
                        Char('j') | Down => self.next(),
                        Char('k') | Up => self.previous(),
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
    fn next(&mut self) {
        // an item will always be selected in this app
        let curr = self.dir_state.selected().unwrap();

        let new = curr
            .ge(&(self.items.len() - 1)) // last item, wrap-around
            .then_some(0)
            .or(Some(curr + 1));

        // let new = Some(if curr >= self.items.len() - 1 {
        //     0
        // } else {
        //     curr + 1
        // });

        self.dir_state.select(new);
    }

    fn previous(&mut self) {
        let curr = self.dir_state.selected().unwrap();

        let new = curr
            .eq(&0) // last item, wrap-around
            .then_some(self.items.len() - 1)
            .or(Some(curr - 1));

        self.dir_state.select(new);
    }

    // TODO: get tags -> search discogs
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
            Constraint::Min(0),
        ]);
        let [upper, _, lower] = hsplit.areas(area);

        let vsplit = Layout::horizontal(Constraint::from_percentages([49, 2, 49]));
        let [left, _, right] = vsplit.areas(lower);

        self.render_dirs(upper, buf);
        self.render_files(left, buf);
        self.render_discogs(right, buf);
    }
}

impl TaggerApp {
    pub fn render_dirs(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let items: Vec<ListItem> = self.items.iter().map(|f| f.as_list_item()).collect();
        StatefulWidget::render(
            List::new(items).highlight_symbol("> "),
            area,
            buf,
            &mut self.dir_state,
        );
    }

    pub fn render_files(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) {
        match self.dir_state.selected() {
            None => (),
            Some(idx) => {
                // this has to be String, since file entries are not persistently stored
                let mut files: Vec<String> = self
                    .items
                    .get(idx)
                    .unwrap()
                    .to_owned()
                    .walk()
                    // yeah...
                    .map(|f| f.path().file_name().unwrap().to_str().unwrap().to_owned())
                    .collect();
                files.sort();
                Widget::render(List::new(files).dim(), area, buf);
            }
        }
    }

    pub fn render_discogs(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) {
        match self.dir_state.selected() {
            None => (),
            Some(idx) => {
                let files: Vec<String> = self
                    .items
                    .get(idx)
                    .unwrap()
                    .walk()
                    .map(|f| f.as_str().to_string())
                    .collect();
                let list = List::new(files);
                Widget::render(list, area, buf);
            }
        }
    }
}
