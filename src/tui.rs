use anyhow::Result;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

use ratatui::{
    prelude::*,
    widgets::{block::Title, Axis, Block, Chart, Dataset, GraphType},
};

use crate::{
    binds::Binds,
    config::{Action, Config},
};

#[derive(Default)]
struct App {
    exit: bool,
    binds: Binds,
    path: Option<std::path::PathBuf>,
}

impl App {
    fn new(config: Config, path: Option<std::path::PathBuf>) -> Result<Self> {
        let binds = Binds::from_config(config.binds)?;
        log::trace!("Using binds: {binds:#?}");
        Ok(Self {
            path,
            binds,
            ..Default::default()
        })
    }

    fn run(&mut self, mut terminal: ratatui::DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn apply_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                log::info!("Exit requested");
                self.exit = true;
            }
            Action::Save => {
                log::info!("TODO Save not handled");
            }
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let Some(bound) = self.binds.get(&key) else {
            log::trace!("Mapped key to no action");
            return Ok(());
        };
        log::trace!("Mapped key to {bound:?}");

        match bound {
            crate::config::Binding::Single(s) => self.apply_action(s.clone())?,
            crate::config::Binding::Multi(m) => {
                for action in m.clone() {
                    self.apply_action(action)?;
                }
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut Buffer) {
        let title = Title::from("Boxt".bold());
        let instructions = Title::from(ratatui::text::Line::from(vec![
            " Move ".into(),
            "<WASD>".blue().bold(),
            " Rect ".into(),
            "<R>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(ratatui::widgets::block::Position::Bottom),
            )
            .border_set(ratatui::symbols::border::THICK);
        block.render(area, buf);

        // Create the datasets to fill the chart with
        let datasets = vec![
            // Scatter chart
            Dataset::default()
                .name("data1")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().cyan())
                .data(&[(0.0, 5.0), (1.0, 6.0), (1.5, 6.434)]),
            // Line chart
            Dataset::default()
                .name("data2")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().magenta())
                .data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)]),
        ];

        // Create the X axis and define its properties
        let x_axis = Axis::default()
            .title("X Axis".red())
            .style(Style::default().white())
            .bounds([0.0, 10.0])
            .labels(["0.0", "5.0", "10.0"]);

        // Create the Y axis and define its properties
        let y_axis = Axis::default()
            .title("Y Axis".red())
            .style(Style::default().white())
            .bounds([0.0, 10.0])
            .labels(["0.0", "5.0", "10.0"]);

        // Create the chart and link all the parts together
        let chart = Chart::new(datasets)
            .block(Block::new().title("Chart"))
            .x_axis(x_axis)
            .y_axis(y_axis);

        chart.render(area, buf);
    }
}

pub fn start(config: Config, path: Option<std::path::PathBuf>) -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let app_result = App::new(config, path)?.run(terminal);
    ratatui::restore();
    app_result
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use event::KeyCode;
    use insta::assert_snapshot;

    struct Test {
        app: App,
        tmp: tempfile::NamedTempFile,
    }

    impl Test {
        fn new() -> Test {
            Test::load(&[])
        }

        fn load(lines: &[&str]) -> Test {
            let mut tmp = tempfile::NamedTempFile::new().unwrap();
            tmp.write_all(lines.join("\n").as_bytes()).unwrap();
            tmp.flush().unwrap();
            let app = App::new(Config::default(), Some(tmp.path().to_path_buf())).unwrap();
            Test { app, tmp }
        }

        fn render(&self) -> String {
            let mut buf = Buffer::empty(layout::Rect::new(0, 0, 32, 8));
            self.app.render(buf.area, &mut buf);
            buf_string(&buf)
        }

        fn key(&mut self, key: KeyCode) {
            self.app.handle_key_event(key.into()).unwrap();
        }

        fn input(&mut self, keys: &str) {
            let chars: Vec<_> = keys.chars().collect();
            input(&mut self.app, chars.as_slice());
        }
    }

    fn buf_string(buf: &Buffer) -> String {
        buf.content
            .chunks(buf.area.width as usize)
            .map(|line| {
                line.iter()
                    .map(|cell| cell.symbol().to_string())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn input(app: &mut App, keys: &[char]) {
        for c in keys {
            app.handle_key_event(KeyCode::Char(*c).into()).unwrap();
        }
    }

    #[test]
    fn test_tui_render_empty() {
        let test = Test::new();
        assert_snapshot!(test.render());
    }

    #[test]
    fn test_tui_draw_rect() {
        let mut test = Test::new();

        // Draw one rect and confirm it
        test.input("rsd");
        test.key(KeyCode::Esc);

        // Start drawing another rect
        test.input("ddrsd");

        assert_snapshot!(test.render());
    }
}
