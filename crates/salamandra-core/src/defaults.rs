//! Padrões de fábrica do M901-1. Fonte: perfis do instalador oficial
//! (`RSProfile1..5.pfd`), conferidos com a memória do mouse.

use crate::buttons::{ButtonAction, FireTarget, Special};
use crate::model::{DpiLevel, Led, LedMode, PollingRate, Profile};
use crate::spec::DeviceSpec;

/// Cor de cada perfil: vermelho, azul, verde, magenta, amarelo.
const COLORS: [[u8; 3]; 5] = [
    [0xFF, 0, 0],
    [0, 0, 0xFF],
    [0, 0xFF, 0],
    [0xFF, 0, 0xFF],
    [0xFF, 0xFF, 0],
];

/// Padrão de fábrica ajustado ao modelo: o DPI nunca passa do que o sensor faz.
pub fn factory_profile_for(spec: &DeviceSpec, p: usize) -> Profile {
    let mut profile = factory_profile(p);
    for level in &mut profile.dpi_levels {
        *level = DpiLevel::new(level.x.min(spec.dpi.max()));
    }
    profile
}

pub fn factory_profile(p: usize) -> Profile {
    let mut buttons = vec![
        ButtonAction::LeftClick,
        ButtonAction::RightClick,
        ButtonAction::MiddleClick,
        ButtonAction::Fire {
            target: FireTarget::Left,
            times: 3,
            delay_ms: 0,
        },
        ButtonAction::Special(Special::DpiUp),
        ButtonAction::Special(Special::DpiDown),
    ];
    // Laterais 1 a 10: teclas 1..0. Laterais 11 e 12: + e - do teclado numérico.
    buttons.extend((0x1E..=0x27).map(ButtonAction::Key));
    buttons.push(ButtonAction::Key(0x57));
    buttons.push(ButtonAction::Key(0x56));
    buttons.push(ButtonAction::Special(Special::ScrollUp));
    buttons.push(ButtonAction::Special(Special::ScrollDown));
    Profile {
        polling_rate: Some(PollingRate::Hz500),
        scroll_speed: 1,
        active_dpi_level: p,
        dpi_levels: [1000, 2000, 4000, 8000, 12000].map(DpiLevel::new),
        led: Led {
            rgb: COLORS[p],
            mode: LedMode::Steady,
            brightness: 2,
        },
        buttons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{BUTTON_COUNT, PROFILE_COUNT};

    #[test]
    fn factory_profiles_are_valid() {
        for p in 0..PROFILE_COUNT {
            let profile = factory_profile(p);
            assert_eq!(profile.buttons.len(), BUTTON_COUNT);
            profile.validate().unwrap();
        }
    }

    #[test]
    fn matches_real_dump_of_profile_1() {
        // Botões 7 e 17 lidos do mouse: `90 00 1e 00` e `90 00 57 00`.
        let p = factory_profile(0);
        assert_eq!(p.buttons[6].encode(), [0x90, 0, 0x1E, 0]);
        assert_eq!(p.buttons[16].encode(), [0x90, 0, 0x57, 0]);
        assert_eq!(p.buttons[3].encode(), [0x99, 0x81, 0x03, 0]);
    }
}
