//! `Mouse` contra um transporte falso que simula a memória do mouse.

use std::time::Duration;

use salamandra_core::defaults::factory_profile;
use salamandra_core::macros::{MacroEvent, MouseButton};
use salamandra_core::map;
use salamandra_core::{
    ButtonAction, DpiLevel, Error, LedMode, Mouse, PollingRate, Special, Transport,
};

#[derive(Default)]
struct Fake {
    memory: Vec<u8>,
    sent: Vec<Vec<u8>>,
    pending: Option<Vec<u8>>,
}

impl Fake {
    fn new() -> Self {
        Self {
            memory: vec![0; 0x1100],
            ..Default::default()
        }
    }

    fn writes(&self) -> Vec<&Vec<u8>> {
        self.sent.iter().filter(|p| p[1] == 0xF3).collect()
    }

    fn sessions(&self) -> usize {
        self.sent.iter().filter(|p| p[1] == 0xF5).count() / 2
    }
}

impl Transport for Fake {
    fn set_feature(&mut self, r: &[u8]) -> Result<(), Error> {
        self.sent.push(r.to_vec());
        let address = r[2] as usize | (r[3] as usize) << 8;
        let len = r[4] as usize;
        match r[1] {
            0xF3 => self.memory[address..address + len].copy_from_slice(&r[8..8 + len]),
            0xF2 => {
                let mut resp = vec![0; r.len()];
                resp[0] = r[0];
                resp[1] = 0x08;
                resp[2] = r[2];
                resp[3] = 0x60;
                resp[4] = r[4];
                resp[6] = 0xFA;
                resp[7] = 0xFA;
                resp[8..8 + len].copy_from_slice(&self.memory[address..address + len]);
                self.pending = Some(resp);
            }
            _ => {}
        }
        Ok(())
    }

    fn get_feature(&mut self, _id: u8, _size: usize) -> Result<Vec<u8>, Error> {
        Ok(self.pending.take().unwrap_or_default())
    }
}

/// Mouse com o estado de fábrica gravado.
fn factory_mouse() -> Mouse<Fake> {
    let mut m = Mouse::new(Fake::new());
    m.session_delay = Duration::ZERO;
    for p in 0..map::PROFILE_COUNT {
        m.update_profile(p, &blank_profile(p), &factory_profile(p))
            .unwrap();
    }
    m.transport_mut().sent.clear();
    m
}

/// Perfil que difere da fábrica em tudo, para forçar a gravação de todos os campos.
fn blank_profile(p: usize) -> salamandra_core::Profile {
    let index = p;
    let mut p = factory_profile(0);
    p.polling_rate = None;
    p.scroll_speed = 99;
    p.active_dpi_level = (index + 1) % 5;
    p.dpi_levels = [DpiLevel {
        enabled: false,
        x: 0,
        y: 0,
    }; 5];
    p.led.brightness = 0;
    p.led.rgb = [1, 2, 3];
    p.buttons = vec![ButtonAction::Raw([0xEE, 0, 0, 0]); map::BUTTON_COUNT];
    p
}

fn nothing_written(m: &mut Mouse<Fake>) -> bool {
    let fake = m.transport_mut();
    fake.writes().is_empty() && fake.sessions() == 0
}

#[test]
fn reads_back_factory_state() {
    let mut m = factory_mouse();
    for p in 0..map::PROFILE_COUNT {
        assert_eq!(m.profile(p).unwrap(), factory_profile(p));
    }
}

#[test]
fn update_writes_only_changes_in_one_session() {
    let mut m = factory_mouse();
    let old = m.profile(1).unwrap();
    let mut new = old.clone();
    new.led.rgb[0] = 0x10;
    new.led.mode = LedMode::Breathing { speed: 3 };
    new.dpi_levels[2] = DpiLevel::new(800);
    new.buttons[7] = ButtonAction::Key(0x04);
    new.buttons[8] = ButtonAction::Special(Special::DpiCycle);
    new.polling_rate = Some(PollingRate::Hz1000);
    new.scroll_speed = 3;
    m.update_profile(1, &old, &new).unwrap();
    let fake = m.transport_mut();
    assert_eq!(fake.sessions(), 1);
    // LED (2) + polling (1) + rolagem (1) + DPI (1) + botões (2)
    assert_eq!(fake.writes().len(), 7);
    assert_eq!(m.profile(1).unwrap(), new);
    assert_eq!(m.profile(0).unwrap().polling_rate, Some(PollingRate::Hz500));
    assert_eq!(m.profile(0).unwrap().scroll_speed, 1);

    m.transport_mut().sent.clear();
    m.update_profile(1, &new, &new).unwrap();
    assert!(
        m.transport_mut().sent.is_empty(),
        "nada mudou, nada enviado"
    );
}

#[test]
fn refuses_profile_without_left_click() {
    let mut m = factory_mouse();
    let old = m.profile(0).unwrap();
    let mut new = old.clone();
    new.buttons[0] = ButtonAction::Key(0x04);
    assert!(m.update_profile(0, &old, &new).is_err());
    assert!(nothing_written(&mut m));
    // Clique esquerdo em outro botão é aceito.
    new.buttons[6] = ButtonAction::LeftClick;
    m.update_profile(0, &old, &new).unwrap();
}

#[test]
fn active_profile_repeats_the_session() {
    let mut m = factory_mouse();
    m.set_active_profile(2).unwrap();
    let cmds: Vec<Vec<u8>> = m
        .transport_mut()
        .sent
        .iter()
        .map(|p| p[..4].to_vec())
        .collect();
    assert_eq!(cmds.len(), 14);
    assert_eq!(cmds[..7], cmds[7..]);
    assert_eq!(
        cmds[..7],
        [
            vec![0x02, 0xF5, 0x00, 0x00],
            vec![0x02, 0xF3, 0x2C, 0x00],
            vec![0x02, 0xF1, 0x02, 0x04],
            vec![0x02, 0xF1, 0x02, 0x01],
            vec![0x02, 0xF1, 0x02, 0x02],
            vec![0x02, 0xF1, 0x02, 0x08],
            vec![0x02, 0xF5, 0x01, 0x00],
        ]
    );
    assert_eq!(m.active_profile().unwrap(), 2);
}

#[test]
fn macro_write_uses_report_4_without_apply() {
    let mut m = factory_mouse();
    let events = [
        MacroEvent::KeyDown(4),
        MacroEvent::Delay(50),
        MacroEvent::KeyUp(4),
        MacroEvent::MouseDown(MouseButton::Left),
    ];
    m.set_macro(2, &events).unwrap();
    let cmds: Vec<Vec<u8>> = m
        .transport_mut()
        .sent
        .iter()
        .map(|p| p[..5].to_vec())
        .collect();
    assert_eq!(
        cmds,
        [
            vec![0x02, 0xF5, 0, 0, 0],
            vec![0x04, 0xF3, 0x08, 0x06, 0xC8],
            vec![0x02, 0xF5, 1, 0, 0]
        ]
    );
    assert_eq!(m.transport_mut().sent[1].len(), 256);
    assert_eq!(m.macro_events(2).unwrap(), events);
}

#[test]
fn backup_restore_round_trip() {
    let mut m = factory_mouse();
    m.set_macro(0, &[MacroEvent::KeyDown(4), MacroEvent::KeyUp(4)])
        .unwrap();
    let saved = m.backup().unwrap();
    let before = m.transport_mut().memory.clone();
    let old = m.profile(3).unwrap();
    let mut new = old.clone();
    new.led.rgb = [9, 9, 9];
    new.buttons[3] = ButtonAction::Disabled;
    m.update_profile(3, &old, &new).unwrap();
    m.set_macro(0, &[]).unwrap();
    m.restore(&saved).unwrap();
    assert_eq!(m.transport_mut().memory, before);
}

#[test]
fn restore_rejects_incomplete_backup() {
    let mut m = factory_mouse();
    let mut partial = std::collections::BTreeMap::new();
    partial.insert("0x002c".to_string(), "00 00".to_string());
    assert!(m.restore(&partial).is_err());
    assert!(m.transport_mut().sent.is_empty());
}

#[test]
fn rejects_invalid_input() {
    let mut m = factory_mouse();
    assert!(m.set_active_profile(5).is_err());
    assert!(m.set_macro(15, &[]).is_err());
    let old = m.profile(0).unwrap();
    let mut bad = old.clone();
    bad.dpi_levels[0] = DpiLevel::new(25000);
    assert!(m.update_profile(0, &old, &bad).is_err());
    let mut bad = old.clone();
    bad.led.brightness = 9;
    assert!(m.update_profile(0, &old, &bad).is_err());
    let mut bad = old.clone();
    bad.dpi_levels[old.active_dpi_level].enabled = false;
    assert!(m.update_profile(0, &old, &bad).is_err());
    assert!(nothing_written(&mut m));
}
