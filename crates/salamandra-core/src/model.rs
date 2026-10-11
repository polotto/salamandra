//! Configuração de um perfil e a codificação de cada campo.

use crate::buttons::ButtonAction;
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PollingRate {
    Hz125,
    Hz250,
    Hz500,
    Hz1000,
}

impl PollingRate {
    pub const ALL: [PollingRate; 4] = [Self::Hz125, Self::Hz250, Self::Hz500, Self::Hz1000];

    /// Código do firmware: 8, 4, 2, 1 (confirmado pelo mouse_m908 e pelo padrão de fábrica).
    pub fn code(self) -> u8 {
        match self {
            Self::Hz125 => 0x08,
            Self::Hz250 => 0x04,
            Self::Hz500 => 0x02,
            Self::Hz1000 => 0x01,
        }
    }

    pub fn from_code(c: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.code() == c)
    }

    pub fn hertz(self) -> u32 {
        1000 / self.code() as u32
    }

    pub fn from_hertz(hz: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.hertz() == hz)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DpiLevel {
    pub enabled: bool,
    pub x: u32,
    pub y: u32,
}

impl DpiLevel {
    pub const MIN: u32 = 100;
    pub const MAX: u32 = 24000;
    pub const STEP: u32 = 100;

    pub fn new(x: u32) -> Self {
        Self {
            enabled: true,
            x,
            y: x,
        }
    }

    pub fn xy(x: u32, y: u32) -> Self {
        Self {
            enabled: true,
            x,
            y,
        }
    }

    /// `ativo, x, xf, y, yf` (`FUN_0040ff40`).
    pub fn encode(&self) -> Result<[u8; 5], Error> {
        let (xb, xf) = encode_dpi(self.x)?;
        let (yb, yf) = encode_dpi(self.y)?;
        Ok([self.enabled as u8, xb, xf, yb, yf])
    }

    pub fn decode(b: &[u8]) -> Self {
        Self {
            enabled: b[0] != 0,
            x: decode_dpi(b[1], b[2]),
            y: decode_dpi(b[3], b[4]),
        }
    }
}

/// `índice = DPI / 100`. Até `0x79`: `índice - 1`, flag 0. Acima: `índice - 0x3D`, flag 1.
fn encode_dpi(dpi: u32) -> Result<(u8, u8), Error> {
    if !(DpiLevel::MIN..=DpiLevel::MAX).contains(&dpi) || !dpi.is_multiple_of(DpiLevel::STEP) {
        return Err(Error::Invalid(format!(
            "DPI {dpi} fora de 100..24000 em passos de 100"
        )));
    }
    let index = dpi / 100;
    Ok(if index <= 0x79 {
        ((index - 1) as u8, 0)
    } else {
        ((index - 0x3D) as u8, 1)
    })
}

fn decode_dpi(byte: u8, flag: u8) -> u32 {
    (byte as u32 + if flag == 0 { 1 } else { 0x3D }) * 100
}

/// Velocidade do efeito (respirar, piscar): 1 a 8 no firmware; o oficial usa
/// baixo, médio e alto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedMode {
    Off,
    Steady,
    Breathing {
        speed: u8,
    },
    Flash {
        speed: u8,
    },
    /// Modo `03 .. 03`, sem nome na interface oficial.
    Cycle,
}

impl LedMode {
    /// `modo velocidade sub` (`FUN_0040e9f0`).
    fn bytes(self) -> [u8; 3] {
        match self {
            Self::Off => [0x00, 0, 0],
            Self::Steady => [0x01, 0, 0],
            Self::Breathing { speed } => [0x02, speed.min(8), 0],
            Self::Flash { speed } => [0x04, speed.min(8), 0],
            Self::Cycle => [0x03, 0, 0x03],
        }
    }

    /// Mesma regra de `FUN_0040fa00`: o que não reconhece vira fixo.
    fn from_bytes(mode: u8, speed: u8, sub: u8) -> Self {
        match (mode, sub) {
            (0x02, 0) => Self::Breathing { speed },
            (0x04, 0) => Self::Flash { speed },
            (0x00, 0) => Self::Off,
            (0x03, 0x03) => Self::Cycle,
            _ => Self::Steady,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Led {
    pub rgb: [u8; 3],
    pub mode: LedMode,
    /// 1 baixo, 2 médio, 3 alto.
    pub brightness: u8,
}

impl Led {
    /// Os 7 bytes a partir de `0x449 + 8p`; o 7º é o brilho (`0x44F + 8p`).
    pub fn decode(b: &[u8]) -> Self {
        Self {
            rgb: [b[0], b[1], b[2]],
            mode: LedMode::from_bytes(b[3], b[4], b[5]),
            brightness: b[6],
        }
    }

    pub fn color_and_mode(&self) -> [u8; 6] {
        let m = self.mode.bytes();
        [self.rgb[0], self.rgb[1], self.rgb[2], m[0], m[1], m[2]]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// `None` quando o mouse tem um código que não conhecemos.
    pub polling_rate: Option<PollingRate>,
    /// Linhas por passo da roda (`WheelScrollLines`).
    pub scroll_speed: u8,
    pub active_dpi_level: usize,
    pub dpi_levels: [DpiLevel; 5],
    pub led: Led,
    pub buttons: Vec<ButtonAction>,
}

impl Profile {
    /// Regra do oficial: "at least one click function to be retained".
    pub fn validate(&self) -> Result<(), Error> {
        if !self.buttons.contains(&ButtonAction::LeftClick) {
            return Err(Error::Invalid(
                "pelo menos um botão precisa ficar com o clique esquerdo".into(),
            ));
        }
        if !(1..=3).contains(&self.led.brightness) {
            return Err(Error::Invalid("brilho deve ser 1, 2 ou 3".into()));
        }
        if self.active_dpi_level >= self.dpi_levels.len() {
            return Err(Error::Invalid("nível de DPI ativo deve ser 1 a 5".into()));
        }
        if !self.dpi_levels[self.active_dpi_level].enabled {
            return Err(Error::Invalid(
                "o nível de DPI ativo precisa estar ligado".into(),
            ));
        }
        if self.scroll_speed == 0 {
            return Err(Error::Invalid(
                "velocidade de rolagem deve ser pelo menos 1".into(),
            ));
        }
        for level in &self.dpi_levels {
            level.encode()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpi_follows_official_rule() {
        // Valores lidos do perfil 2 do mouse real.
        assert_eq!(DpiLevel::decode(&[1, 0x0B, 0, 0x0B, 0]).x, 1200);
        assert_eq!(DpiLevel::decode(&[1, 0x50, 1, 0x50, 1]).x, 14100);
        assert_eq!(DpiLevel::new(1000).encode().unwrap(), [1, 0x09, 0, 0x09, 0]);
        assert_eq!(
            DpiLevel::new(12100).encode().unwrap(),
            [1, 0x78, 0, 0x78, 0]
        );
        assert_eq!(
            DpiLevel::new(12200).encode().unwrap(),
            [1, 0x3D, 1, 0x3D, 1]
        );
        assert_eq!(
            DpiLevel::new(24000).encode().unwrap(),
            [1, 0xB3, 1, 0xB3, 1]
        );
        assert!(DpiLevel::new(150).encode().is_err());
        assert!(DpiLevel::new(24100).encode().is_err());
    }

    #[test]
    fn dpi_round_trip() {
        for dpi in (100..=24000).step_by(100) {
            assert_eq!(
                DpiLevel::decode(&DpiLevel::new(dpi).encode().unwrap()).x,
                dpi
            );
        }
    }

    #[test]
    fn led_of_real_dump() {
        let led = Led::decode(&[0xFF, 0, 0, 0x01, 0, 0, 0x02]);
        assert_eq!(led.mode, LedMode::Steady);
        assert_eq!(led.rgb, [0xFF, 0, 0]);
        assert_eq!(led.brightness, 2);
        let b = Led {
            rgb: [0, 0x80, 0xFF],
            mode: LedMode::Breathing { speed: 4 },
            brightness: 3,
        };
        assert_eq!(b.color_and_mode(), [0, 0x80, 0xFF, 0x02, 4, 0]);
    }

    #[test]
    fn polling_codes() {
        assert_eq!(PollingRate::from_code(2), Some(PollingRate::Hz500));
        assert_eq!(PollingRate::from_hertz(1000).unwrap().code(), 1);
        assert_eq!(PollingRate::from_code(3), None);
    }
}
