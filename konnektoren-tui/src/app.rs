use crate::{
    components::{ChallengeList, ChallengeTabs, ChallengeWidget, MapWidget, PageTab, PageTabs},
    error::{Error, Result},
};

#[cfg(feature = "crossterm")]
use crate::tui::Tui;

#[cfg(feature = "crossterm")]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use konnektoren_core::{
    challenges::Timed,
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
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum AppPage {
    #[default]
    Challenge,
    Challenges,
    Map,
}

impl AppPage {
    const fn tab(self) -> PageTab {
        match self {
            Self::Challenge => PageTab::Challenge,
            Self::Challenges => PageTab::Challenges,
            Self::Map => PageTab::Map,
        }
    }
}

#[derive(Debug, Default)]
pub struct App {
    title: String,
    username: Option<String>,
    session: Session,
    page: AppPage,
    selected_challenge_index: usize,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            title: " Konnektoren ".into(),
            username: None,
            page: AppPage::Challenge,
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
        } else {
            self.selected_challenge_index = self.session.game_state.current_challenge_index;
        }
    }

    pub fn previous_challenge(&mut self) {
        let command = Command::Game(GameCommand::PreviousChallenge);
        if let Err(err) = command.execute(&mut self.session.game_state) {
            tracing::error!("Failed to execute previous challenge command: {}", err);
        } else {
            self.selected_challenge_index = self.session.game_state.current_challenge_index;
        }
    }

    pub fn solve_option(&mut self, option_id: usize) -> Result<()> {
        let command = Command::Challenge(ChallengeCommand::SolveOption(option_id));
        command
            .execute(&mut self.session.game_state)
            .map_err(Error::Command)
    }

    pub fn toggle_map(&mut self) {
        self.page = if self.page == AppPage::Map {
            AppPage::Challenge
        } else {
            AppPage::Map
        };
    }

    pub fn show_challenge_page(&mut self) {
        self.page = AppPage::Challenge;
    }

    pub fn show_challenge_list(&mut self) {
        self.selected_challenge_index = self.session.game_state.current_challenge_index;
        self.page = AppPage::Challenges;
    }

    pub fn select_next_challenge_in_list(&mut self) {
        let challenge_count = self
            .current_game_path()
            .map_or(0, |path| path.challenges.len());
        if challenge_count > 0 {
            self.selected_challenge_index =
                (self.selected_challenge_index + 1).min(challenge_count - 1);
        }
    }

    pub fn select_previous_challenge_in_list(&mut self) {
        self.selected_challenge_index = self.selected_challenge_index.saturating_sub(1);
    }

    pub fn open_selected_challenge(&mut self) -> Result<()> {
        self.open_challenge(self.selected_challenge_index)?;
        self.page = AppPage::Challenge;
        Ok(())
    }

    pub fn open_challenge(&mut self, index: usize) -> Result<()> {
        let challenge_id = self
            .current_game_path()
            .and_then(|path| path.challenges.get(index))
            .map(|challenge| challenge.id.clone())
            .ok_or_else(|| Error::State(format!("challenge index not found: {index}")))?;

        let mut challenge = self
            .session
            .game_state
            .game
            .create_challenge(&challenge_id)
            .map_err(Error::Game)?;
        challenge.start();

        self.session.game_state.challenge = challenge;
        self.session.game_state.current_challenge_index = index;
        self.session.game_state.current_task_index = 0;
        self.selected_challenge_index = index;
        Ok(())
    }

    fn current_game_path(&self) -> Option<&konnektoren_core::game::GamePath> {
        self.session
            .game_state
            .game
            .game_paths
            .get(self.session.game_state.current_game_path)
    }

    #[cfg(feature = "crossterm")]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('c') => self.show_challenge_page(),
            KeyCode::Char('g') => self.show_challenge_list(),
            KeyCode::Char('m') => self.toggle_map(),
            KeyCode::Up | KeyCode::Char('k') if self.page == AppPage::Challenges => {
                self.select_previous_challenge_in_list();
            }
            KeyCode::Down | KeyCode::Char('j') if self.page == AppPage::Challenges => {
                self.select_next_challenge_in_list();
            }
            KeyCode::Enter if self.page == AppPage::Challenges => self.open_selected_challenge()?,
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
            " List ".into(),
            "<G>".blue().bold(),
            " Play ".into(),
            "<C>".blue().bold(),
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

        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
        let [page_tabs_area, content_area] = vertical.areas(inner_area);
        PageTabs::new(self.page.tab()).render(page_tabs_area, buf);

        let Some(game_path) = self.current_game_path() else {
            Paragraph::new("No game path").render(content_area, buf);
            return;
        };

        match self.page {
            AppPage::Map => {
                MapWidget::new(game_path, self.session.game_state.current_challenge_index)
                    .render(content_area, buf);
            }
            AppPage::Challenges => {
                let mut state = ratatui::widgets::ListState::default();
                ChallengeList::new(
                    game_path,
                    self.session.game_state.current_challenge_index,
                    self.selected_challenge_index,
                )
                .render(content_area, buf, &mut state);
            }
            AppPage::Challenge => {
                let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
                let [tab_area, challenge_area] = vertical.areas(content_area);
                ChallengeTabs::new(game_path, self.session.game_state.current_challenge_index)
                    .render(tab_area, buf);

                ChallengeWidget {
                    challenge: &self.session.game_state.challenge,
                    show_help: true,
                    current_question: self.session.game_state.current_task_index,
                }
                .render(challenge_area, buf);
            }
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

    #[test]
    fn open_challenge_selects_requested_index() -> Result<()> {
        let mut app = App::new();
        app.open_challenge(1)?;

        assert_eq!(app.session.game_state.current_challenge_index, 1);
        assert_eq!(app.session.game_state.current_task_index, 0);
        assert_eq!(app.selected_challenge_index, 1);
        Ok(())
    }

    #[test]
    fn list_selection_is_bounded() {
        let mut app = App::new();

        app.select_previous_challenge_in_list();
        assert_eq!(app.selected_challenge_index, 0);

        for _ in 0..100 {
            app.select_next_challenge_in_list();
        }
        let last_index = app
            .current_game_path()
            .map(|path| path.challenges.len().saturating_sub(1))
            .unwrap_or_default();
        assert_eq!(app.selected_challenge_index, last_index);
    }
}
