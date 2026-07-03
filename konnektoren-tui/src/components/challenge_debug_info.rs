use konnektoren_core::{
    challenges::{Challenge, ChallengeConfig, Performance, Timed},
    game::GameError,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Wrap},
};

pub struct ChallengeDebugInfo<'a> {
    config: &'a ChallengeConfig,
    selected_index: usize,
    created: std::result::Result<&'a Challenge, &'a GameError>,
    active: Option<&'a Challenge>,
    scroll: u16,
}

impl<'a> ChallengeDebugInfo<'a> {
    pub const fn new(
        config: &'a ChallengeConfig,
        selected_index: usize,
        created: std::result::Result<&'a Challenge, &'a GameError>,
        active: Option<&'a Challenge>,
        scroll: u16,
    ) -> Self {
        Self {
            config,
            selected_index,
            created,
            active,
            scroll,
        }
    }
}

impl Widget for ChallengeDebugInfo<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.lines())
            .block(Block::bordered().title(" Challenge Debug Info "))
            .scroll((self.scroll, 0))
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl ChallengeDebugInfo<'_> {
    fn lines(&self) -> Text<'static> {
        let mut lines = vec![
            Line::from("Config".bold()),
            Line::from(format!("index: {}", self.selected_index)),
            Line::from(format!("id: {}", self.config.id)),
            Line::from(format!("name: {}", self.config.name)),
            Line::from(format!("description: {}", self.config.description)),
            Line::from(format!("challenge asset id: {}", self.config.challenge)),
            Line::from(format!(
                "variant: {}",
                self.config
                    .variant
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "-".to_string())
            )),
            Line::from(format!("tasks: {}", self.config.tasks)),
            Line::from(format!("unlock_points: {}", self.config.unlock_points)),
            Line::from(format!("position: {:?}", self.config.position)),
            Line::from(format!(
                "icon: {}",
                self.config.icon.as_deref().unwrap_or("-")
            )),
            Line::from(""),
            Line::from("Asset validation".bold()),
        ];

        match self.created {
            Ok(challenge) => {
                lines.push(Line::from("status: ok".green()));
                lines.push(Line::from(format!(
                    "challenge type id: {}",
                    challenge.challenge_type.id()
                )));
                lines.push(Line::from(format!(
                    "challenge type name: {}",
                    challenge.challenge_type.name()
                )));
                push_yaml(&mut lines, "challenge type yaml", &challenge.challenge_type);
                push_yaml(
                    &mut lines,
                    "initial result yaml",
                    &challenge.challenge_result,
                );
            }
            Err(err) => {
                lines.push(Line::from("status: failed".red().bold()));
                lines.push(Line::from(format!("error: {err}")));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from("Active runtime".bold()));
        if let Some(challenge) = self.active {
            lines.push(Line::from(format!("active id: {}", challenge.get_id())));
            lines.push(Line::from(format!("solved: {}", challenge.solved())));
            lines.push(Line::from(format!(
                "performance: {}",
                challenge.performance(&challenge.challenge_result)
            )));
            lines.push(Line::from(format!(
                "start_time: {:?}",
                challenge.start_time()
            )));
            lines.push(Line::from(format!("end_time: {:?}", challenge.end_time())));
            lines.push(Line::from(format!(
                "elapsed_seconds: {}",
                challenge
                    .elapsed_time()
                    .map(|elapsed| elapsed.num_seconds().to_string())
                    .unwrap_or_else(|| "-".to_string())
            )));
            push_yaml(&mut lines, "config yaml", &challenge.challenge_config);
            push_yaml(&mut lines, "result yaml", &challenge.challenge_result);
        } else {
            lines.push(Line::from("not the active challenge"));
        }

        Text::from(lines)
    }
}

fn push_yaml<T>(lines: &mut Vec<Line<'static>>, title: &str, value: &T)
where
    T: serde::Serialize,
{
    lines.push(Line::from(""));
    lines.push(Line::from(title.to_string().bold()));
    for line in to_yaml(value).lines() {
        lines.push(Line::from(line.to_string()));
    }
}

fn to_yaml<T>(value: &T) -> String
where
    T: serde::Serialize,
{
    serde_yaml::to_string(value)
        .map(|yaml| yaml.trim().to_string())
        .unwrap_or_else(|err| format!("serialization failed: {err}"))
}
