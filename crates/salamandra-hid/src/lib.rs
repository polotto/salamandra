//! Transporte USB HID por `hidapi`: Windows (`HidD_SetFeature`), Linux
//! (`hidraw`) e macOS (IOKit, aberto sem exclusividade).

use hidapi::{HidApi, HidDevice};
use salamandra_core::spec::{self, DeviceSpec, MODELS};
use salamandra_core::{Error, Mouse, Transport, USAGE_PAGE};

pub struct HidTransport {
    device: HidDevice,
}

impl HidTransport {
    /// Abre o primeiro mouse suportado conectado.
    pub fn open() -> Result<Self, Error> {
        Ok(Self::open_model(None)?.0)
    }

    /// Abre a interface de configuração (usage page `0xFFA0`) de um mouse da
    /// tabela de modelos. Com mais de um conectado, `prefer` (id do modelo)
    /// escolhe qual; sem ele, o erro lista os encontrados.
    pub fn open_model(prefer: Option<&str>) -> Result<(Self, &'static DeviceSpec), Error> {
        let api = HidApi::new().map_err(io)?;
        let mut found: Vec<(&'static DeviceSpec, &hidapi::DeviceInfo)> = Vec::new();
        for info in api.device_list() {
            if info.usage_page() != USAGE_PAGE {
                continue;
            }
            if let Some(model) = spec::by_product_id(info.vendor_id(), info.product_id()) {
                if !found.iter().any(|(m, _)| m.id == model.id) {
                    found.push((model, info));
                }
            }
        }
        let (model, info) =
            match (prefer, found.as_slice()) {
                (_, []) => return Err(Error::NotFound),
                (Some(id), list) => *list
                    .iter()
                    .find(|(m, _)| m.id.eq_ignore_ascii_case(id))
                    .ok_or_else(|| {
                        Error::Invalid(format!(
                            "modelo {id} não está conectado. Conectados: {}",
                            list.iter()
                                .map(|(m, _)| m.id)
                                .collect::<Vec<_>>()
                                .join(", ")
                        ))
                    })?,
                (None, [only]) => *only,
                (None, list) => {
                    return Err(Error::Invalid(format!(
                    "mais de um mouse conectado ({}). Escolha com --model <id> ou SALAMANDRA_MODEL",
                    list.iter().map(|(m, _)| m.id).collect::<Vec<_>>().join(", ")
                )))
                }
            };
        let device = info.open_device(&api).map_err(|e| {
            Error::Io(format!(
                "não deu para abrir o mouse ({e}).{}",
                if cfg!(target_os = "linux") {
                    " No Linux, instale a regra udev do projeto."
                } else {
                    ""
                }
            ))
        })?;
        Ok((Self { device }, model))
    }

    /// Espera até `timeout_ms` por um report de entrada da interface de
    /// configuração (eventos do mouse e atalhos). `None` se nada chegou.
    pub fn read_event(&self, timeout_ms: i32) -> Result<Option<Vec<u8>>, Error> {
        let mut buf = [0u8; 64];
        let n = self.device.read_timeout(&mut buf, timeout_ms).map_err(io)?;
        Ok((n > 0).then(|| buf[..n].to_vec()))
    }
}

impl Transport for HidTransport {
    fn set_feature(&mut self, report: &[u8]) -> Result<(), Error> {
        self.device.send_feature_report(report).map_err(io)
    }

    fn get_feature(&mut self, report_id: u8, size: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; size];
        buf[0] = report_id;
        let n = self.device.get_feature_report(&mut buf).map_err(io)?;
        buf.truncate(n);
        Ok(buf)
    }
}

pub fn open() -> Result<Mouse<HidTransport>, Error> {
    open_model(std::env::var("SALAMANDRA_MODEL").ok().as_deref())
}

pub fn open_model(prefer: Option<&str>) -> Result<Mouse<HidTransport>, Error> {
    let (transport, model) = HidTransport::open_model(prefer)?;
    Ok(Mouse::with_spec(transport, model))
}

/// Modelos que o Salamandra conhece, para listar na interface.
pub fn known_models() -> &'static [DeviceSpec] {
    MODELS
}

fn io(e: hidapi::HidError) -> Error {
    Error::Io(e.to_string())
}
