use std::{fs::File, io::BufReader};

use anyhow::Result;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

use ratatui::{
    prelude::*,
    widgets::{block::Title, Axis, Block, Chart, Dataset, GraphType},
};
use rodio::{source::Buffered, Decoder, OutputStream, OutputStreamHandle, Source};

use crate::{
    binds::Binds,
    config::{Action, Config},
};

struct App {
    exit: bool,
    binds: Binds,
    path: std::path::PathBuf,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    source: Buffered<Decoder<BufReader<File>>>,
}

impl App {
    fn new(config: Config, path: std::path::PathBuf) -> Result<Self> {
        let binds = Binds::from_config(config.binds)?;
        log::trace!("Using binds: {binds:#?}");
        let (stream, stream_handle) = OutputStream::try_default()?;

        let file = BufReader::new(File::open(&path)?);
        let source = Decoder::new(file)?.buffered();

        Ok(Self {
            path,
            binds,
            _stream: stream,
            stream_handle,
            source,
            exit: false,
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
        let title = Title::from("atuio".bold());
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

        let sample_rate = self.source.sample_rate() as f64;
        let data: Vec<_> = self
            .source
            .clone()
            .enumerate()
            .map(|(i, v)| ((i as f64) / sample_rate, (v as f64) / (i16::MAX as f64)))
            .collect();
        // Create the datasets to fill the chart with
        let datasets = vec![
            // Scatter chart
            Dataset::default()
                .name(self.path.file_name().and_then(|f| f.to_str()).unwrap_or(""))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().cyan())
                .data(data.as_slice()),
        ];

        // Create the X axis and define its properties
        let len = (data.len() as f64) / sample_rate;
        let x_axis = Axis::default()
            .title("Time".red())
            .style(Style::default().white())
            .bounds([0.0, len])
            .labels(["0.0".to_string(), format!("{len}")]);

        // Create the Y axis and define its properties
        let y_axis = Axis::default()
            .title("Amplitude".red())
            .style(Style::default().white())
            .bounds([-1.0, 1.0])
            .labels(["0.0", "-1.0", "1.0"]);

        // Create the chart and link all the parts together
        let chart = Chart::new(datasets)
            .block(Block::new().title("Chart"))
            .x_axis(x_axis)
            .y_axis(y_axis);

        chart.render(area, buf);
    }
}

pub fn start(config: Config, path: std::path::PathBuf) -> Result<()> {
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
            let app = App::new(Config::default(), tmp.path().to_path_buf()).unwrap();
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
