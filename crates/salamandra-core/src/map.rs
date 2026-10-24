//! Endereços da memória de configuração do M901-1. Ver `docs/protocol.md`.

pub const PROFILE_COUNT: usize = 5;
pub const DPI_LEVEL_COUNT: usize = 5;
pub const BUTTON_COUNT: usize = 20;
pub const MACRO_COUNT: usize = 15;
pub const MACRO_SIZE: usize = 200;

/// Tabela em `0x47E35C` do software oficial.
pub const PROFILE_BASES: [u16; PROFILE_COUNT] = [0x040, 0x100, 0x1B0, 0x260, 0x310];

pub const ACTIVE_PROFILE: u16 = 0x002C;

pub fn scroll_speed(p: usize) -> u16 {
    0x0020 + 2 * p as u16
}

pub fn polling_rate(p: usize) -> u16 {
    0x0032 + 2 * p as u16
}

pub fn active_dpi_level(p: usize) -> u16 {
    PROFILE_BASES[p] + 2
}

pub fn dpi_level(p: usize, i: usize) -> u16 {
    PROFILE_BASES[p] + 4 + 6 * i as u16
}

/// Os slots `0x12` e `0x13` (roda) ficam em `0x16` e `0x17` (`FUN_00412980`).
pub fn button(p: usize, b: usize) -> u16 {
    let slot = match b {
        0x12 => 0x16,
        0x13 => 0x17,
        _ => b,
    } as u16;
    PROFILE_BASES[p] + 0x42 + 4 * slot
}

/// Dois bytes antes do LED na família 3 (`FUN_00456d90`): `00 00` no Impact.
pub const LED_GROUP: u16 = 0x0446;

pub fn led(p: usize) -> u16 {
    0x0449 + 8 * p as u16
}

pub fn led_brightness(p: usize) -> u16 {
    0x044F + 8 * p as u16
}

/// `FUN_004125b0`: `0x478 + 200 * índice`.
pub fn macro_slot(i: usize) -> u16 {
    0x0478 + (MACRO_SIZE * i) as u16
}

/// Blocos que `FUN_00412e90` grava, com o mesmo endereço e tamanho.
pub fn config_blocks() -> Vec<(u16, usize)> {
    let mut blocks = vec![(ACTIVE_PROFILE, 2), (0x0020, 10), (0x0032, 6), (0x0038, 4)];
    for p in 0..PROFILE_COUNT {
        blocks.push((active_dpi_level(p), 2));
        for i in 0..DPI_LEVEL_COUNT {
            blocks.push((dpi_level(p, i), 5));
        }
        for b in 0..BUTTON_COUNT {
            blocks.push((button(p, b), 4));
        }
        blocks.push((led(p), 6));
        blocks.push((led_brightness(p), 1));
    }
    blocks
}

pub fn macro_blocks() -> Vec<(u16, usize)> {
    (0..MACRO_COUNT)
        .map(|i| (macro_slot(i), MACRO_SIZE))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses() {
        assert_eq!(button(0, 0), 0x82);
        assert_eq!(button(0, 0x12), 0x40 + 0x42 + 0x16 * 4);
        assert_eq!(dpi_level(1, 2), 0x100 + 4 + 12);
        assert_eq!(led(4), 0x469);
        assert_eq!(macro_slot(1), 0x540);
        assert_eq!(macro_slot(14) as usize + MACRO_SIZE, 0x1030);
    }
}
