use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

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
#[cfg_attr(test, derive(PartialEq, Clone))]
#[serde(untagged)]
pub enum Binding {
    Single(Action),
    Multi(Vec<Action>),
    Chain(HashMap<String, Binding>),
}

#[derive(Debug, Deserialize)]
pub struct BindConfig(pub HashMap<String, Binding>);

impl std::ops::Index<&str> for BindConfig {
    type Output = Binding;

    fn index(&self, index: &str) -> &Self::Output {
        &self.0[index]
    }
}

impl Default for BindConfig {
    fn default() -> Self {
        Self(
            [
                // general
                ("C-s".to_string(), Binding::Single(Action::Save)),
                ("q".to_string(), Binding::Single(Action::Quit)),
                ("h".to_string(), Binding::Single(Action::CursorLeft)),
                ("l".to_string(), Binding::Single(Action::CursorRight)),
                ("space".to_string(), Binding::Single(Action::Play)),
                ("z".to_string(), Binding::Single(Action::ZoomIn)),
                ("Z".to_string(), Binding::Single(Action::ZoomOut)),
            ]
            .into(),
        )
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub binds: BindConfig,
}

impl Config {
    pub fn read(s: &str) -> Result<Config> {
        let c: Self = toml::from_str(s)?;
        Ok(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_binds() {
        let s = toml::toml! {
            [binds]
            C-c = ["save", "quit"]
            s = "save"
        }
        .to_string();

        let c = Config::read(&s).unwrap();
        let b = c.binds;

        assert_eq!(b.0["C-c"], Binding::Multi(vec![Action::Save, Action::Quit]));
        assert_eq!(b.0["s"], Binding::Single(Action::Save));
    }
}
