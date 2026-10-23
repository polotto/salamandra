//! API de alto nível, genérica sobre o transporte. Lê e escreve só os blocos
//! que o software oficial usa, no mesmo envelope de comandos.

use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;

use crate::buttons::ButtonAction;
use crate::macros::{self, MacroEvent};
use crate::map;
use crate::model::{DpiLevel, Led, PollingRate, Profile};
use crate::packet::{self, ApplyArea};
use crate::spec::{self, DeviceSpec};
use crate::Error;

/// Canal de Feature Reports. O byte 0 do buffer é o report ID.
pub trait Transport {
    fn set_feature(&mut self, report: &[u8]) -> Result<(), Error>;
    fn get_feature(&mut self, report_id: u8, size: usize) -> Result<Vec<u8>, Error>;
}

/// `F1` enviados ao fim de uma sessão, como `FUN_00412e90`.
const ALL_AREAS: [ApplyArea; 4] = [
    ApplyArea::Buttons,
    ApplyArea::Profile,
    ApplyArea::Dpi,
    ApplyArea::PollingRate,
];

pub struct Mouse<T: Transport> {
    transport: T,
    spec: &'static DeviceSpec,
    /// Pausa depois de `F5`, como o oficial (`FUN_0040f1f0`, Sleep(100)).
    pub session_delay: Duration,
}

impl<T: Transport> Mouse<T> {
    /// Mouse com o modelo padrão (M901-1).
    pub fn new(transport: T) -> Self {
        Self::with_spec(transport, &spec::M901_1)
    }

    pub fn with_spec(transport: T, spec: &'static DeviceSpec) -> Self {
        Self {
            transport,
            spec,
            session_delay: Duration::from_millis(100),
        }
    }

    pub fn spec(&self) -> &'static DeviceSpec {
        self.spec
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    // Leitura

    pub fn read(&mut self, address: u16, len: usize) -> Result<Vec<u8>, Error> {
        let request = packet::read_request(address, len);
        self.transport.set_feature(&request)?;
        let response = self.transport.get_feature(request[0], request.len())?;
        packet::payload(&response, len)
    }

    pub fn active_profile(&mut self) -> Result<usize, Error> {
        Ok(self.read(map::ACTIVE_PROFILE, 1)?[0] as usize)
    }

    pub fn profile(&mut self, p: usize) -> Result<Profile, Error> {
        check_profile(p)?;
        let rate = self.read(map::polling_rate(p), 1)?[0];
        let scroll = self.read(map::scroll_speed(p), 1)?[0];
        // FUN_0040fc50: 0x20 bytes a partir de base+2 trazem o nível ativo e os 5 níveis.
        let dpi = self.read(map::active_dpi_level(p), 0x20)?;
        let codec = self.spec.dpi;
        let levels: [DpiLevel; 5] =
            std::array::from_fn(|i| DpiLevel::decode_with(&dpi[2 + 6 * i..2 + 6 * i + 5], codec));
        let led = Led::decode(&self.read(map::led(p), 7)?);
        // Botão que o modelo não tem fica como zeros e nunca é gravado.
        let mut buttons = vec![ButtonAction::decode(&[0; 4]); map::BUTTON_COUNT];
        for &b in self.spec.buttons {
            buttons[b] = ButtonAction::decode(&self.read(map::button(p, b), 4)?);
        }
        Ok(Profile {
            polling_rate: PollingRate::from_code(rate),
            scroll_speed: scroll,
            active_dpi_level: dpi[0] as usize,
            dpi_levels: levels,
            led,
            buttons,
        })
    }

    pub fn macro_events(&mut self, i: usize) -> Result<Vec<MacroEvent>, Error> {
        check_macro(i)?;
        Ok(macros::decode(
            &self.read(map::macro_slot(i), map::MACRO_SIZE)?,
        ))
    }

    // Escrita

    /// O firmware só acende o LED do perfil novo na sessão seguinte (testado
    /// no mouse em 11/10/2026), então a sessão vai duas vezes.
    pub fn set_active_profile(&mut self, p: usize) -> Result<(), Error> {
        check_profile(p)?;
        for _ in 0..2 {
            self.commit(&ALL_AREAS, |m| m.write(map::ACTIVE_PROFILE, &[p as u8, 0]))?;
        }
        Ok(())
    }

    /// Grava numa sessão só o que mudou de `old` para `new` no perfil `p`.
    pub fn update_profile(&mut self, p: usize, old: &Profile, new: &Profile) -> Result<(), Error> {
        check_profile(p)?;
        new.validate_for(self.spec)?;
        let mut writes: Vec<(u16, Vec<u8>)> = Vec::new();
        if new.led != old.led {
            writes.push((map::led(p), new.led.color_and_mode().to_vec()));
            writes.push((map::led_brightness(p), vec![new.led.brightness]));
        }
        if new.polling_rate != old.polling_rate {
            if let Some(rate) = new.polling_rate {
                // FUN_004103c0 grava em dois blocos: perfis 0..2 e 3..4.
                let (start, len) = if p < 3 {
                    (map::polling_rate(0), 6)
                } else {
                    (map::polling_rate(3), 4)
                };
                let mut block = self.read(start, len)?;
                block[(map::polling_rate(p) - start) as usize] = rate.code();
                writes.push((start, block));
            }
        }
        if new.scroll_speed != old.scroll_speed {
            // FUN_004101e0 grava os 5 perfis de uma vez (report 3, 10 bytes).
            let mut block = self.read(map::scroll_speed(0), 10)?;
            block[2 * p] = new.scroll_speed;
            writes.push((map::scroll_speed(0), block));
        }
        let mut dpi_changed = false;
        for i in 0..map::DPI_LEVEL_COUNT {
            if new.dpi_levels[i] != old.dpi_levels[i] {
                writes.push((
                    map::dpi_level(p, i),
                    new.dpi_levels[i].encode_with(self.spec.dpi)?.to_vec(),
                ));
                dpi_changed = true;
            }
        }
        if new.active_dpi_level != old.active_dpi_level {
            writes.push((
                map::active_dpi_level(p),
                vec![new.active_dpi_level as u8, 0],
            ));
            dpi_changed = true;
        }
        for &b in self.spec.buttons {
            if new.buttons[b] != old.buttons[b] {
                writes.push((map::button(p, b), new.buttons[b].encode().to_vec()));
            }
        }
        if writes.is_empty() {
            return Ok(());
        }
        self.commit(&ALL_AREAS, |m| {
            for (address, bytes) in &writes {
                m.write(*address, bytes)?;
            }
            if dpi_changed {
                m.transport
                    .set_feature(&packet::apply(ApplyArea::DpiLevel))?;
            }
            Ok(())
        })
    }

    /// `FUN_004125b0`: os 200 bytes entre `F5 00` e `F5 01`, sem `F1`.
    pub fn set_macro(&mut self, i: usize, events: &[MacroEvent]) -> Result<(), Error> {
        check_macro(i)?;
        let bytes = macros::encode(events)?;
        self.commit(&[], |m| m.write(map::macro_slot(i), &bytes))
    }

    // Backup

    /// Endereço em hex -> bytes em hex, para gravar em JSON. Inclui as macros.
    pub fn backup(&mut self) -> Result<BTreeMap<String, String>, Error> {
        let mut out = BTreeMap::new();
        for (address, len) in map::config_blocks().into_iter().chain(map::macro_blocks()) {
            out.insert(
                format!("0x{address:04x}"),
                packet::hex(&self.read(address, len)?),
            );
        }
        Ok(out)
    }

    /// Grava de volta um backup. Backup sem macros deixa as macros como estão.
    pub fn restore(&mut self, backup: &BTreeMap<String, String>) -> Result<(), Error> {
        let mut writes = Vec::new();
        for (address, len) in map::config_blocks() {
            let key = format!("0x{address:04x}");
            let hex = backup
                .get(&key)
                .ok_or_else(|| Error::Invalid(format!("backup sem o bloco {key}")))?;
            writes.push((address, parse_block(&key, hex, len)?));
        }
        for (address, len) in map::macro_blocks() {
            let key = format!("0x{address:04x}");
            if let Some(hex) = backup.get(&key) {
                writes.push((address, parse_block(&key, hex, len)?));
            }
        }
        // Os blocos de config_blocks() começam pelo perfil ativo.
        let active = writes[0].1.clone();
        self.commit(&ALL_AREAS, |m| {
            for (address, bytes) in &writes {
                m.write(*address, bytes)?;
            }
            m.transport.set_feature(&packet::apply(ApplyArea::DpiLevel))
        })?;
        // O LED só acompanha o perfil ativo na sessão seguinte (testado no mouse).
        self.commit(&ALL_AREAS, |m| m.write(map::ACTIVE_PROFILE, &active))
    }

    // Interno

    fn write(&mut self, address: u16, data: &[u8]) -> Result<(), Error> {
        self.transport.set_feature(&packet::write(address, data))
    }

    fn commit(
        &mut self,
        areas: &[ApplyArea],
        writes: impl FnOnce(&mut Self) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.transport.set_feature(&packet::session(true))?;
        self.pause();
        writes(self)?;
        for area in areas {
            self.transport.set_feature(&packet::apply(*area))?;
        }
        self.transport.set_feature(&packet::session(false))?;
        self.pause();
        Ok(())
    }

    fn pause(&self) {
        if !self.session_delay.is_zero() {
            sleep(self.session_delay);
        }
    }
}

fn parse_block(key: &str, hex: &str, len: usize) -> Result<Vec<u8>, Error> {
    let bytes: Vec<u8> = hex
        .split_whitespace()
        .filter_map(|h| u8::from_str_radix(h, 16).ok())
        .collect();
    if bytes.len() != len {
        return Err(Error::Invalid(format!("bloco {key} com tamanho errado")));
    }
    Ok(bytes)
}

fn check_profile(p: usize) -> Result<(), Error> {
    if p >= map::PROFILE_COUNT {
        return Err(Error::Invalid("perfil deve ser 1 a 5".into()));
    }
    Ok(())
}

fn check_macro(i: usize) -> Result<(), Error> {
    if i >= map::MACRO_COUNT {
        return Err(Error::Invalid(format!(
            "macro deve ser 1 a {}",
            map::MACRO_COUNT
        )));
    }
    Ok(())
}
