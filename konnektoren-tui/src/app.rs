use crate::{
    challenge_tabs::ChallengeTabs,
    challenge_widget::ChallengeWidget,
    error::{Error, Result},
    map_widget::MapWidget,
};

#[cfg(feature = "crossterm")]
use crate::tui::Tui;

#[cfg(feature = "crossterm")]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use konnektoren_core::{
    commands::{ChallengeCommand, Command, CommandTrait, GameCommand},
    session::Session,
};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct App {
    title: String,
    username: Option<String>,
    session: Session,
    show_map: bool,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            title: " Konnektoren ".into(),
            username: None,
            ..Self::default()
        }
    }

    pub fn with_session(session: Session) -> Self {
        App {
            session,
            ..Self::new()
        }
    }

    pub fn set_username(&mut self, username: String) {
        self.username = Some(username);
    }

    #[cfg(feature = "crossterm")]
    pub fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        terminal.clear().map_err(Error::Io)?;
        terminal.hide_cursor().map_err(Error::Io)?;

        while !self.exit {
            terminal
                .draw(|frame| self.render_frame(frame))
                .map_err(Error::Io)?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn next_question(&mut self) {
        let command = Command::Challenge(ChallengeCommand::NextTask);
        if let Err(err) = command.execute(&mut self.session.game_state) {
            tracing::error!("Failed to execute next question command: {}", err);
        }
    }

    pub fn previous_question(&mut self) {
        let command = Command::Challenge(ChallengeCommand::PreviousTask);
        if let Err(err) = command.execute(&mut self.session.game_state) {
            tracing::error!("Failed to execute previous question command: {}", err);
        }
    }

    pub fn next_challenge(&mut self) {
        let command = Command::Game(GameCommand::NextChallenge);
        if let Err(err) = command.execute(&mut self.session.game_state) {
            tracing::error!("Failed to execute next challenge command: {}", err);
        }
    }

    pub fn previous_challenge(&mut self) {
        let command = Command::Game(GameCommand::PreviousChallenge);
        if let Err(err) = command.execute(&mut self.session.game_state) {
            tracing::error!("Failed to execute previous challenge command: {}", err);
        }
    }

    pub fn solve_option(&mut self, option_id: usize) -> Result<()> {
        let command = Command::Challenge(ChallengeCommand::SolveOption(option_id));
        command
            .execute(&mut self.session.game_state)
            .map_err(Error::Command)
    }

    pub fn toggle_map(&mut self) {
        self.show_map = !self.show_map;
    }

    #[cfg(feature = "crossterm")]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Left | KeyCode::Char('h') => self.previous_question(),
            KeyCode::Right | KeyCode::Char('l') => self.next_question(),
            KeyCode::Tab => self.next_challenge(),
            KeyCode::BackTab => self.previous_challenge(),
            KeyCode::Char('0') => self.solve_option(0)?,
            KeyCode::Char('1') => self.solve_option(1)?,
            KeyCode::Char('2') => self.solve_option(2)?,
            KeyCode::Char('3') => self.solve_option(3)?,
            KeyCode::Char('4') => self.solve_option(4)?,
            KeyCode::Char('5') => self.solve_option(5)?,
            KeyCode::Char('6') => self.solve_option(6)?,
            KeyCode::Char('7') => self.solve_option(7)?,
            KeyCode::Char('8') => self.solve_option(8)?,
            KeyCode::Char('9') => self.solve_option(9)?,
            KeyCode::Char('m') => self.toggle_map(),
            _ => {}
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        #[cfg(feature = "crossterm")]
        match event::read().map_err(Error::Io)? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if let Err(e) = self.handle_key_event(key_event) {
                    tracing::error!("Error handling key event: {}", e);
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let username_display = self
            .username
            .as_ref()
            .map(|u| format!(" User: {} ", u))
            .unwrap_or_else(|| self.title.clone());

        let instructions = Line::from(vec![
            " Previous ".into(),
            "<Left>".blue().bold(),
            " Next ".into(),
            "<Right>".blue().bold(),
            " Map ".into(),
            "<M>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::default()
            .title(Line::from(username_display).bold().centered())
            .title_bottom(instructions.centered())
            .borders(Borders::ALL)
            .border_set(border::THICK);

        Paragraph::new(":")
            .centered()
            .block(block)
            .render(area, buf);

        let inner_area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        if self.show_map {
            let map = MapWidget::new(
                &self.session.game_state.game.game_paths[0],
                self.session.game_state.current_challenge_index,
            );
            map.render(inner_area, buf);
        } else {
            let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
            let [tab_area, challenge_area] = vertical.areas(inner_area);

            let tabs = ChallengeTabs::new(
                &self.session.game_state.game.game_paths[0],
                self.session.game_state.current_challenge_index,
            );
            tabs.render(tab_area, buf);

            let challenge_widget = ChallengeWidget {
                challenge: &self.session.game_state.challenge,
                show_help: true,
                current_question: self.session.game_state.current_task_index,
            };
            challenge_widget.render(challenge_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "crossterm")]
    fn handle_key_event() -> Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into())?;
        assert!(app.exit);

        Ok(())
    }
}
