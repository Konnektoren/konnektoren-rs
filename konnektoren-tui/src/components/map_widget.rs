use konnektoren_core::game::GamePath;
use konnektoren_platform::i18n::{I18nConfig, Language};
use ratatui::{
    prelude::*,
    style::Styled,
    symbols::Marker,
    widgets::{Block, Widget, canvas::Line, canvas::*},
};

pub struct MapWidget<'a> {
    current_challenge: usize,
    path: &'a GamePath,
    i18n: &'a I18nConfig,
    language: Option<&'a Language>,
}

impl MapWidget<'_> {
    pub fn calculate_bounds(challenges: &[(String, f64, f64)]) -> ([f64; 2], [f64; 2]) {
        let x_min = challenges
            .iter()
            .map(|(_, x, _)| *x)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let x_max = challenges
            .iter()
            .map(|(_, x, _)| *x)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let y_min = challenges
            .iter()
            .map(|(_, _, y)| *y)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let y_max = challenges
            .iter()
            .map(|(_, _, y)| *y)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        ([x_min - 10.0, x_max + 10.0], [y_min - 10.0, y_max + 10.0])
    }
}

impl<'a> MapWidget<'a> {
    pub fn new(
        path: &'a GamePath,
        current_challenge: usize,
        i18n: &'a I18nConfig,
        language: Option<&'a Language>,
    ) -> Self {
        MapWidget {
            path,
            current_challenge,
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

    fn process_challenges(&self) -> Vec<(String, f64, f64)> {
        self.path
            .challenges
            .iter()
            .map(|challenge| {
                let (x, y) = challenge.position.unwrap_or((0, 0));
                (challenge.name.to_string(), x as f64 * 10.0, y as f64 * 10.0)
            })
            .collect()
    }

    fn draw_challenge(&self, index: usize, challenge: &str, x: f64, y: f64, ctx: &mut Context) {
        let color = if self.current_challenge == index {
            Color::Red
        } else {
            Color::Yellow
        };

        ctx.draw(&Rectangle {
            x,
            y,
            width: 1.0,
            height: 1.0,
            color,
        });
        ctx.print(x, y, challenge.to_string().set_style(color));
    }

    fn draw_map(
        &self,
        title: &str,
        challenges: &[(String, f64, f64)],
        x_bounds: [f64; 2],
        y_bounds: [f64; 2],
        area: Rect,
        buf: &mut Buffer,
    ) {
        Canvas::default()
            .block(Block::bordered().title(title))
            .marker(Marker::Braille)
            .paint(|ctx| {
                let mut last: Option<(f64, f64)> = None;
                for (index, (name, x, y)) in challenges.iter().enumerate() {
                    self.draw_challenge(index, name, *x, *y, ctx);
                    if let Some((x1, y1)) = last {
                        ctx.draw(&Line {
                            x1,
                            y1,
                            x2: *x,
                            y2: *y,
                            color: Color::Yellow,
                        });
                    }
                    last = Some((*x, *y));
                }
            })
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .render(area, buf);
    }
}

impl Widget for MapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Canvas::default()
            .block(Block::bordered().title(self.t("World")))
            .marker(Marker::Braille)
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Green,
                    resolution: MapResolution::High,
                });
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .render(area, buf);

        let challenges = self.process_challenges();
        let (x_bounds, y_bounds) = Self::calculate_bounds(&challenges);
        self.draw_map(
            &self.t("Challenges"),
            &challenges,
            x_bounds,
            y_bounds,
            area,
            buf,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_boundaries() {
        let challenges = vec![
            ("Challenge 1".to_string(), 0.0, 0.0),
            ("Challenge 2".to_string(), 10.0, 10.0),
            ("Challenge 3".to_string(), -10.0, -10.0),
        ];

        let bounds = MapWidget::calculate_bounds(&challenges);
        assert_eq!(bounds, ([-20.0, 20.0], [-20.0, 20.0]));
    }
}
