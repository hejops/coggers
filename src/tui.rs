use std::io;
use std::io::stdout;

use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::*;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::io::Walk;
use crate::release::Release;

// https://github.com/ratatui-org/ratatui/tree/main?tab=readme-ov-file#example

fn handle_events() -> io::Result<bool> {
    if !event::poll(std::time::Duration::from_millis(50))? {
        return Ok(false);
    };
    if let Event::Key(key) = event::read()? {
        if key.kind != event::KeyEventKind::Press {
            return Ok(false);
        }
        match key.code {
            // what if we allowed spawning a new .menu()?
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('x') => return Ok(true),
            _ => (),
        }
    }

    Ok(false)
}

pub trait Menu {
    /// Responsible for the `ratatui` loop, and controlled by an event handler.
    /// The event handler is currently implemented globally; this will
    /// probably become a trait impl.
    fn menu(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let mut should_quit = false;

        // any data fetched within this loop must be cached -- time to use HashMap
        while !should_quit {
            terminal.draw(|frame| Self::render(self, frame))?;
            // should_quit = handle_events()?;
            should_quit = self.get_new_state()?;
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    // default impl is somewhat pointless, as it can only quit. in order to do
    // anything stateful, we need to know the implementor's state, which we, as
    // the trait, cannot know anything about
    fn get_new_state(&mut self) -> io::Result<bool> {
        // no events detected, do nothing
        if !event::poll(std::time::Duration::from_millis(50))? {
            return Ok(false);
        };
        if let Event::Key(key) = event::read()? {
            if key.kind != event::KeyEventKind::Press {
                return Ok(false);
            }
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('x') => return Ok(true),
                _ => (),
            }
        }

        Ok(false)
    }

    /// Responsible for rendering a single 'frame' in `menu`. Implementation
    /// will vary depending on the data structure, and the intended widget to be
    /// rendered.
    fn render(
        &mut self,
        frame: &mut Frame,
    );
}

impl Menu for Release {
    fn render(
        &mut self,
        frame: &mut Frame,
    ) {
        // https://docs.rs/ratatui/0.26.1/ratatui/widgets/trait.Widget.html

        // para surrounded by block -- not very stateful, and not a good representation
        // of the data
        // let para = Paragraph::new(rel.display_tracklist());

        let items = self.parse_tracklist().into_iter().map(|t| t.to_string());
        let list = List::new(items);

        // TODO: https://docs.rs/ratatui/0.26.1/ratatui/widgets/struct.ListState.html

        // not essential
        let block = Block::default()
            .title(self.to_string())
            .borders(Borders::TOP);

        frame.render_widget(list.block(block), frame.size());
    }
}

impl Menu for DirEntry {
    fn render(
        &mut self,
        frame: &mut Frame,
    ) {
        // https://docs.rs/ratatui/0.26.1/ratatui/widgets/trait.Widget.html

        let entries: Vec<_> = self
            .walk()
            .map(|d| d.path().to_str().unwrap().to_string())
            .collect();
        let list = List::new(entries);

        let block = Block::default().borders(Borders::TOP);
        frame.render_widget(list.block(block), frame.size());

        // let mut state = ListState::default(); //.with_offset(1);
        // frame.render_stateful_widget(list.block(block), frame.size(), &mut
        // state);
    }
}

pub struct DirMenu {
    dirs: Vec<DirEntry>,
    state: ListState,
}

impl DirMenu {
    pub fn new(root: &str) -> Self {
        let dirs = WalkDir::new(root)
            .into_iter()
            .filter_map(|f| f.ok())
            .collect();
        let state = ListState::default(); //.with_offset(1);
        Self { dirs, state }
    }
}
impl Menu for DirMenu {
    fn render(
        &mut self,
        frame: &mut Frame,
    ) {
        let entries: Vec<_> = self
            .dirs
            .iter()
            .map(|d| d.path().to_str().unwrap().to_string())
            .collect();
        let list = List::new(entries);

        let block = Block::default().borders(Borders::TOP);

        frame.render_stateful_widget(list.block(block), frame.size(), &mut self.state);

        // https://docs.rs/ratatui/0.26.1/src/demo2/tabs/email.rs.html#97
    }

    fn get_new_state(&mut self) -> io::Result<bool> {
        // no events detected, do nothing
        if !event::poll(std::time::Duration::from_millis(50))? {
            return Ok(false);
        };
        if let Event::Key(key) = event::read()? {
            if key.kind != event::KeyEventKind::Press {
                return Ok(false);
            }
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('x') => return Ok(true),
                KeyCode::Char('j') => {
                    // TODO: overflow = panic
                    *self.state.offset_mut() += 1;
                    return Ok(false);
                }
                KeyCode::Char('k') => {
                    *self.state.offset_mut() -= 1;
                    return Ok(false);
                }
                _ => (),
            }
        }

        Ok(false)
    }
}
