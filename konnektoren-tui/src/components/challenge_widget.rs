use konnektoren_core::challenges::{Challenge, ChallengeType};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use super::{options_widget::OptionsWidget, results_widget::ResultsWidget};

pub struct ChallengeWidget<'a> {
    pub challenge: &'a Challenge,
    pub show_help: bool,
    pub current_question: usize,
}

impl Widget for ChallengeWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.challenge.challenge_type {
            ChallengeType::MultipleChoice(ref dataset) => {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

                let title = format!(
                    " Question ({}/{}) ",
                    self.current_question + 1,
                    self.challenge.challenge_config.tasks
                );

                let block = Block::bordered()
                    .title(title.bold())
                    .border_set(border::ROUNDED);

                let text = if let Some(question) = dataset.questions.get(self.current_question) {
                    let mut lines = vec![Line::from(question.question.as_str())];
                    if self.show_help {
                        lines.push(Line::from(question.help.as_str()).dim());
                    }
                    Text::from(lines)
                } else {
                    Text::from(Line::from("Question not found").red())
                };

                let panels = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(layout[1]);

                Paragraph::new(text)
                    .centered()
                    .block(block)
                    .render(layout[0], buf);

                OptionsWidget::new(self.challenge).render(panels[0], buf);
                ResultsWidget::new(self.challenge).render(panels[1], buf);
            }
            _ => {
                Paragraph::new("Unsupported challenge type")
                    .block(Block::bordered().title(" Challenge "))
                    .render(area, buf);
            }
        }
    }
}
