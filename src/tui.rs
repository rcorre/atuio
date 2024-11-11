use std::{fs::File, io::BufReader, time::Duration};

use anyhow::Result;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

use ratatui::{
    prelude::*,
    widgets::{block::Title, Axis, Block, Chart, Dataset, GraphType},
};
use rodio::{source::Buffered, Decoder, OutputStream, Sink, Source};

use crate::{
    binds::Binds,
    config::{Action, Config},
};

struct App {
    exit: bool,
    binds: Binds,
    path: std::path::PathBuf,
    _stream: OutputStream,
    sink: Sink,
    source: Buffered<Decoder<BufReader<File>>>,
    cursor: Duration,
    playhead: Duration,
    window_start: Duration,
    window_end: Duration,
    playing: bool,
}

impl App {
    fn new(config: Config, path: std::path::PathBuf) -> Result<Self> {
        let binds = Binds::from_config(config.binds)?;
        log::trace!("Using binds: {binds:#?}");
        let (stream, stream_handle) = OutputStream::try_default()?;

        let file = BufReader::new(File::open(&path)?);
        let source = Decoder::new(file)?.buffered();
        let sink = Sink::try_new(&stream_handle)?;
        let window_end = source.total_duration().unwrap_or(Duration::from_secs(1));

        Ok(Self {
            path,
            binds,
            _stream: stream,
            source,
            sink,
            cursor: Duration::ZERO,
            playhead: Duration::ZERO,
            window_start: Duration::ZERO,
            window_end,
            exit: false,
            playing: false,
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
        log::trace!("Applying action: {action:?}");
        match action {
            Action::Quit => {
                log::info!("Exit requested");
                self.exit = true;
            }
            Action::Save => {
                log::info!("TODO Save not handled");
            }
            Action::CursorLeft => {
                self.cursor = self.cursor.saturating_sub(Duration::from_millis(10));
            }
            Action::CursorRight => {
                self.cursor = self.cursor + Duration::from_millis(10);
                if let Some(max) = self.source.total_duration() {
                    self.cursor = self.cursor.min(max);
                }
            }
            Action::Play => {
                if self.playing {
                    log::debug!("Stopping playback");
                    self.sink.stop();
                } else {
                    self.sink
                        .append(self.source.clone().skip_duration(self.cursor));
                    log::debug!("Starting playback at {:?}", self.cursor);
                }
                self.playing = !self.playing;
            }
            Action::ZoomIn => {
                self.window_end = self.window_end.saturating_sub(Duration::from_millis(100));
            }
            Action::ZoomOut => {
                self.window_end += Duration::from_millis(100);
            }
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        if self.playing {
            self.playhead = self.cursor + self.sink.get_pos();
            if self.sink.empty() {
                log::debug!("Done playing");
                self.playing = false;
            }
            if !event::poll(Duration::from_millis(50))? {
                return Ok(());
            }
        }
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

        let cursor_data = [
            (self.cursor.as_secs_f64(), -1.0),
            (self.cursor.as_secs_f64(), 1.0),
        ];
        let mut datasets = vec![
            // wave
            Dataset::default()
                .name(self.path.file_name().and_then(|f| f.to_str()).unwrap_or(""))
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().cyan())
                .data(data.as_slice()),
            // cursor
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().white())
                .data(&cursor_data),
        ];

        let playhead_data = [
            (self.playhead.as_secs_f64(), -1.0),
            (self.playhead.as_secs_f64(), 1.0),
        ];
        if self.playing {
            datasets.push(
                Dataset::default()
                    .marker(symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().red())
                    .data(&playhead_data),
            )
        }

        let x_axis = Axis::default()
            .style(Style::default().white())
            .bounds([0.0, self.window_end.as_secs_f64()])
            .labels([
                "0.0".to_string(),
                format!("{}", self.window_end.as_secs_f64()),
            ]);

        let y_axis = Axis::default()
            .style(Style::default().white())
            .bounds([-1.0, 1.0])
            .labels(["0.0", "-1.0", "1.0"]);

        let chart = Chart::new(datasets).x_axis(x_axis).y_axis(y_axis);

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
    use super::*;
    use event::KeyCode;
    use insta::assert_snapshot;

    struct Test {
        app: App,
    }

    impl Test {
        fn new() -> Test {
            let app = App::new(
                Config::default(),
                std::path::Path::new("testdata/sine440.wav").to_path_buf(),
            )
            .unwrap();
            Test { app }
        }

        fn load(path: &str) -> Test {
            let app = App::new(
                Config::default(),
                std::path::Path::new("testdata").join(path).to_path_buf(),
            )
            .unwrap();
            Test { app }
        }

        fn render(&self) -> String {
            let mut buf = Buffer::empty(layout::Rect::new(0, 0, 160, 20));
            self.app.render(buf.area, &mut buf);
            buf_string(&buf)
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
        assert_snapshot!("load", test.render());
    }

    #[test]
    fn test_tui_move_cursor() {
        let mut test = Test::new();

        test.input("llll");
        assert_snapshot!("cursor_right", test.render());

        test.input("hh");
        assert_snapshot!("cursor_left", test.render());
    }

    #[test]
    fn test_tui_zoom() {
        let mut test = Test::load("sine440fade.wav");

        assert_snapshot!("zoom_start", test.render());

        test.input("zz");
        assert_snapshot!("zoom_in", test.render());

        test.input("Z");
        assert_snapshot!("zoom_out", test.render());
    }
}
