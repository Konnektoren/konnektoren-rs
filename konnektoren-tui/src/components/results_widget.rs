use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use konnektoren_core::challenges::{Challenge, ChallengeResult, ChallengeType, Performance};

pub struct ResultsWidget<'a> {
    pub challenge: &'a Challenge,
}

impl<'a> ResultsWidget<'a> {
    pub fn new(challenge: &'a Challenge) -> Self {
        ResultsWidget { challenge }
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
                        let correct = if question.option == option.id {
                            "Correct".green().bold()
                        } else {
                            "Incorrect".red().bold()
                        };
                        text.push_line(Line::from(vec![
                            format!(" {}: {} ", question.question, option.name).into(),
                            correct,
                        ]));
                        text
                    },
                )
            }
            _ => Text::from("No results"),
        };

        let text = text.into_iter().rev().collect::<Vec<Line>>();
        Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(" Results ".bold())
                    .border_set(border::ROUNDED),
            )
            .render(layout[1], buf);

        let performance = self.challenge.performance(&self.challenge.challenge_result);
        Paragraph::new(Text::from(vec![Line::from(format!(
            "Performance: {}",
            performance
        ))]))
        .block(
            Block::bordered()
                .title(" Performance ".bold())
                .border_set(border::ROUNDED),
        )
        .render(layout[0], buf);
    }
}
