use konnektoren_core::challenges::{Challenge, ChallengeType};
use konnektoren_platform::i18n::{I18nConfig, Language};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

pub struct OptionsWidget<'a> {
    pub challenge_type: ChallengeType,
    i18n: &'a I18nConfig,
    language: Option<&'a Language>,
}

impl<'a> OptionsWidget<'a> {
    pub fn new(
        challenge: &Challenge,
        i18n: &'a I18nConfig,
        language: Option<&'a Language>,
    ) -> Self {
        OptionsWidget {
            challenge_type: challenge.challenge_type.clone(),
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

impl Widget for OptionsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.challenge_type {
            ChallengeType::MultipleChoice(ref dataset) => {
                let block = Block::bordered()
                    .title(format!(" {} ", self.t("Options")).bold())
                    .border_set(border::ROUNDED);

                let options = dataset
                    .options
                    .iter()
                    .map(|option| Line::from(format!("<{}> {}", option.id, option.name)));

                Paragraph::new(Text::from(options.collect::<Vec<Line>>()))
                    .block(block)
                    .render(area, buf);
            }
            _ => {
                Paragraph::new(self.t("No options"))
                    .block(Block::bordered().title(format!(" {} ", self.t("Options"))))
                    .render(area, buf);
            }
        }
    }
}
