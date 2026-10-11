//! Macros: até 63 eventos de 3 bytes em `0x478 + 200i`. Formato do editor
//! oficial (`ldcfg.exe`), conferido com as macros gravadas no mouse.

use crate::keys;
use crate::map::MACRO_SIZE;
use crate::Error;

pub const MAX_EVENTS: usize = 63;
/// O atraso vai em unidades de 10 ms, em 16 bits.
pub const MAX_DELAY_MS: u32 = 0xFFFF * 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    pub const ALL: [MouseButton; 3] = [Self::Left, Self::Right, Self::Middle];

    fn code(self) -> u8 {
        match self {
            Self::Left => 1,
            Self::Right => 2,
            Self::Middle => 4,
        }
    }

    fn from_code(c: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.code() == c)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Middle => "middle",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum MacroEvent {
    KeyDown(u8),
    KeyUp(u8),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    /// Milissegundos, múltiplo de 10, de 10 a 655350.
    Delay(u32),
}

impl MacroEvent {
    fn encode(self) -> [u8; 3] {
        match self {
            Self::KeyDown(k) => [0x84, k, 0],
            Self::KeyUp(k) => [0x04, k, 0],
            Self::MouseDown(b) => [0x81, b.code(), 0],
            Self::MouseUp(b) => [0x01, b.code(), 0],
            Self::Delay(ms) => {
                let units = (ms / 10) as u16;
                [0x06, (units & 0xFF) as u8, (units >> 8) as u8]
            }
        }
    }

    fn decode(b: &[u8]) -> Option<Self> {
        match b[0] {
            0x84 => Some(Self::KeyDown(b[1])),
            0x04 => Some(Self::KeyUp(b[1])),
            0x81 => MouseButton::from_code(b[1]).map(Self::MouseDown),
            0x01 => MouseButton::from_code(b[1]).map(Self::MouseUp),
            0x06 => Some(Self::Delay((b[1] as u32 | (b[2] as u32) << 8) * 10)),
            _ => None,
        }
    }
}

pub fn encode(events: &[MacroEvent]) -> Result<Vec<u8>, Error> {
    if events.len() > MAX_EVENTS {
        return Err(Error::Invalid(format!(
            "macro com {} eventos; o máximo é {MAX_EVENTS}",
            events.len()
        )));
    }
    for e in events {
        if let MacroEvent::Delay(ms) = e {
            if *ms < 10 || *ms > MAX_DELAY_MS || !ms.is_multiple_of(10) {
                return Err(Error::Invalid(format!(
                    "atraso de {ms} ms: use múltiplos de 10, de 10 a {MAX_DELAY_MS}"
                )));
            }
        }
    }
    let mut bytes: Vec<u8> = events.iter().flat_map(|e| e.encode()).collect();
    bytes.resize(MACRO_SIZE, 0);
    Ok(bytes)
}

/// Para no primeiro evento vazio ou desconhecido.
pub fn decode(bytes: &[u8]) -> Vec<MacroEvent> {
    let (chunks, _) = bytes.as_chunks::<3>();
    chunks
        .iter()
        .take(MAX_EVENTS)
        .take_while(|c| c[0] != 0)
        .map_while(|c| MacroEvent::decode(c))
        .collect()
}

/// Texto de macro, um evento por palavra: `down:<tecla> up:<tecla> tap:<tecla>
/// mdown:<botão> mup:<botão> click:<botão> delay:<ms>`. Mesmo formato no CLI e no app.
pub fn parse(text: &str) -> Result<Vec<MacroEvent>, Error> {
    let mut events = Vec::new();
    for word in text.split_whitespace() {
        let (kind, arg) = word
            .split_once(':')
            .ok_or_else(|| Error::Invalid(format!("evento inválido: {word}")))?;
        let key =
            || keys::usage(arg).ok_or_else(|| Error::Invalid(format!("tecla desconhecida: {arg}")));
        let button = || {
            MouseButton::ALL
                .into_iter()
                .find(|b| b.name() == arg)
                .ok_or_else(|| {
                    Error::Invalid(format!(
                        "botão do mouse inválido: {arg} (use left, right, middle)"
                    ))
                })
        };
        match kind {
            "down" => events.push(MacroEvent::KeyDown(key()?)),
            "up" => events.push(MacroEvent::KeyUp(key()?)),
            "tap" => {
                let k = key()?;
                events.extend([MacroEvent::KeyDown(k), MacroEvent::KeyUp(k)]);
            }
            "mdown" => events.push(MacroEvent::MouseDown(button()?)),
            "mup" => events.push(MacroEvent::MouseUp(button()?)),
            "click" => {
                let b = button()?;
                events.extend([MacroEvent::MouseDown(b), MacroEvent::MouseUp(b)]);
            }
            "delay" => {
                let ms = arg
                    .parse()
                    .map_err(|_| Error::Invalid(format!("atraso inválido: {arg}")))?;
                events.push(MacroEvent::Delay(ms));
            }
            _ => return Err(Error::Invalid(format!("evento inválido: {word}"))),
        }
    }
    encode(&events)?;
    Ok(events)
}

pub fn format(events: &[MacroEvent]) -> String {
    events
        .iter()
        .map(|e| match e {
            MacroEvent::KeyDown(k) => format!("down:{}", keys::name(*k)),
            MacroEvent::KeyUp(k) => format!("up:{}", keys::name(*k)),
            MacroEvent::MouseDown(b) => format!("mdown:{}", b.name()),
            MacroEvent::MouseUp(b) => format!("mup:{}", b.name()),
            MacroEvent::Delay(ms) => format!("delay:{ms}"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_real_macros() {
        // Macros 1 e 2 lidas do mouse: Shift duas vezes; Win+V.
        let mut m1 = vec![0x84, 0xE1, 0, 0x04, 0xE1, 0, 0x84, 0xE1, 0, 0x04, 0xE1, 0];
        m1.resize(200, 0);
        assert_eq!(decode(&m1).len(), 4);
        let mut m2 = vec![0x84, 0xE3, 0, 0x84, 0x19, 0, 0x04, 0x19, 0, 0x04, 0xE3, 0];
        m2.resize(200, 0);
        assert_eq!(format(&decode(&m2)), "down:lmeta down:v up:v up:lmeta");
    }

    #[test]
    fn delay_is_16_bit_in_10ms_units() {
        let bytes = encode(&[
            MacroEvent::Delay(10),
            MacroEvent::Delay(5000),
            MacroEvent::Delay(MAX_DELAY_MS),
        ])
        .unwrap();
        assert_eq!(
            &bytes[..9],
            &[0x06, 1, 0, 0x06, 0xF4, 0x01, 0x06, 0xFF, 0xFF]
        );
        assert_eq!(decode(&bytes)[1], MacroEvent::Delay(5000));
        assert!(encode(&[MacroEvent::Delay(5)]).is_err());
        assert!(encode(&[MacroEvent::Delay(15)]).is_err());
    }

    #[test]
    fn mouse_buttons() {
        let e = [
            MacroEvent::MouseDown(MouseButton::Middle),
            MacroEvent::MouseUp(MouseButton::Middle),
        ];
        assert_eq!(&encode(&e).unwrap()[..6], &[0x81, 4, 0, 0x01, 4, 0]);
    }

    #[test]
    fn limits() {
        assert!(encode(&[MacroEvent::KeyDown(4); 63]).is_ok());
        assert!(encode(&[MacroEvent::KeyDown(4); 64]).is_err());
        assert_eq!(encode(&[]).unwrap(), vec![0; 200]);
    }

    #[test]
    fn text_round_trip() {
        let events = parse("down:lcmd tap:v up:lcmd delay:30 click:left").unwrap();
        assert_eq!(events.len(), 7);
        assert_eq!(parse(&format(&events)).unwrap(), events);
        assert!(parse("tap:naoexiste").is_err());
        assert!(parse("click:back").is_err());
        assert!(parse("delay:7").is_err());
    }
}
