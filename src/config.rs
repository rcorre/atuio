use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;

use crate::binds::{BindMap, Binding};

#[derive(Copy, Clone, Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Quit,
    Save,
    Play,
    CursorLeft,
    CursorRight,
    ZoomIn,
    ZoomOut,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub binds: BindMap<Action>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            binds: BindMap::new([
                // general
                (
                    KeyEvent::new(KeyCode::Char('s'), KeyModifiers::SHIFT),
                    Binding::Action(vec![Action::Save]),
                ),
                (
                    KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
                    Binding::Action(vec![Action::Quit]),
                ),
                (
                    KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()),
                    Binding::Action(vec![Action::CursorLeft]),
                ),
                (
                    KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()),
                    Binding::Action(vec![Action::CursorRight]),
                ),
                (
                    KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()),
                    Binding::Action(vec![Action::Play]),
                ),
                (
                    KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty()),
                    Binding::Action(vec![Action::ZoomIn]),
                ),
                (
                    KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::empty()),
                    Binding::Action(vec![Action::ZoomOut]),
                ),
            ]),
        }
    }
}

impl Config {
    pub fn read(s: &str) -> Result<Config> {
        let c: Self = toml::from_str(s)?;
        Ok(c)
    }
}
