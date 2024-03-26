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
    fn menu(&self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let mut should_quit = false;

        // any data fetched within this loop must be cached -- time to use HashMap
        while !should_quit {
            terminal.draw(|frame| Self::render(frame, self))?;
            should_quit = handle_events()?;
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    /// Responsible for rendering a single 'frame' in `menu`. Implementation
    /// will vary depending on the data structure, and the intended widget to be
    /// rendered.
    fn render(
        frame: &mut Frame,
        content: &Self,
    );
}

impl Menu for Release {
    fn render(
        frame: &mut Frame,
        rel: &Self,
    ) {
        // https://docs.rs/ratatui/0.26.1/ratatui/widgets/trait.Widget.html

        // para surrounded by block -- not very stateful, and not a good representation
        // of the data
        // let para = Paragraph::new(rel.display_tracklist());

        let items = rel.parse_tracklist().into_iter().map(|t| t.to_string());
        let list = List::new(items);

        // TODO: https://docs.rs/ratatui/0.26.1/ratatui/widgets/struct.ListState.html

        // not essential
        let block = Block::default()
            .title(rel.to_string())
            .borders(Borders::TOP);

        frame.render_widget(list.block(block), frame.size());
    }
}

impl Menu for DirEntry {
    fn render(
        frame: &mut Frame,
        dir: &Self,
    ) {
        // https://docs.rs/ratatui/0.26.1/ratatui/widgets/trait.Widget.html

        let entries: Vec<_> = dir
            .walk()
            .map(|d| d.path().to_str().unwrap().to_string())
            .collect();
        let list = List::new(entries);

        let block = Block::default().borders(Borders::TOP);

        frame.render_widget(list.block(block), frame.size());
    }
}
