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
use walkdir::WalkDir;

use crate::io::Sort;
use crate::io::Walk;

pub struct TaggerApp {
    state: ListState,
    items: Vec<String>, // TODO: &str (lifetimes...)
    last_selected: Option<usize>,
}

impl TaggerApp {
    pub fn with_items(items: Vec<DirEntry>) -> Self {
        TaggerApp {
            state: ListState::default(), //.with_offset(3),
            items: items.iter().map(|f| f.as_str().to_string()).collect(),
            last_selected: None,
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

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
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
        let vertical = Layout::vertical([
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Min(0),
        ]);
        let [upper, _, lower] = vertical.areas(area);

        let foo = Layout::horizontal(Constraint::from_percentages([49, 2, 49]));
        let [left, _, right] = foo.areas(lower);

        self.render_dirs(upper, buf);
        self.render_files(left, buf);
        self.render_files(right, buf); // TODO: discogs
    }
}

impl Walk for &String {
    fn walk(&self) -> impl Iterator<Item = DirEntry> {
        // 1;
        WalkDir::new(self).into_iter().filter_map(|f| f.ok())
    }
}

impl TaggerApp {
    pub fn render_dirs(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) {
        // note: highlight only becomes visible when an item is selected
        // TODO: don't clone!
        let list = List::new(self.items.clone()).highlight_symbol("> ");
        StatefulWidget::render(list, area, buf, &mut self.state);
    }
    pub fn render_files(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
    ) {
        match self.state.selected() {
            None => (),
            Some(idx) => {
                let files: Vec<String> = self
                    .items
                    .get(idx)
                    .unwrap()
                    .walk()
                    .map(|f| f.as_str().to_string())
                    .collect();
                // TODO: sort files
                let list = List::new(files);
                Widget::render(list, area, buf);
            }
        }
    }
}
