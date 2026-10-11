//! Transporte USB HID por `hidapi`: Windows (`HidD_SetFeature`), Linux
//! (`hidraw`) e macOS (IOKit, aberto sem exclusividade).

use hidapi::{HidApi, HidDevice};
use salamandra_core::{Error, Mouse, Transport, PRODUCT_ID, USAGE_PAGE, VENDOR_ID};

pub struct HidTransport {
    device: HidDevice,
}

impl HidTransport {
    /// Abre a interface de configuração (usage page `0xFFA0`) do primeiro
    /// M901-1 conectado.
    pub fn open() -> Result<Self, Error> {
        let api = HidApi::new().map_err(io)?;
        let info = api
            .device_list()
            .find(|d| {
                d.vendor_id() == VENDOR_ID
                    && d.product_id() == PRODUCT_ID
                    && d.usage_page() == USAGE_PAGE
            })
            .ok_or(Error::NotFound)?;
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
        Ok(Self { device })
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
    Ok(Mouse::new(HidTransport::open()?))
}

fn io(e: hidapi::HidError) -> Error {
    Error::Io(e.to_string())
}
