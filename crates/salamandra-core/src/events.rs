//! Avisos que o mouse envia ao PC pela interface de configuração
//! (`FUN_00410be0` do software oficial), e o que o PC faz com cada atalho.

use crate::buttons::Shortcut;
use crate::model::PollingRate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum DeviceEvent {
    /// Perfil ativo trocado por um botão do mouse (0 a 4).
    ProfileChanged(usize),
    /// Nível de DPI trocado por um botão do mouse (0 a 4).
    DpiLevelChanged(usize),
    PollingRateChanged(Option<PollingRate>),
    /// Modo do LED trocado por um botão do mouse (bytes crus do aviso).
    LedModeChanged([u8; 3]),
    /// Botão com atalho embutido apertado: o PC executa (`host_action`).
    ShortcutPressed(Shortcut),
    /// Fim do aviso de atalho (`01 00 00`).
    ShortcutReleased,
}

/// Lê um report de entrada. Conferido no mouse: `08 02 04` (DPI), `01 ff 12`
/// (copiar) e `01 00 00` (solto).
pub fn parse_event(r: &[u8]) -> Option<DeviceEvent> {
    match r {
        [0x08, 0x01, p, ..] => Some(DeviceEvent::ProfileChanged(*p as usize)),
        [0x08, 0x02, n, ..] => Some(DeviceEvent::DpiLevelChanged(*n as usize)),
        [0x08, 0x06, code, ..] => Some(DeviceEvent::PollingRateChanged(PollingRate::from_code(
            *code,
        ))),
        [0x08, 0x09, a, b, c, ..] => Some(DeviceEvent::LedModeChanged([*a, *b, *c])),
        [0x01, 0xFF, id, ..] => Shortcut::from_id(*id).map(DeviceEvent::ShortcutPressed),
        [0x01, 0x00, 0x00, ..] => Some(DeviceEvent::ShortcutReleased),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    MacOs,
    Linux,
}

impl Os {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Linux
        }
    }
}

/// Tecla para simular. `Browser*` e `LaunchMail` só existem no Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKey {
    Ctrl,
    Shift,
    Alt,
    /// Windows, Cmd ou Super.
    Meta,
    Char(char),
    Tab,
    Escape,
    Left,
    Right,
    Home,
    F(u8),
    BrowserBack,
    BrowserForward,
    BrowserStop,
    BrowserRefresh,
    BrowserSearch,
    BrowserFavorites,
    BrowserHome,
    LaunchMail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostAction {
    /// Segura os modificadores, toca a tecla e solta tudo.
    Keys {
        modifiers: Vec<HostKey>,
        key: HostKey,
    },
    /// Abre a pasta pessoal no gerenciador de arquivos.
    OpenHome,
    /// Abre uma URL com o programa padrão (`mailto:`).
    OpenUrl(&'static str),
    /// Roda um comando.
    Command(&'static [&'static str]),
}

fn keys(modifiers: &[HostKey], key: HostKey) -> HostAction {
    HostAction::Keys {
        modifiers: modifiers.to_vec(),
        key,
    }
}

/// O que o PC faz quando o mouse avisa um atalho. No Windows, o mesmo que o
/// software oficial; no macOS e no Linux, o equivalente de cada sistema.
pub fn host_action(s: Shortcut, os: Os) -> HostAction {
    use HostKey::*;
    use Shortcut as S;
    let primary = if os == Os::MacOs { Meta } else { Ctrl };
    let letter = |c| keys(&[primary], Char(c));
    match (s, os) {
        (S::Cut, _) => letter('x'),
        (S::Copy, _) => letter('c'),
        (S::Paste, _) => letter('v'),
        (S::SelectAll, _) => letter('a'),
        (S::Find, _) => letter('f'),
        (S::New, _) => letter('n'),
        (S::Print, _) => letter('p'),
        (S::Save, _) => letter('s'),
        (S::SwitchWindow, Os::MacOs) => keys(&[Meta], Tab),
        (S::SwitchWindow, _) => keys(&[Alt], Tab),
        (S::CloseWindow, Os::MacOs) => keys(&[Meta], Char('w')),
        (S::CloseWindow, _) => keys(&[Alt], F(4)),
        (S::OpenFileManager, Os::Windows) => keys(&[Meta], Char('e')),
        (S::OpenFileManager, _) => HostAction::OpenHome,
        (S::Run, Os::Windows) => keys(&[Meta], Char('r')),
        (S::Run, Os::MacOs) => keys(&[Meta], Char(' ')),
        (S::Run, Os::Linux) => keys(&[Alt], F(2)),
        (S::ShowDesktop, Os::MacOs) => keys(&[], F(11)),
        (S::ShowDesktop, _) => keys(&[Meta], Char('d')),
        (S::Lock, Os::Windows) => keys(&[Meta], Char('l')),
        (S::Lock, Os::MacOs) => keys(&[Ctrl, Meta], Char('q')),
        (S::Lock, Os::Linux) => HostAction::Command(&["loginctl", "lock-session"]),
        (S::BrowserHome, Os::Windows) => keys(&[], BrowserHome),
        (S::BrowserHome, Os::MacOs) => keys(&[Meta, Shift], Char('h')),
        (S::BrowserHome, Os::Linux) => keys(&[Alt], Home),
        (S::BrowserBack, Os::Windows) => keys(&[], BrowserBack),
        (S::BrowserBack, Os::MacOs) => keys(&[Meta], Char('[')),
        (S::BrowserBack, Os::Linux) => keys(&[Alt], Left),
        (S::BrowserForward, Os::Windows) => keys(&[], BrowserForward),
        (S::BrowserForward, Os::MacOs) => keys(&[Meta], Char(']')),
        (S::BrowserForward, Os::Linux) => keys(&[Alt], Right),
        (S::BrowserStop, Os::Windows) => keys(&[], BrowserStop),
        (S::BrowserStop, Os::MacOs) => keys(&[Meta], Char('.')),
        (S::BrowserStop, Os::Linux) => keys(&[], Escape),
        (S::BrowserRefresh, Os::Windows) => keys(&[], BrowserRefresh),
        (S::BrowserRefresh, Os::MacOs) => keys(&[Meta], Char('r')),
        (S::BrowserRefresh, Os::Linux) => keys(&[], F(5)),
        (S::BrowserSearch, Os::Windows) => keys(&[], BrowserSearch),
        (S::BrowserSearch, _) => letter('l'),
        (S::BrowserFavorites, Os::Windows) => keys(&[], BrowserFavorites),
        (S::BrowserFavorites, Os::MacOs) => keys(&[Meta, Alt], Char('b')),
        (S::BrowserFavorites, Os::Linux) => keys(&[Ctrl, Shift], Char('o')),
        (S::Mail, Os::Windows) => keys(&[], LaunchMail),
        (S::Mail, _) => HostAction::OpenUrl("mailto:"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_reports() {
        assert_eq!(
            parse_event(&[0x08, 0x02, 0x04, 0, 0, 0, 0, 0]),
            Some(DeviceEvent::DpiLevelChanged(4))
        );
        assert_eq!(
            parse_event(&[0x01, 0xFF, 0x12, 0, 0, 0, 0, 0]),
            Some(DeviceEvent::ShortcutPressed(Shortcut::Copy))
        );
        assert_eq!(
            parse_event(&[0x01, 0, 0, 0, 0, 0, 0, 0]),
            Some(DeviceEvent::ShortcutReleased)
        );
        assert_eq!(
            parse_event(&[0x08, 0x06, 0x01]),
            Some(DeviceEvent::PollingRateChanged(Some(PollingRate::Hz1000)))
        );
        assert_eq!(
            parse_event(&[0x08, 0x01, 0x02]),
            Some(DeviceEvent::ProfileChanged(2))
        );
        assert_eq!(parse_event(&[0x01, 0xFF, 0x99]), None);
        assert_eq!(parse_event(&[]), None);
    }

    #[test]
    fn every_shortcut_has_an_action_on_every_os() {
        for os in [Os::Windows, Os::MacOs, Os::Linux] {
            for s in Shortcut::ALL {
                let a = host_action(s, os);
                if os != Os::Windows {
                    let windows_only = [
                        HostKey::BrowserBack,
                        HostKey::BrowserForward,
                        HostKey::BrowserStop,
                        HostKey::BrowserRefresh,
                        HostKey::BrowserSearch,
                        HostKey::BrowserFavorites,
                        HostKey::BrowserHome,
                        HostKey::LaunchMail,
                    ];
                    if let HostAction::Keys { key, .. } = a {
                        assert!(
                            !windows_only.contains(&key),
                            "{s:?} no {os:?} usa tecla só do Windows"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn copy_follows_the_platform() {
        assert_eq!(
            host_action(Shortcut::Copy, Os::Windows),
            keys(&[HostKey::Ctrl], HostKey::Char('c'))
        );
        assert_eq!(
            host_action(Shortcut::Copy, Os::MacOs),
            keys(&[HostKey::Meta], HostKey::Char('c'))
        );
        assert_eq!(
            host_action(Shortcut::CloseWindow, Os::Windows),
            keys(&[HostKey::Alt], HostKey::F(4))
        );
    }
}
