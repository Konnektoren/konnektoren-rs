#[cfg(target_arch = "wasm32")]
mod app {
    use std::{cell::RefCell, io, rc::Rc};

    use konnektoren_tui::prelude::{App, Key};
    use ratzilla::{
        DomBackend, WebRenderer,
        event::{KeyCode, KeyEvent},
        ratatui::{
            Frame, Terminal,
            layout::{Alignment, Constraint, Layout, Margin, Rect},
            style::{Color, Modifier, Style, Stylize},
            text::{Line, Span, Text},
            widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
        },
        web_sys,
    };

    const LINKS: [Link; 4] = [
        Link {
            label: "Play",
            description: "Play the Konnektoren challenges right here in the terminal",
            url: "",
            play: true,
        },
        Link {
            label: "GitHub",
            description: "Source, issues, releases, and workspace history",
            url: "https://github.com/konnektoren/konnektoren-rs",
            play: false,
        },
        Link {
            label: "Crate docs",
            description: "Generated Rust API documentation",
            url: "https://konnektoren.github.io/konnektoren-rs/doc/konnektoren_core/",
            play: false,
        },
        Link {
            label: "Project docs",
            description: "Architecture notes and rendered AsciiDoc documentation",
            url: "https://konnektoren.github.io/konnektoren-rs/docs/",
            play: false,
        },
    ];

    #[derive(Debug, Clone, Copy)]
    struct Link {
        label: &'static str,
        description: &'static str,
        url: &'static str,
        play: bool,
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    enum Mode {
        #[default]
        Menu,
        Play,
    }

    #[derive(Debug)]
    struct State {
        selected: usize,
        mode: Mode,
        app: App,
    }

    impl Default for State {
        fn default() -> Self {
            State {
                selected: 0,
                mode: Mode::default(),
                app: App::new(),
            }
        }
    }

    impl State {
        fn previous(&mut self) {
            self.selected = self.selected.saturating_sub(1);
        }

        fn next(&mut self) {
            self.selected = (self.selected + 1).min(LINKS.len() - 1);
        }

        fn selected_link(&self) -> Link {
            LINKS[self.selected]
        }
    }

    /// Maps a ratzilla key event to the backend-agnostic key konnektoren-tui expects.
    fn to_app_key(event: &KeyEvent) -> Option<Key> {
        Some(match event.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Tab if event.shift => Key::BackTab,
            KeyCode::Tab => Key::Tab,
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Home => Key::Home,
            _ => return None,
        })
    }

    pub fn run() -> io::Result<()> {
        let state = Rc::new(RefCell::new(State::default()));
        let backend = DomBackend::new()?;
        let mut terminal = Terminal::new(backend)?;

        terminal.on_key_event({
            let state = Rc::clone(&state);
            move |key_event| {
                let mut state = state.borrow_mut();
                match state.mode {
                    Mode::Menu => match key_event.code {
                        KeyCode::Up | KeyCode::Char('k') => state.previous(),
                        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => state.next(),
                        KeyCode::Enter => {
                            let link = state.selected_link();
                            if link.play {
                                state.mode = Mode::Play;
                            } else {
                                open_link(link.url);
                            }
                        }
                        _ => {}
                    },
                    Mode::Play => match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => state.mode = Mode::Menu,
                        _ => {
                            if let Some(key) = to_app_key(&key_event) {
                                let _ = state.app.handle_key(key);
                            }
                        }
                    },
                }
            }
        })?;

        terminal.draw_web(move |frame| {
            let state = state.borrow();
            match state.mode {
                Mode::Menu => render(frame, &state),
                Mode::Play => frame.render_widget(&state.app, frame.area()),
            }
        });

        Ok(())
    }

    fn render(frame: &mut Frame<'_>, state: &State) {
        let area = frame.area();
        frame.render_widget(Clear, area);

        let shell = centered_area(area);
        let block = Block::default()
            .title(Line::from(" Konnektoren RS ").bold().centered())
            .title_bottom(Line::from(" Up/Down select  Enter open ").centered())
            .borders(Borders::ALL)
            .border_style(Color::Green);
        frame.render_widget(block, shell);

        let inner = shell.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });
        let [hero_area, list_area, detail_area] = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Min(5),
        ])
        .areas(inner);

        render_hero(frame, hero_area);
        render_links(frame, list_area, state);
        render_detail(frame, detail_area, state.selected_link());
    }

    fn centered_area(area: Rect) -> Rect {
        let width = area.width.clamp(48, 96).min(area.width);
        let height = area.height.clamp(20, 34).min(area.height);
        let horizontal_margin = area.width.saturating_sub(width) / 2;
        let vertical_margin = area.height.saturating_sub(height) / 2;
        Rect {
            x: area.x + horizontal_margin,
            y: area.y + vertical_margin,
            width,
            height,
        }
    }

    fn render_hero(frame: &mut Frame<'_>, area: Rect) {
        let text = Text::from(vec![
            Line::from(vec![
                Span::styled("konnektoren-rs", Style::new().fg(Color::LightGreen).bold()),
                Span::raw(" workspace"),
            ]),
            Line::from(
                "Rust crates for Konnektoren game logic, assets, platform tools, and TUI debugging.",
            ),
            Line::from("A compact terminal index for the repository documentation."),
        ]);

        frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
    }

    fn render_links(frame: &mut Frame<'_>, area: Rect, state: &State) {
        let items = LINKS
            .iter()
            .enumerate()
            .map(|(index, link)| {
                let selected = index == state.selected;
                let marker = if selected { ">" } else { " " };
                let style = if selected {
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(Color::White)
                };

                ListItem::new(Line::from(vec![
                    Span::raw(format!("{marker} ")),
                    Span::styled(format!("{:<12}", link.label), style),
                    Span::raw(" "),
                    Span::styled(link.description, Style::new().fg(Color::Gray)),
                ]))
            })
            .collect::<Vec<_>>();

        frame.render_widget(
            List::new(items).block(
                Block::bordered()
                    .title(" Links ")
                    .border_style(Color::Yellow),
            ),
            area,
        );
    }

    fn render_detail(frame: &mut Frame<'_>, area: Rect, link: Link) {
        let action = if link.play {
            "press Enter to play".to_string()
        } else {
            link.url.to_string()
        };
        let text = Text::from(vec![
            Line::from(vec![
                Span::styled("selected: ", Style::new().fg(Color::Gray)),
                Span::styled(link.label, Style::new().fg(Color::LightGreen).bold()),
            ]),
            Line::from(link.description),
            Line::from(""),
            Line::from(Span::styled(action, Style::new().fg(Color::Cyan))),
        ]);

        frame.render_widget(
            Paragraph::new(text)
                .wrap(ratzilla::ratatui::widgets::Wrap { trim: false })
                .block(Block::bordered().title(" Open ").border_style(Color::Green)),
            area,
        );
    }

    fn open_link(url: &str) {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(url, "_blank");
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() -> std::io::Result<()> {
    app::run()
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!(
        "konnektoren-page is a WebAssembly app. Build it with trunk for wasm32-unknown-unknown."
    );
}
