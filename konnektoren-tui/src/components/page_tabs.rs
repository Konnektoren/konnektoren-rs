use konnektoren_platform::i18n::{I18nConfig, Language};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Tabs, Widget},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageTab {
    Challenge,
    Challenges,
    Map,
    Info,
}

impl PageTab {
    pub const fn index(self) -> usize {
        match self {
            Self::Challenge => 0,
            Self::Challenges => 1,
            Self::Map => 2,
            Self::Info => 3,
        }
    }
}

pub struct PageTabs {
    selected: PageTab,
    labels: [String; 4],
}

impl PageTabs {
    pub fn new(selected: PageTab, i18n: &I18nConfig, language: Option<&Language>) -> Self {
        Self {
            selected,
            labels: [
                t(i18n, language, "Challenge"),
                t(i18n, language, "Challenges"),
                t(i18n, language, "Map"),
                t(i18n, language, "Info"),
            ],
        }
    }
}

impl Widget for PageTabs {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Tabs::new(self.labels)
            .highlight_style(Style::new().bg(Color::Blue).add_modifier(Modifier::BOLD))
            .select(self.selected.index())
            .padding(" ", " ")
            .divider(" | ")
            .render(area, buf);
    }
}

fn t(i18n: &I18nConfig, language: Option<&Language>, key: &str) -> String {
    language.map_or_else(|| i18n.t(key), |language| i18n.t_with_lang(key, language))
}
