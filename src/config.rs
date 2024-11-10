use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Quit,
    Save,
    CursorLeft,
    CursorRight,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Clone))]
#[serde(untagged)]
pub enum Binding {
    Single(Action),
    Multi(Vec<Action>),
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
