//! Ação de cada botão: 4 bytes em `base + 0x42 + 4 * slot`.
//! Tabela em `0x47D3D0` e `FUN_00412980` do software oficial.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers(pub u8);

impl Modifiers {
    pub const CTRL: u8 = 0x01;
    pub const SHIFT: u8 = 0x02;
    pub const ALT: u8 = 0x04;
    /// Tecla Windows, Cmd no macOS, Super no Linux.
    pub const META: u8 = 0x08;

    pub fn has(self, m: u8) -> bool {
        self.0 & m != 0
    }
}

/// Alvo do tiro rápido: botão do mouse ou tecla (uso HID).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FireTarget {
    Left,
    Right,
    Middle,
    Key(u8),
}

impl FireTarget {
    fn code(self) -> u8 {
        match self {
            Self::Left => 0x81,
            Self::Right => 0x82,
            Self::Middle => 0x84,
            Self::Key(k) => k,
        }
    }

    fn from_code(c: u8) -> Self {
        match c {
            0x81 => Self::Left,
            0x82 => Self::Right,
            0x84 => Self::Middle,
            k => Self::Key(k),
        }
    }
}

/// Submenus DPI, RATE, SCROLL e PROFILE SWITCH da interface oficial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Special {
    DpiCycle,
    DpiUp,
    DpiDown,
    ScrollUp,
    ScrollDown,
    ProfileSwitch,
    RateUp,
    RateDown,
    LedModeSwitch,
}

impl Special {
    pub const ALL: [Special; 9] = [
        Self::DpiCycle,
        Self::DpiUp,
        Self::DpiDown,
        Self::ScrollUp,
        Self::ScrollDown,
        Self::ProfileSwitch,
        Self::RateUp,
        Self::RateDown,
        Self::LedModeSwitch,
    ];

    fn bytes(self) -> [u8; 4] {
        match self {
            Self::DpiCycle => [0x88, 0, 0, 0],
            Self::DpiDown => [0x89, 0, 0, 0],
            Self::DpiUp => [0x8A, 0, 0, 0],
            Self::ScrollUp => [0x8B, 0, 0, 0],
            Self::ScrollDown => [0x8C, 0, 0, 0],
            Self::ProfileSwitch => [0x8D, 0, 0, 0],
            Self::RateUp => [0x97, 0, 0, 0],
            Self::RateDown => [0x98, 0, 0, 0],
            Self::LedModeSwitch => [0x9B, 0x04, 0, 0],
        }
    }
}

/// Atalhos BASIC e ADVANCED (`8E 01 FF <id>`). O mouse só avisa o PC; quem
/// executa é o programa no PC (`FUN_00410be0` do oficial). Ver `host_action`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Shortcut {
    Cut = 0x11,
    Copy,
    Paste,
    SelectAll,
    Find,
    New,
    Print,
    Save,
    SwitchWindow,
    CloseWindow,
    OpenFileManager,
    Run,
    ShowDesktop,
    Lock,
    BrowserHome,
    BrowserBack,
    BrowserForward,
    BrowserStop,
    BrowserRefresh,
    BrowserSearch,
    BrowserFavorites,
    Mail,
}

impl Shortcut {
    pub const ALL: [Shortcut; 22] = [
        Self::Cut,
        Self::Copy,
        Self::Paste,
        Self::SelectAll,
        Self::Find,
        Self::New,
        Self::Print,
        Self::Save,
        Self::SwitchWindow,
        Self::CloseWindow,
        Self::OpenFileManager,
        Self::Run,
        Self::ShowDesktop,
        Self::Lock,
        Self::BrowserHome,
        Self::BrowserBack,
        Self::BrowserForward,
        Self::BrowserStop,
        Self::BrowserRefresh,
        Self::BrowserSearch,
        Self::BrowserFavorites,
        Self::Mail,
    ];

    pub fn id(self) -> u8 {
        self as u8
    }

    pub fn from_id(id: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    Disabled,
    LeftClick,
    RightClick,
    MiddleClick,
    Back,
    Forward,
    /// Uso HID da tecla (página 0x07).
    Key(u8),
    Combo {
        modifiers: Modifiers,
        key: u8,
    },
    /// Uso da página Consumer, ex. `0xCD` play/pausa.
    Media(u8),
    Fire {
        target: FireTarget,
        times: u8,
        delay_ms: u16,
    },
    /// `repeat_while_held`: repete enquanto o botão está apertado; senão roda `count` vezes.
    Macro {
        index: u8,
        repeat_while_held: bool,
        count: u16,
    },
    Special(Special),
    Shortcut(Shortcut),
    /// Código que a tabela oficial não tem.
    Raw([u8; 4]),
}

impl ButtonAction {
    pub fn encode(&self) -> [u8; 4] {
        match *self {
            Self::Disabled => [0, 0, 0, 0],
            Self::LeftClick => [0x81, 0, 0, 0],
            Self::RightClick => [0x82, 0, 0, 0],
            Self::MiddleClick => [0x83, 0, 0, 0],
            Self::Back => [0x84, 0, 0, 0],
            Self::Forward => [0x85, 0, 0, 0],
            Self::Key(k) => [0x90, 0, k, 0],
            Self::Combo { modifiers, key } => [0x8F, modifiers.0, key, 0],
            Self::Media(u) => [0x8E, 0x01, u, 0],
            Self::Fire {
                target,
                times,
                delay_ms,
            } => [0x99, target.code(), times, (delay_ms / 10).min(255) as u8],
            Self::Macro {
                index,
                repeat_while_held,
                count,
            } => {
                let i = index | if repeat_while_held { 0x80 } else { 0 };
                [0x91, i, (count & 0xFF) as u8, (count >> 8) as u8]
            }
            Self::Special(s) => s.bytes(),
            Self::Shortcut(s) => [0x8E, 0x01, 0xFF, s.id()],
            Self::Raw(b) => b,
        }
    }

    pub fn decode(b: &[u8]) -> Self {
        let raw = [b[0], b[1], b[2], b[3]];
        match b[0] {
            0x00 => Self::Disabled,
            0x81 => Self::LeftClick,
            0x82 => Self::RightClick,
            0x83 => Self::MiddleClick,
            0x84 => Self::Back,
            0x85 => Self::Forward,
            0x90 => Self::Key(b[2]),
            0x8F => Self::Combo {
                modifiers: Modifiers(b[1]),
                key: b[2],
            },
            0x8E if b[1] == 0x01 && b[2] != 0xFF => Self::Media(b[2]),
            0x8E if b[1] == 0x01 => Shortcut::from_id(b[3]).map_or(Self::Raw(raw), Self::Shortcut),
            0x99 => Self::Fire {
                target: FireTarget::from_code(b[1]),
                times: b[2],
                delay_ms: b[3] as u16 * 10,
            },
            0x91 => Self::Macro {
                index: b[1] & 0x7F,
                repeat_while_held: b[1] & 0x80 != 0,
                count: b[2] as u16 | (b[3] as u16) << 8,
            },
            _ => Special::ALL
                .into_iter()
                .find(|s| s.bytes() == raw)
                .map_or(Self::Raw(raw), Self::Special),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let actions = [
            ButtonAction::LeftClick,
            ButtonAction::Back,
            ButtonAction::Key(0x1E),
            ButtonAction::Combo {
                modifiers: Modifiers(Modifiers::CTRL | Modifiers::SHIFT),
                key: 0x09,
            },
            ButtonAction::Media(0xCD),
            ButtonAction::Fire {
                target: FireTarget::Left,
                times: 3,
                delay_ms: 0,
            },
            ButtonAction::Fire {
                target: FireTarget::Key(0x04),
                times: 5,
                delay_ms: 50,
            },
            ButtonAction::Macro {
                index: 0,
                repeat_while_held: false,
                count: 1,
            },
            ButtonAction::Macro {
                index: 2,
                repeat_while_held: true,
                count: 1,
            },
            ButtonAction::Disabled,
            ButtonAction::Raw([0x30, 0, 0, 0]),
            ButtonAction::Special(Special::DpiUp),
            ButtonAction::Special(Special::LedModeSwitch),
            ButtonAction::Shortcut(Shortcut::Copy),
            ButtonAction::Shortcut(Shortcut::Mail),
        ];
        for a in actions {
            assert_eq!(ButtonAction::decode(&a.encode()), a);
        }
    }

    #[test]
    fn codes_of_real_dump() {
        // Perfil 1 de fábrica: botões 4, 5, 6, 19 e 20.
        assert_eq!(
            ButtonAction::decode(&[0x99, 0x81, 0x03, 0]),
            ButtonAction::Fire {
                target: FireTarget::Left,
                times: 3,
                delay_ms: 0
            }
        );
        assert_eq!(
            ButtonAction::decode(&[0x8A, 0, 0, 0]),
            ButtonAction::Special(Special::DpiUp)
        );
        assert_eq!(
            ButtonAction::decode(&[0x89, 0, 0, 0]),
            ButtonAction::Special(Special::DpiDown)
        );
        assert_eq!(
            ButtonAction::decode(&[0x8B, 0, 0, 0]),
            ButtonAction::Special(Special::ScrollUp)
        );
        assert_eq!(
            ButtonAction::decode(&[0x8C, 0, 0, 0]),
            ButtonAction::Special(Special::ScrollDown)
        );
        assert_eq!(
            ButtonAction::Shortcut(Shortcut::Cut).encode(),
            [0x8E, 0x01, 0xFF, 0x11]
        );
        assert_eq!(
            ButtonAction::Shortcut(Shortcut::Mail).encode(),
            [0x8E, 0x01, 0xFF, 0x26]
        );
    }
}
