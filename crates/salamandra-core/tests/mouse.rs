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
fn restore_refreshes_the_led_with_a_second_session() {
    let mut m = factory_mouse();
    m.set_active_profile(3).unwrap();
    let saved = m.backup().unwrap();
    m.transport_mut().sent.clear();
    m.restore(&saved).unwrap();
    let fake = m.transport_mut();
    assert_eq!(fake.sessions(), 2);
    let last_write = (*fake.writes().last().unwrap()).clone();
    assert_eq!(&last_write[..5], &[0x02, 0xF3, 0x2C, 0x00, 0x02]);
    assert_eq!(last_write[8], 3);
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

fn cobra() -> Mouse<Fake> {
    let mut m = Mouse::with_spec(Fake::new(), &salamandra_core::spec::M711_COBRA);
    m.session_delay = Duration::ZERO;
    m
}

/// Os endereços e os bytes batem com a captura do M711 no mouse_m908 (DPI de
/// 800 em `0x44`, botões a partir de `0x82`, sem os slots 8 e 9).
#[test]
fn cobra_writes_dpi_and_buttons_like_the_capture() {
    let mut m = cobra();
    let old = blank_profile(0);
    let mut new = factory_profile(0);
    new.dpi_levels[0] = DpiLevel::new(800);
    new.dpi_levels[4] = DpiLevel::new(9000);
    m.update_profile(0, &old, &new).unwrap();
    let mem = &m.transport_mut().memory;
    assert_eq!(&mem[0x44..0x49], &[1, 0x12, 0, 0x12, 0]);
    assert_eq!(&mem[0x44 + 24..0x44 + 29], &[1, 0x68, 1, 0x68, 1]);
    assert_eq!(mem[0x82], 0x81);
    let written: Vec<u16> = m
        .transport_mut()
        .writes()
        .iter()
        .map(|p| p[2] as u16 | (p[3] as u16) << 8)
        .collect();
    for slot in [8usize, 9, 12, 19] {
        assert!(!written.contains(&map::button(0, slot)), "slot {slot}");
    }
    assert!(written.contains(&map::button(0, 11)));
}

#[test]
fn cobra_rejects_dpi_the_sensor_cannot_do() {
    let mut m = cobra();
    let old = blank_profile(0);
    let mut new = factory_profile(0);
    new.dpi_levels[1] = DpiLevel::new(5100);
    assert!(m.update_profile(0, &old, &new).is_err());
    new.dpi_levels[1] = DpiLevel::new(12000);
    assert!(m.update_profile(0, &old, &new).is_err());
    assert!(m.transport_mut().sent.is_empty());
}

#[test]
fn cobra_reads_back_what_it_wrote() {
    let mut m = cobra();
    let old = blank_profile(0);
    let mut new = factory_profile(0);
    new.dpi_levels = [800, 1600, 5000, 5200, 10000].map(DpiLevel::new);
    m.update_profile(0, &old, &new).unwrap();
    let back = m.profile(0).unwrap();
    assert_eq!(back.dpi_levels, new.dpi_levels);
    for &b in salamandra_core::spec::M711_COBRA.buttons {
        assert_eq!(back.buttons[b], new.buttons[b]);
    }
}

fn impact() -> Mouse<Fake> {
    let mut m = Mouse::with_spec(Fake::new(), &salamandra_core::spec::M908_IMPACT);
    m.session_delay = Duration::ZERO;
    m
}

/// Bytes do DPI e do LED como na captura do M908 no mouse_m908.
#[test]
fn impact_writes_dpi_and_led_like_the_capture() {
    let mut m = impact();
    let old = blank_profile(0);
    let mut new = factory_profile(0);
    new.dpi_levels = [800, 1000, 6400, 12400, 200].map(DpiLevel::new);
    new.led.rgb = [0xFF, 0, 0];
    new.led.mode = LedMode::Breathing { speed: 3 };
    new.led.brightness = 2;
    m.update_profile(0, &old, &new).unwrap();
    let mem = &m.transport_mut().memory;
    assert_eq!(&mem[0x44..0x49], &[1, 0x12, 0, 0x12, 0]);
    assert_eq!(&mem[0x44 + 6..0x44 + 11], &[1, 0x16, 0, 0x16, 0]);
    assert_eq!(&mem[0x44 + 12..0x44 + 17], &[1, 72, 1, 72, 1]);
    assert_eq!(&mem[0x44 + 18..0x44 + 23], &[1, 140, 1, 140, 1]);
    assert_eq!(&mem[0x449..0x44F], &[0xFF, 0, 0, 1, 3, 4]);
    assert_eq!(mem[0x44F], 2);
    assert_eq!(m.profile(0).unwrap().led, new.led);
    assert_eq!(m.profile(0).unwrap().dpi_levels, new.dpi_levels);
}

#[test]
fn impact_has_all_twenty_buttons_and_rejects_dpi_above_its_sensor() {
    let mut m = impact();
    let old = blank_profile(0);
    let mut new = factory_profile(0);
    m.update_profile(0, &old, &new).unwrap();
    let written: Vec<u16> = m
        .transport_mut()
        .writes()
        .iter()
        .map(|p| p[2] as u16 | (p[3] as u16) << 8)
        .collect();
    for b in 0..map::BUTTON_COUNT {
        assert!(written.contains(&map::button(0, b)), "botão {b}");
    }
    new.dpi_levels[0] = DpiLevel::new(12600);
    assert!(m.update_profile(0, &old, &new).is_err());
}

#[test]
fn invader_has_one_macro_and_the_cobra_dpi_scale() {
    let mut m = Mouse::with_spec(Fake::new(), &salamandra_core::spec::M719_INVADER);
    m.session_delay = Duration::ZERO;
    assert!(m.set_macro(0, &[]).is_ok());
    assert!(m.set_macro(1, &[]).is_err());
    assert!(m.macro_events(1).is_err());
    let old = blank_profile(0);
    let mut new =
        salamandra_core::defaults::factory_profile_for(&salamandra_core::spec::M719_INVADER, 0);
    new.dpi_levels[0] = DpiLevel::new(10000);
    m.update_profile(0, &old, &new).unwrap();
    assert_eq!(&m.transport_mut().memory[0x44..0x49], &[1, 115, 1, 115, 1]);
    let backup = m.backup().unwrap();
    assert!(backup.contains_key("0x0478"));
    assert!(!backup.contains_key("0x0540"));
}

/// Pares `(modo, sub)` das oito posições da família 3 em `FUN_00456d90`.
#[test]
fn impact_encodes_the_eight_official_led_modes() {
    let cases = [
        (LedMode::Off, [0, 0, 0]),
        (LedMode::Steady, [1, 0, 2]),
        (LedMode::Breathing { speed: 3 }, [1, 3, 4]),
        (LedMode::Rainbow { speed: 7 }, [1, 7, 8]),
        (LedMode::Wave { speed: 5 }, [2, 5, 0]),
        (LedMode::Alternating { speed: 6 }, [6, 6, 0]),
        (LedMode::Reactive { speed: 2 }, [7, 2, 0]),
        (LedMode::Flash { speed: 4 }, [1, 4, 0x10]),
    ];
    for (mode, bytes) in cases {
        let mut m = impact();
        let old = blank_profile(0);
        let mut new = factory_profile(0);
        new.led.rgb = [1, 2, 3];
        new.led.mode = mode;
        m.update_profile(0, &old, &new).unwrap();
        let mem = &m.transport_mut().memory;
        assert_eq!(
            &mem[0x449..0x44F],
            &[1, 2, 3, bytes[0], bytes[1], bytes[2]],
            "{mode:?}"
        );
        let got = m.profile(0).unwrap().led.mode;
        assert_eq!(got, mode);
    }
}

#[test]
fn impact_writes_zeros_at_0x446_before_the_led_and_the_cobra_does_not() {
    let mut m = impact();
    m.transport_mut().memory[0x446] = 9;
    m.transport_mut().memory[0x447] = 9;
    let old = blank_profile(0);
    let mut new = factory_profile(0);
    new.led.rgb = [5, 5, 5];
    m.update_profile(0, &old, &new).unwrap();
    assert_eq!(&m.transport_mut().memory[0x446..0x448], &[0, 0]);
    let addrs: Vec<u16> = m
        .transport_mut()
        .writes()
        .iter()
        .map(|p| p[2] as u16 | (p[3] as u16) << 8)
        .collect();
    let group = addrs.iter().position(|&a| a == 0x446).unwrap();
    let led = addrs.iter().position(|&a| a == 0x449).unwrap();
    assert!(group < led, "0x446 antes do LED: {addrs:x?}");

    let mut c = Mouse::with_spec(Fake::new(), &salamandra_core::spec::M711_COBRA);
    c.session_delay = Duration::ZERO;
    c.transport_mut().memory[0x446] = 9;
    let cobra_new =
        salamandra_core::defaults::factory_profile_for(&salamandra_core::spec::M711_COBRA, 0);
    c.update_profile(0, &blank_profile(0), &cobra_new).unwrap();
    assert_eq!(c.transport_mut().memory[0x446], 9);
}

#[test]
fn led_modes_of_family_3_are_rejected_by_other_models() {
    let mut c = Mouse::with_spec(Fake::new(), &salamandra_core::spec::M711_COBRA);
    c.session_delay = Duration::ZERO;
    let mut new = factory_profile(0);
    new.led.mode = LedMode::Rainbow { speed: 4 };
    assert!(c.update_profile(0, &blank_profile(0), &new).is_err());
}
