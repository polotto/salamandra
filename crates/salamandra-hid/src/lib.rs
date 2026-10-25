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
        let found = enumerate(&api);
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

/// Mouses suportados conectados, um por modelo, na ordem em que o sistema os lista.
fn enumerate(api: &HidApi) -> Vec<(&'static DeviceSpec, &hidapi::DeviceInfo)> {
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
    found
}

/// Modelos suportados que estão conectados agora.
pub fn connected_models() -> Result<Vec<&'static DeviceSpec>, Error> {
    let api = HidApi::new().map_err(io)?;
    Ok(enumerate(&api).into_iter().map(|(m, _)| m).collect())
}

/// Qual modelo abrir quando o app pode ter mais de um mouse: o escolhido, se
/// ainda estiver conectado; senão `SALAMANDRA_MODEL`, se conectado; senão o primeiro, se
/// houver mais de um. `None` deixa a escolha para `open` (um mouse só).
pub fn pick_model(preferred: Option<&str>) -> Option<String> {
    let connected: Vec<&str> = connected_models()
        .map(|list| list.into_iter().map(|m| m.id).collect())
        .unwrap_or_default();
    choose_model(
        &connected,
        preferred,
        std::env::var("SALAMANDRA_MODEL").ok().as_deref(),
    )
}

fn choose_model(connected: &[&str], preferred: Option<&str>, env: Option<&str>) -> Option<String> {
    if let Some(p) = preferred.filter(|p| connected.iter().any(|c| c.eq_ignore_ascii_case(p))) {
        return Some(p.to_string());
    }
    if let Some(e) = env.filter(|e| connected.iter().any(|c| c.eq_ignore_ascii_case(e))) {
        return Some(e.to_string());
    }
    (connected.len() > 1).then(|| connected[0].to_string())
}

/// Modelos que o Salamandra conhece, para listar na interface.
pub fn known_models() -> &'static [DeviceSpec] {
    MODELS
}

fn io(e: hidapi::HidError) -> Error {
    Error::Io(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::choose_model;

    #[test]
    fn keeps_the_choice_while_it_is_connected() {
        let two = ["m901-1", "m711-cobra"];
        assert_eq!(
            choose_model(&two, Some("m711-cobra"), None).as_deref(),
            Some("m711-cobra")
        );
    }

    #[test]
    fn drops_a_choice_that_was_unplugged() {
        let two = ["m901-1", "m711-cobra"];
        assert_eq!(
            choose_model(&two, Some("m908-impact"), None).as_deref(),
            Some("m901-1")
        );
        assert_eq!(choose_model(&["m901-1"], Some("m908-impact"), None), None);
    }

    #[test]
    fn the_environment_wins_over_the_first_mouse_but_not_over_a_choice() {
        let two = ["m901-1", "m711-cobra"];
        assert_eq!(
            choose_model(&two, None, Some("m711-cobra")).as_deref(),
            Some("m711-cobra")
        );
        assert_eq!(
            choose_model(&two, Some("m901-1"), Some("m711-cobra")).as_deref(),
            Some("m901-1")
        );
    }

    #[test]
    fn an_environment_choice_that_is_unplugged_is_ignored() {
        let two = ["m901-1", "m711-cobra"];
        assert_eq!(
            choose_model(&two, None, Some("m908-impact")).as_deref(),
            Some("m901-1")
        );
    }

    #[test]
    fn one_mouse_needs_no_choice() {
        assert_eq!(choose_model(&["m901-1"], None, None), None);
        assert_eq!(choose_model(&[], None, None), None);
    }
}
