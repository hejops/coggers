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

impl Release {
    pub fn menu(&self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let mut should_quit = false;

        // any data fetched within this loop must be cached -- time to use HashMap
        while !should_quit {
            terminal.draw(|frame| Self::ui(frame, self))?;
            should_quit = handle_events()?;
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    fn ui(
        frame: &mut Frame,
        rel: &Self,
    ) {
        // para surrounded by block
        // https://docs.rs/ratatui/0.26.1/ratatui/widgets/trait.Widget.html
        // let para = Paragraph::new(rel.display_tracklist());

        let items = rel.parse_tracklist().into_iter().map(|t| t.to_string());
        let list = List::new(items);

        let block = Block::default()
            .title(rel.to_string())
            .borders(Borders::TOP);

        frame.render_widget(list.block(block), frame.size());
    }
}
