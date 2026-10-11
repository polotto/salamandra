//! Nomes das teclas pelo uso HID (página 0x07), o código que o mouse grava em
//! tecla, combinação, tiro rápido e macro.

use std::sync::OnceLock;

pub fn all() -> &'static [(String, u8)] {
    static KEYS: OnceLock<Vec<(String, u8)>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let mut k: Vec<(String, u8)> = Vec::new();
        for (i, c) in ('a'..='z').enumerate() {
            k.push((c.to_string(), 0x04 + i as u8));
        }
        for (i, c) in "1234567890".chars().enumerate() {
            k.push((c.to_string(), 0x1E + i as u8));
        }
        let named: &[(&str, u8)] = &[
            ("enter", 0x28),
            ("esc", 0x29),
            ("backspace", 0x2A),
            ("tab", 0x2B),
            ("space", 0x2C),
            ("minus", 0x2D),
            ("equal", 0x2E),
            ("lbracket", 0x2F),
            ("rbracket", 0x30),
            ("backslash", 0x31),
            ("semicolon", 0x33),
            ("quote", 0x34),
            ("grave", 0x35),
            ("comma", 0x36),
            ("period", 0x37),
            ("slash", 0x38),
            ("capslock", 0x39),
        ];
        k.extend(named.iter().map(|(n, u)| (n.to_string(), *u)));
        for i in 0..12u8 {
            k.push((format!("f{}", i + 1), 0x3A + i));
        }
        let named: &[(&str, u8)] = &[
            ("printscreen", 0x46),
            ("scrolllock", 0x47),
            ("pause", 0x48),
            ("insert", 0x49),
            ("home", 0x4A),
            ("pageup", 0x4B),
            ("delete", 0x4C),
            ("end", 0x4D),
            ("pagedown", 0x4E),
            ("right", 0x4F),
            ("left", 0x50),
            ("down", 0x51),
            ("up", 0x52),
            ("numlock", 0x53),
            ("kp_slash", 0x54),
            ("kp_star", 0x55),
            ("kp_minus", 0x56),
            ("kp_plus", 0x57),
            ("kp_enter", 0x58),
        ];
        k.extend(named.iter().map(|(n, u)| (n.to_string(), *u)));
        for i in 0..9u8 {
            k.push((format!("kp_{}", i + 1), 0x59 + i));
        }
        k.extend([
            ("kp_0".into(), 0x62),
            ("kp_period".into(), 0x63),
            ("menu".into(), 0x65),
        ]);
        for i in 0..12u8 {
            k.push((format!("f{}", i + 13), 0x68 + i));
        }
        let mods: &[(&str, u8)] = &[
            ("lctrl", 0xE0),
            ("lshift", 0xE1),
            ("lalt", 0xE2),
            ("lmeta", 0xE3),
            ("rctrl", 0xE4),
            ("rshift", 0xE5),
            ("ralt", 0xE6),
            ("rmeta", 0xE7),
        ];
        k.extend(mods.iter().map(|(n, u)| (n.to_string(), *u)));
        k
    })
}

pub fn name(usage: u8) -> String {
    all()
        .iter()
        .find(|(_, u)| *u == usage)
        .map_or_else(|| format!("0x{usage:02x}"), |(n, _)| n.clone())
}

/// Aceita o nome (`a`, `f5`, `lmeta`), os apelidos `lcmd`/`lwin` e o uso em hex (`0x04`).
pub fn usage(text: &str) -> Option<u8> {
    let t = text.to_lowercase();
    let t = match t.as_str() {
        "lcmd" | "lwin" | "lsuper" => "lmeta".to_string(),
        "rcmd" | "rwin" | "rsuper" => "rmeta".to_string(),
        _ => t,
    };
    if let Some((_, u)) = all().iter().find(|(n, _)| *n == t) {
        return Some(*u);
    }
    t.strip_prefix("0x")
        .and_then(|h| u8::from_str_radix(h, 16).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn names_and_usages_are_unique() {
        let names: HashSet<_> = all().iter().map(|(n, _)| n).collect();
        let usages: HashSet<_> = all().iter().map(|(_, u)| u).collect();
        assert_eq!(names.len(), all().len());
        assert_eq!(usages.len(), all().len());
    }

    #[test]
    fn lookup() {
        assert_eq!(usage("A"), Some(0x04));
        assert_eq!(usage("kp_plus"), Some(0x57));
        assert_eq!(usage("lcmd"), Some(0xE3));
        assert_eq!(usage("0x1e"), Some(0x1E));
        assert_eq!(usage("naoexiste"), None);
        assert_eq!(name(0xE3), "lmeta");
        assert_eq!(name(0xF0), "0xf0");
    }
}
