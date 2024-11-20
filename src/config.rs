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
    CursorStart,
    CursorEnd,
    ZoomIn,
    ZoomOut,
    Select,
    SelectAll,
    Amplify,
    Cut,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub binds: BindMap<Action>,
}

impl Default for Config {
    fn default() -> Self {
        let key = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty());
        Self {
            binds: BindMap::new([
                // general
                (key('s'), Binding::Action(vec![Action::Save])),
                (key('q'), Binding::Action(vec![Action::Quit])),
                (key('h'), Binding::Action(vec![Action::CursorLeft])),
                (key('l'), Binding::Action(vec![Action::CursorRight])),
                (key(' '), Binding::Action(vec![Action::Play])),
                // zoom
                (key('z'), Binding::Action(vec![Action::ZoomIn])),
                (key('Z'), Binding::Action(vec![Action::ZoomOut])),
                // selection
                (key('v'), Binding::Action(vec![Action::Select])),
                (key('%'), Binding::Action(vec![Action::SelectAll])),
                // editing
                (key('a'), Binding::Action(vec![Action::Amplify])),
                (key('x'), Binding::Action(vec![Action::Cut])),
                // g navigation chains
                (
                    key('g'),
                    Binding::Chain(BindMap::new([
                        (key('s'), Binding::Action(vec![Action::CursorStart])),
                        (key('l'), Binding::Action(vec![Action::CursorEnd])),
                    ])),
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
