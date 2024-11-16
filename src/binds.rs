use std::collections::HashMap;

use anyhow::{bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use serde::Deserialize;

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct BindMap<Action>(HashMap<KeyEvent, Binding<Action>>);

impl<Action> BindMap<Action> {
    pub fn new<T: Into<HashMap<KeyEvent, Binding<Action>>>>(map: T) -> Self {
        Self(map.into())
    }
}

impl<'de, Action> Deserialize<'de> for BindMap<Action>
where
    Action: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize, Debug)]
        pub struct Serialized<Action>(HashMap<String, Binding<Action>>);

        let parsed = Serialized::deserialize(deserializer)?;
        let mut map = HashMap::new();
        for (k, v) in parsed.0 {
            let k = map_key(&k).map_err(serde::de::Error::custom)?;
            map.insert(k, v);
        }
        Ok(Self(map))
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Binding<Action> {
    Action(Vec<Action>),
    Chain(BindMap<Action>),
}

impl<'de, Action> Deserialize<'de> for Binding<Action>
where
    Action: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[cfg_attr(test, derive(PartialEq))]
        #[serde(untagged)]
        pub enum Serialized<Action> {
            Single(Action),
            Multi(Vec<Action>),
            Chain(BindMap<Action>),
        }
        let parsed = Serialized::deserialize(deserializer)?;
        Ok(match parsed {
            Serialized::Single(a) => Binding::Action(vec![a]),
            Serialized::Multi(a) => Binding::Action(a),
            Serialized::Chain(c) => Binding::Chain(c),
        })
    }
}

#[derive(Debug)]
pub struct Binds<Action> {
    map: BindMap<Action>,
    keys: Vec<KeyEvent>,
}

impl<Action> Binds<Action> {
    pub fn new(map: BindMap<Action>) -> Self {
        Self { map, keys: vec![] }
    }

    pub fn apply(&mut self, key: KeyEvent) -> Option<&Vec<Action>> {
        let mut bound = &self.map;
        self.keys.push(key);
        for k in &self.keys {
            bound = match bound.0.get(&k) {
                Some(Binding::Chain(c)) => c,
                Some(Binding::Action(a)) => {
                    self.keys.clear();
                    return Some(a);
                }
                None => {
                    log::trace!("{:?} bound to nothing", self.keys);
                    self.keys.clear();
                    return None;
                }
            }
        }
        log::trace!("key chain: {:?}", self.keys);
        None
    }
}

fn map_key(key: &str) -> Result<KeyEvent> {
    let mut parts = key.split('-').rev();
    let Some(code) = parts.next() else {
        bail!("Empty key");
    };
    let code = match code {
        c if c.len() == 1 => KeyCode::Char(c.chars().next().unwrap()),
        s if s.starts_with("f") => {
            let (_, num) = s.split_at(1);
            let num = num.parse()?;
            KeyCode::F(num)
        }
        "space" => KeyCode::Char(' '),
        "backspace" => KeyCode::Backspace,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "null" => KeyCode::Null,
        "esc" => KeyCode::Esc,
        "capslock" => KeyCode::CapsLock,
        "scrolllock" => KeyCode::ScrollLock,
        "numlock" => KeyCode::NumLock,
        "print" => KeyCode::PrintScreen,
        "pause" => KeyCode::Pause,
        "menu" => KeyCode::Menu,
        "keypadbegin" => KeyCode::KeypadBegin,
        unknown => bail!("Unknown keycode: {unknown}"),
    };
    let mut modifiers = KeyModifiers::empty();
    for p in parts {
        modifiers.insert(match p {
            "s" | "S" => KeyModifiers::SHIFT,
            "c" | "C" => KeyModifiers::CONTROL,
            "a" | "A" => KeyModifiers::ALT,
            m => bail!(format!("Unknown key modifier: {m}")),
        });
    }
    Ok(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Action {
        One,
        Two,
        Three,
        Four,
    }

    #[test]
    fn test_binds() {
        use Action::*;

        let map: BindMap<Action> = toml::from_str(
            &toml::toml! {
                a = "one"
                s-s = "two"
                S-l = "three"
                X = "four"
                c-s = ["four", "four"]
                [space]
                z = "four"
                enter = ["one", "two"]
            }
            .to_string(),
        )
        .unwrap();

        let mut binds = Binds::new(map);

        assert_eq!(
            binds.apply(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())),
            Some(&vec![One])
        );

        for ev in [
            KeyEvent::new(KeyCode::Char('S'), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT),
        ] {
            assert_eq!(binds.apply(ev), Some(&vec![Two]));
        }

        for ev in [
            KeyEvent::new(KeyCode::Char('L'), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('L'), KeyModifiers::SHIFT),
        ] {
            assert_eq!(binds.apply(ev), Some(&vec![Three]));
        }

        for ev in [
            KeyEvent::new(KeyCode::Char('X'), KeyModifiers::empty()),
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT),
        ] {
            assert_eq!(binds.apply(ev), Some(&vec![Four]));
        }

        assert_eq!(
            binds.apply(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
            Some(&vec![Four, Four])
        );

        assert_eq!(
            binds.apply(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT)),
            None
        );
        assert_eq!(
            binds.apply(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())),
            None,
        );

        // space - z
        assert_eq!(
            binds.apply(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty())),
            None,
        );
        assert_eq!(
            binds.apply(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty())),
            Some(&vec![Four]),
        );

        // space - enter
        assert_eq!(
            binds.apply(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty())),
            None,
        );
        assert_eq!(
            binds.apply(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())),
            Some(&vec![One, Two]),
        );
    }
}
