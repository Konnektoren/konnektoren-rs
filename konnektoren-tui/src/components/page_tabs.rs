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
}

impl PageTab {
    pub const fn index(self) -> usize {
        match self {
            Self::Challenge => 0,
            Self::Challenges => 1,
            Self::Map => 2,
        }
    }
}

pub struct PageTabs {
    selected: PageTab,
}

impl PageTabs {
    pub const fn new(selected: PageTab) -> Self {
        Self { selected }
    }
}

impl Widget for PageTabs {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Tabs::new(["Challenge", "Challenges", "Map"])
            .highlight_style(Style::new().bg(Color::Blue).add_modifier(Modifier::BOLD))
            .select(self.selected.index())
            .padding(" ", " ")
            .divider(" | ")
            .render(area, buf);
    }
}
