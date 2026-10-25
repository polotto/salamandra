//! Mouse em memória: responde aos pacotes como o firmware, sem USB. Serve para
//! testar o CLI e o servidor MCP sem o mouse.

use crate::mouse::Transport;
use crate::packet::{self, CMD_READ, CMD_WRITE, DATA_OFFSET};
use crate::Error;

/// Cobre o último slot de macro (`0x478 + 15 * 200`) com folga.
const MEMORY_SIZE: usize = 0x1200;

pub struct MemoryTransport {
    memory: Vec<u8>,
    response: Vec<u8>,
    /// Todo pacote enviado, na ordem (para conferir a sessão `F5`/`F1`).
    pub sent: Vec<Vec<u8>>,
    /// Versão do dispositivo que o mouse falso informa (`None`: não informa).
    pub release: Option<u16>,
}

impl Default for MemoryTransport {
    fn default() -> Self {
        Self {
            memory: vec![0; MEMORY_SIZE],
            response: Vec::new(),
            sent: Vec::new(),
            release: None,
        }
    }
}

impl MemoryTransport {
    pub fn read_memory(&self, address: u16, len: usize) -> &[u8] {
        &self.memory[address as usize..address as usize + len]
    }
}

impl Transport for MemoryTransport {
    fn release(&self) -> Option<u16> {
        self.release
    }

    fn set_feature(&mut self, report: &[u8]) -> Result<(), Error> {
        self.sent.push(report.to_vec());
        let address = u16::from_le_bytes([report[2], report[3]]) as usize;
        let len = report[4] as usize;
        match report[1] {
            CMD_WRITE => {
                let end = address + len;
                if end > self.memory.len() {
                    return Err(Error::Io(format!("endereço fora da memória: 0x{end:04x}")));
                }
                self.memory[address..end].copy_from_slice(&report[DATA_OFFSET..DATA_OFFSET + len]);
            }
            CMD_READ => {
                let end = address + len;
                if end > self.memory.len() {
                    return Err(Error::Io(format!("endereço fora da memória: 0x{end:04x}")));
                }
                let mut r = vec![0u8; packet::report_size(report[0])];
                r[0] = report[0];
                r[1] = 0x08;
                r[2] = report[2];
                r[3] = 0x60;
                r[4] = report[4];
                r[6] = 0xFA;
                r[7] = 0xFA;
                r[DATA_OFFSET..DATA_OFFSET + len].copy_from_slice(&self.memory[address..end]);
                self.response = r;
            }
            _ => {}
        }
        Ok(())
    }

    fn get_feature(&mut self, _report_id: u8, size: usize) -> Result<Vec<u8>, Error> {
        let mut r = self.response.clone();
        r.resize(size.max(r.len()), 0);
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{map, Mouse};

    fn mouse() -> Mouse<MemoryTransport> {
        let mut m = Mouse::new(MemoryTransport::default());
        m.session_delay = std::time::Duration::ZERO;
        m
    }

    #[test]
    fn writes_are_read_back() {
        let mut m = mouse();
        m.set_active_profile(2).unwrap();
        assert_eq!(m.active_profile().unwrap(), 2);
    }

    #[test]
    fn writes_go_between_session_open_and_close() {
        let mut m = mouse();
        m.set_active_profile(1).unwrap();
        let sent = &m.transport_mut().sent;
        assert_eq!(sent.first().unwrap(), &packet::session(true));
        assert_eq!(sent.last().unwrap(), &packet::session(false));
    }

    #[test]
    fn macro_slot_fits() {
        let end = map::macro_slot(map::MACRO_COUNT - 1) as usize + map::MACRO_SIZE;
        assert!(end <= MEMORY_SIZE);
    }
}
