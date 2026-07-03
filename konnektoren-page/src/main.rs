#[cfg(target_arch = "wasm32")]
mod app {
    use std::{cell::RefCell, io, rc::Rc};

    use ratzilla::{
        DomBackend, WebRenderer,
        event::KeyCode,
        ratatui::{
            Frame, Terminal,
            layout::{Alignment, Constraint, Layout, Margin, Rect},
            style::{Color, Modifier, Style, Stylize},
            text::{Line, Span, Text},
            widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
        },
        web_sys,
    };

    const LINKS: [Link; 3] = [
        Link {
            label: "GitHub",
            description: "Source, issues, releases, and workspace history",
            url: "https://github.com/konnektoren/konnektoren-rs",
        },
        Link {
            label: "Crate docs",
            description: "Generated Rust API documentation",
            url: "https://konnektoren.github.io/konnektoren-rs/doc/konnektoren_core/",
        },
        Link {
            label: "Project docs",
            description: "Architecture notes and rendered AsciiDoc documentation",
            url: "https://konnektoren.github.io/konnektoren-rs/docs/",
        },
    ];

    #[derive(Debug, Clone, Copy)]
    struct Link {
        label: &'static str,
        description: &'static str,
        url: &'static str,
    }

    #[derive(Debug, Default)]
    struct State {
        selected: usize,
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

    pub fn run() -> io::Result<()> {
        let state = Rc::new(RefCell::new(State::default()));
        let backend = DomBackend::new()?;
        let mut terminal = Terminal::new(backend)?;

        terminal.on_key_event({
            let state = Rc::clone(&state);
            move |key_event| {
                let mut state = state.borrow_mut();
                match key_event.code {
                    KeyCode::Up | KeyCode::Char('k') => state.previous(),
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => state.next(),
                    KeyCode::Enter => open_link(state.selected_link().url),
                    _ => {}
                }
            }
        })?;

        terminal.draw_web(move |frame| {
            let state = state.borrow();
            render(frame, &state);
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
        let text = Text::from(vec![
            Line::from(vec![
                Span::styled("selected: ", Style::new().fg(Color::Gray)),
                Span::styled(link.label, Style::new().fg(Color::LightGreen).bold()),
            ]),
            Line::from(link.description),
            Line::from(""),
            Line::from(Span::styled(link.url, Style::new().fg(Color::Cyan))),
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
