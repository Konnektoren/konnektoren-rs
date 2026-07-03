use crate::{
    components::{
        ChallengeDebugInfo, ChallengeList, ChallengeTabs, ChallengeWidget, MapWidget, PageTab,
        PageTabs,
    },
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
use konnektoren_platform::i18n::{I18nAssets, I18nConfig, JsonTranslationAsset, Language};
#[cfg(feature = "crossterm")]
use ratatui::Frame;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

/// A backend-agnostic key, decoupled from any specific terminal or input crate.
///
/// Native (crossterm) input is mapped to this type internally; other frontends
/// (e.g. a web-based terminal) can map their own key events to it and drive
/// [`App`] via [`App::handle_key`] without this crate depending on their input crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Tab,
    BackTab,
    Enter,
    Esc,
    PageUp,
    PageDown,
    Home,
}

#[cfg(feature = "crossterm")]
impl Key {
    fn from_crossterm(code: KeyCode) -> Option<Self> {
        Some(match code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::BackTab,
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Home => Key::Home,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum AppPage {
    #[default]
    Challenge,
    Challenges,
    Map,
    Info,
}

impl AppPage {
    const fn tab(self) -> PageTab {
        match self {
            Self::Challenge => PageTab::Challenge,
            Self::Challenges => PageTab::Challenges,
            Self::Map => PageTab::Map,
            Self::Info => PageTab::Info,
        }
    }
}

#[derive(Debug, Default)]
pub struct App {
    title: String,
    username: Option<String>,
    language: Option<String>,
    i18n: I18nConfig,
    session: Session,
    page: AppPage,
    selected_challenge_index: usize,
    info_scroll: u16,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            title: " Konnektoren ".into(),
            username: None,
            language: None,
            i18n: I18nConfig::with_assets(JsonTranslationAsset::<I18nAssets>::new()),
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

    pub fn set_language(&mut self, language: impl Into<String>) {
        self.language = Some(language.into());
    }

    fn selected_language(&self) -> Option<Language> {
        self.language
            .as_deref()
            .and_then(|language| Language::try_from_code(language).ok())
    }

    fn t(&self, key: &str) -> String {
        self.selected_language().as_ref().map_or_else(
            || self.i18n.t(key),
            |language| self.i18n.t_with_lang(key, language),
        )
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

    #[cfg(feature = "crossterm")]
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

    pub fn show_challenge_info(&mut self) {
        if self.page != AppPage::Challenges {
            self.selected_challenge_index = self.session.game_state.current_challenge_index;
        }
        self.info_scroll = 0;
        self.page = AppPage::Info;
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

    pub fn scroll_info_down(&mut self, lines: u16) {
        self.info_scroll = self.info_scroll.saturating_add(lines);
    }

    pub fn scroll_info_up(&mut self, lines: u16) {
        self.info_scroll = self.info_scroll.saturating_sub(lines);
    }

    pub fn reset_info_scroll(&mut self) {
        self.info_scroll = 0;
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

    /// Handle a backend-agnostic key. See [`Key`].
    pub fn handle_key(&mut self, key: Key) -> Result<()> {
        match key {
            Key::Char('q') | Key::Esc => self.exit(),
            Key::Char('c') => self.show_challenge_page(),
            Key::Char('g') => self.show_challenge_list(),
            Key::Char('i') => self.show_challenge_info(),
            Key::Char('m') => self.toggle_map(),
            Key::Up | Key::Char('k') if self.page == AppPage::Info => {
                self.scroll_info_up(1);
            }
            Key::Down | Key::Char('j') if self.page == AppPage::Info => {
                self.scroll_info_down(1);
            }
            Key::PageUp if self.page == AppPage::Info => {
                self.scroll_info_up(10);
            }
            Key::PageDown if self.page == AppPage::Info => {
                self.scroll_info_down(10);
            }
            Key::Home if self.page == AppPage::Info => {
                self.reset_info_scroll();
            }
            Key::Up | Key::Char('k') if self.page == AppPage::Challenges => {
                self.select_previous_challenge_in_list();
            }
            Key::Down | Key::Char('j') if self.page == AppPage::Challenges => {
                self.select_next_challenge_in_list();
            }
            Key::Enter if self.page == AppPage::Challenges => self.open_selected_challenge()?,
            Key::Left | Key::Char('h') => self.previous_question(),
            Key::Right | Key::Char('l') => self.next_question(),
            Key::Tab => self.next_challenge(),
            Key::BackTab => self.previous_challenge(),
            Key::Char('0') => self.solve_option(0)?,
            Key::Char('1') => self.solve_option(1)?,
            Key::Char('2') => self.solve_option(2)?,
            Key::Char('3') => self.solve_option(3)?,
            Key::Char('4') => self.solve_option(4)?,
            Key::Char('5') => self.solve_option(5)?,
            Key::Char('6') => self.solve_option(6)?,
            Key::Char('7') => self.solve_option(7)?,
            Key::Char('8') => self.solve_option(8)?,
            Key::Char('9') => self.solve_option(9)?,
            _ => {}
        }
        Ok(())
    }

    #[cfg(feature = "crossterm")]
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let Some(key) = Key::from_crossterm(key_event.code) else {
            return Ok(());
        };
        self.handle_key(key)
    }

    #[cfg(feature = "crossterm")]
    fn handle_events(&mut self) -> Result<()> {
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
            .map(|u| format!(" {}: {} ", self.t("User"), u))
            .unwrap_or_else(|| self.title.clone());
        let header = if let Some(language) = &self.language {
            format!("{} {}: {} ", username_display, self.t("Language"), language)
        } else {
            username_display
        };

        let instructions = Line::from(vec![
            format!(" {} ", self.t("Previous")).into(),
            "<Left>".blue().bold(),
            format!(" {} ", self.t("Next")).into(),
            "<Right>".blue().bold(),
            format!(" {} ", self.t("Map")).into(),
            "<M>".blue().bold(),
            format!(" {} ", self.t("List")).into(),
            "<G>".blue().bold(),
            format!(" {} ", self.t("Play")).into(),
            "<C>".blue().bold(),
            format!(" {} ", self.t("Info")).into(),
            "<I>".blue().bold(),
            format!(" {} ", self.t("Quit")).into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::default()
            .title(Line::from(header).bold().centered())
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
        PageTabs::new(
            self.page.tab(),
            &self.i18n,
            self.selected_language().as_ref(),
        )
        .render(page_tabs_area, buf);

        let Some(game_path) = self.current_game_path() else {
            Paragraph::new(self.t("No game path")).render(content_area, buf);
            return;
        };

        match self.page {
            AppPage::Map => {
                MapWidget::new(
                    game_path,
                    self.session.game_state.current_challenge_index,
                    &self.i18n,
                    self.selected_language().as_ref(),
                )
                .render(content_area, buf);
            }
            AppPage::Challenges => {
                let mut state = ratatui::widgets::ListState::default();
                ChallengeList::new(
                    game_path,
                    self.session.game_state.current_challenge_index,
                    self.selected_challenge_index,
                    &self.i18n,
                    self.selected_language().as_ref(),
                )
                .render(content_area, buf, &mut state);
            }
            AppPage::Info => {
                if let Some(config) = game_path.challenges.get(self.selected_challenge_index) {
                    let created = self.session.game_state.game.create_challenge(&config.id);
                    let active = (self.selected_challenge_index
                        == self.session.game_state.current_challenge_index)
                        .then_some(&self.session.game_state.challenge);
                    ChallengeDebugInfo::new(
                        config,
                        self.selected_challenge_index,
                        created.as_ref(),
                        active,
                        self.info_scroll,
                        &self.i18n,
                        self.selected_language().as_ref(),
                    )
                    .render(content_area, buf);
                } else {
                    Paragraph::new(self.t("No challenge selected")).render(content_area, buf);
                }
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
                    i18n: &self.i18n,
                    language: self.selected_language().as_ref(),
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

    #[test]
    fn info_page_uses_current_challenge_outside_list() {
        let mut app = App::new();
        app.selected_challenge_index = 3;

        app.show_challenge_info();

        assert_eq!(app.page, AppPage::Info);
        assert_eq!(
            app.selected_challenge_index,
            app.session.game_state.current_challenge_index
        );
    }

    #[test]
    fn info_page_preserves_list_selection() {
        let mut app = App::new();
        app.show_challenge_list();
        app.select_next_challenge_in_list();

        app.show_challenge_info();

        assert_eq!(app.page, AppPage::Info);
        assert_eq!(app.selected_challenge_index, 1);
    }

    #[test]
    fn info_scroll_is_bounded_at_zero() {
        let mut app = App::new();

        app.scroll_info_down(3);
        assert_eq!(app.info_scroll, 3);
        app.scroll_info_up(10);
        assert_eq!(app.info_scroll, 0);
    }
}
