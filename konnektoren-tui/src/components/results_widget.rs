use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use konnektoren_core::challenges::{Challenge, ChallengeResult, ChallengeType, Performance};
use konnektoren_platform::i18n::{I18nConfig, Language};

pub struct ResultsWidget<'a> {
    pub challenge: &'a Challenge,
    i18n: &'a I18nConfig,
    language: Option<&'a Language>,
}

impl<'a> ResultsWidget<'a> {
    pub fn new(
        challenge: &'a Challenge,
        i18n: &'a I18nConfig,
        language: Option<&'a Language>,
    ) -> Self {
        ResultsWidget {
            challenge,
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

impl Widget for ResultsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        let text: Text = match (
            &self.challenge.challenge_type,
            &self.challenge.challenge_result,
        ) {
            (ChallengeType::MultipleChoice(dataset), ChallengeResult::MultipleChoice(options)) => {
                dataset.questions.iter().zip(options.iter()).fold(
                    Text::default(),
                    |mut text, (question, option)| {
                        let label = if question.option == option.id {
                            self.t("Correct")
                        } else {
                            self.t("Incorrect")
                        };
                        let correct = if question.option == option.id {
                            label.green().bold()
                        } else {
                            label.red().bold()
                        };
                        text.push_line(Line::from(vec![
                            format!(" {}: {} ", question.question, option.name).into(),
                            correct,
                        ]));
                        text
                    },
                )
            }
            _ => Text::from(self.t("No results")),
        };

        let text = text.into_iter().rev().collect::<Vec<Line>>();
        Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(format!(" {} ", self.t("Results")).bold())
                    .border_set(border::ROUNDED),
            )
            .render(layout[1], buf);

        let performance = self.challenge.performance(&self.challenge.challenge_result);
        Paragraph::new(Text::from(vec![Line::from(format!(
            "{}: {}",
            self.t("Performance"),
            performance
        ))]))
        .block(
            Block::bordered()
                .title(format!(" {} ", self.t("Performance")).bold())
                .border_set(border::ROUNDED),
        )
        .render(layout[0], buf);
    }
}
