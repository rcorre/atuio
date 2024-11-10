use std::collections::HashMap;

use anyhow::{bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use crate::config::{self, Binding};

#[derive(Default, Debug)]
pub struct Binds(HashMap<KeyEvent, Binding>);

impl Binds {
    pub fn get(&self, ev: &KeyEvent) -> Option<&Binding> {
        self.0.get(&ev)
    }

    pub fn from_config(c: config::BindConfig) -> Result<Self> {
        let mut m = HashMap::new();
        for (k, v) in c.0.into_iter() {
            m.insert(map_key(&k)?, v);
        }
        Ok(Self(m))
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
        unknown => bail!("Unknown key: {unknown}"),
    };
    let mut modifiers = KeyModifiers::empty();
    for p in parts {
        modifiers.insert(match p {
            "s" | "S" => KeyModifiers::SHIFT,
            "c" | "C" => KeyModifiers::CONTROL,
            "a" | "A" => KeyModifiers::ALT,
            m => bail!(format!("Unknown modifier {m}")),
        });
    }
    Ok(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    })
}

impl Binds {}

// TODO
// #[cfg(test)]
// mod tests {
//     use config::{Action, BindConfig, Binding};

//     use super::*;

//     #[test]
//     fn test_binds() {
//         let s = Binding::Single(Action::MoveCursorUp);
//         let shift_s = Binding::Multi(vec![Action::MoveCursorUp; 6]);
//         let shift_l = Binding::Single(Action::Save);
//         let shift_x = Binding::Multi(vec![Action::Save, Action::Quit]);
//         let ctrl_s = Binding::Single(Action::LineAddPoint);
//         let enter = Binding::Single(Action::ExitMode);
//         let ctrl_shift_tab = Binding::Single(Action::Delete);
//         let alt_enter = Binding::Single(Action::Undo);
//         let b = Binds::from_config(BindConfig(
//             [
//                 ("s".into(), s.clone()),
//                 ("S".into(), shift_s.clone()),
//                 ("S-l".into(), shift_l.clone()),
//                 ("s-X".into(), shift_x.clone()),
//                 ("C-s".into(), ctrl_s.clone()),
//                 ("enter".into(), enter.clone()),
//                 ("C-S-tab".into(), ctrl_shift_tab.clone()),
//                 ("a-enter".into(), alt_enter.clone()),
//             ]
//             .into(),
//         ))
//         .unwrap();

//         assert_eq!(
//             b.get(&KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty())),
//             Some(&s)
//         );

//         for ev in [
//             KeyEvent::new(KeyCode::Char('S'), KeyModifiers::empty()),
//             KeyEvent::new(KeyCode::Char('s'), KeyModifiers::SHIFT),
//             KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT),
//         ] {
//             assert_eq!(b.get(&ev), Some(&shift_s));
//         }

//         for ev in [
//             KeyEvent::new(KeyCode::Char('L'), KeyModifiers::empty()),
//             KeyEvent::new(KeyCode::Char('l'), KeyModifiers::SHIFT),
//             KeyEvent::new(KeyCode::Char('L'), KeyModifiers::SHIFT),
//         ] {
//             assert_eq!(b.get(&ev), Some(&shift_l));
//         }

//         for ev in [
//             KeyEvent::new(KeyCode::Char('X'), KeyModifiers::empty()),
//             KeyEvent::new(KeyCode::Char('x'), KeyModifiers::SHIFT),
//             KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT),
//         ] {
//             assert_eq!(b.get(&ev), Some(&shift_x));
//         }

//         assert_eq!(
//             b.get(&KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
//             Some(&ctrl_s)
//         );
//         assert_eq!(
//             b.get(&KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT)),
//             None
//         );
//         assert_eq!(
//             b.get(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())),
//             Some(&enter)
//         );
//         assert_eq!(
//             b.get(&KeyEvent::new(
//                 KeyCode::Tab,
//                 KeyModifiers::SHIFT | KeyModifiers::CONTROL
//             )),
//             Some(&ctrl_shift_tab)
//         );
//         assert_eq!(
//             b.get(&KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT,)),
//             Some(&alt_enter)
//         );
//     }
// }
