//! Protocolo de configuração do Redragon M901-1 Perdition 2 (`04d9:fc40`).
//! Sem I/O: o transporte entra pelo trait [`Transport`].

pub mod buttons;
pub mod defaults;
pub mod keys;
pub mod macros;
pub mod map;
pub mod model;
pub mod mouse;
pub mod packet;

pub use buttons::{ButtonAction, FireTarget, Modifiers, Shortcut, Special};
pub use macros::MacroEvent;
pub use model::{DpiLevel, Led, LedMode, PollingRate, Profile};
pub use mouse::{Mouse, Transport};

pub const VENDOR_ID: u16 = 0x04D9;
pub const PRODUCT_ID: u16 = 0xFC40;
/// Interface de configuração (vendor).
pub const USAGE_PAGE: u16 = 0xFFA0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Io(String),
    BadResponse(Vec<u8>),
    Invalid(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "mouse 04d9:fc40 não encontrado"),
            Self::Io(msg) => write!(f, "falha de comunicação: {msg}"),
            Self::BadResponse(r) => write!(f, "resposta inesperada: {}", packet::hex(r)),
            Self::Invalid(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}
