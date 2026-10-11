//! Pacote de configuração: `rid cmd addr_lo addr_hi len 0 0 0 dados`.
//! Ver `docs/protocol.md`.

use crate::Error;

pub const DATA_OFFSET: usize = 8;

pub const CMD_APPLY: u8 = 0xF1;
pub const CMD_READ: u8 = 0xF2;
pub const CMD_WRITE: u8 = 0xF3;
pub const CMD_SESSION: u8 = 0xF5;

/// Área que o mouse aplica depois de `F1 02 <área>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ApplyArea {
    Profile = 0x01,
    Dpi = 0x02,
    Buttons = 0x04,
    PollingRate = 0x08,
    DpiLevel = 0x10,
}

/// Do descritor HID: report 0x02 = 16 bytes, 0x03 = 64, 0x04 = 256.
pub fn report_id_for_len(len: usize) -> u8 {
    match len {
        0..=8 => 0x02,
        9..=56 => 0x03,
        _ => 0x04,
    }
}

pub fn report_size(id: u8) -> usize {
    match id {
        0x03 => 64,
        0x04 => 256,
        _ => 16,
    }
}

fn header(rid: u8, cmd: u8, address: u16, len: usize) -> Vec<u8> {
    let mut p = vec![0u8; report_size(rid)];
    p[0] = rid;
    p[1] = cmd;
    p[2] = (address & 0xFF) as u8;
    p[3] = (address >> 8) as u8;
    p[4] = len as u8;
    p
}

pub fn write(address: u16, data: &[u8]) -> Vec<u8> {
    let mut p = header(
        report_id_for_len(data.len()),
        CMD_WRITE,
        address,
        data.len(),
    );
    p[DATA_OFFSET..DATA_OFFSET + data.len()].copy_from_slice(data);
    p
}

pub fn read_request(address: u16, len: usize) -> Vec<u8> {
    header(report_id_for_len(len), CMD_READ, address, len)
}

pub fn apply(area: ApplyArea) -> Vec<u8> {
    let mut p = vec![0u8; 16];
    p[0] = 0x02;
    p[1] = CMD_APPLY;
    p[2] = 0x02;
    p[3] = area as u8;
    p
}

/// `F5 00` abre e `F5 01` fecha a sessão de escrita.
pub fn session(open: bool) -> Vec<u8> {
    let mut p = vec![0u8; 16];
    p[0] = 0x02;
    p[1] = CMD_SESSION;
    p[2] = if open { 0 } else { 1 };
    p
}

/// Dados da resposta `rid 08 addr_lo 60 len 00 FA FA dados`.
pub fn payload(response: &[u8], len: usize) -> Result<Vec<u8>, Error> {
    if response.len() < DATA_OFFSET + len || response[1] != 0x08 {
        return Err(Error::BadResponse(response.to_vec()));
    }
    Ok(response[DATA_OFFSET..DATA_OFFSET + len].to_vec())
}

pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_matches_official_led_packet() {
        // FUN_0040e9f0: 02 f3 49 04 06 00 00 00 R G B modo vel sub
        let p = write(0x0449, &[0xFF, 0, 0, 0x01, 0, 0]);
        assert_eq!(
            p,
            [0x02, 0xF3, 0x49, 0x04, 0x06, 0, 0, 0, 0xFF, 0, 0, 0x01, 0, 0, 0, 0]
        );
    }

    #[test]
    fn long_writes_use_bigger_reports() {
        let p = write(0x20, &[1; 10]);
        assert_eq!(p.len(), 64);
        assert_eq!(&p[..5], &[0x03, 0xF3, 0x20, 0x00, 0x0A]);
        assert_eq!(write(0x478, &[0; 200]).len(), 256);
    }

    #[test]
    fn session_and_apply() {
        assert_eq!(&session(true)[..3], &[0x02, 0xF5, 0x00]);
        assert_eq!(&session(false)[..3], &[0x02, 0xF5, 0x01]);
        assert_eq!(
            &apply(ApplyArea::PollingRate)[..4],
            &[0x02, 0xF1, 0x02, 0x08]
        );
    }

    #[test]
    fn payload_of_real_response() {
        // Resposta real do mouse ao ler 0x0032 com 5 bytes.
        let r = [
            0x02, 0x08, 0x32, 0x60, 0x05, 0x00, 0xFA, 0xFA, 0x02, 0, 0x02, 0, 0x02, 0, 0, 0,
        ];
        assert_eq!(payload(&r, 5).unwrap(), vec![2, 0, 2, 0, 2]);
        assert!(payload(&[0x02, 0x00], 1).is_err());
    }
}
