use konnektoren_core::game::GamePath;
use konnektoren_platform::i18n::{I18nConfig, Language};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};

pub struct ChallengeList<'a> {
    game_path: &'a GamePath,
    current_index: usize,
    selected_index: usize,
    i18n: &'a I18nConfig,
    language: Option<&'a Language>,
}

impl<'a> ChallengeList<'a> {
    pub const fn new(
        game_path: &'a GamePath,
        current_index: usize,
        selected_index: usize,
        i18n: &'a I18nConfig,
        language: Option<&'a Language>,
    ) -> Self {
        Self {
            game_path,
            current_index,
            selected_index,
            i18n,
            language,
        }
    }

    fn t(&self, key: &str) -> String {
        self.language.map_or_else(
            || self.i18n.t(key),
            |language| self.i18n.t_with_lang(key, language),
        )
    }
}

impl StatefulWidget for ChallengeList<'_> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.select(Some(self.selected_index));

        let items = self
            .game_path
            .challenges
            .iter()
            .enumerate()
            .map(|(index, challenge)| {
                let marker = if index == self.current_index {
                    ">"
                } else {
                    " "
                };
                ListItem::new(Line::from(format!(
                    "{} {} - {}",
                    marker, challenge.id, challenge.name
                )))
            });

        List::new(items.collect::<Vec<_>>())
            .block(Block::bordered().title(format!(" {} ", self.t("Challenges"))))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ")
            .render(area, buf, state);
    }
}
