//! Serviço em segundo plano: escuta os avisos do mouse e executa no PC os
//! atalhos embutidos (copiar, colar, navegador...), como o `hid.exe` oficial.
//!
//! No macOS a simulação de teclas precisa da permissão de Acessibilidade.
//! No Linux funciona em X11; Wayland ainda não.

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use salamandra_core::events::{host_action, parse_event, DeviceEvent, HostAction, HostKey, Os};
use salamandra_core::Error;
use salamandra_hid::HidTransport;

pub struct Executor {
    enigo: Enigo,
}

impl Executor {
    pub fn new() -> Result<Self, Error> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| Error::Io(format!("não deu para simular teclas: {e}")))?;
        Ok(Self { enigo })
    }

    pub fn run(&mut self, action: &HostAction) -> Result<(), Error> {
        match action {
            HostAction::Keys { modifiers, key } => {
                let fail = |e: enigo::InputError| Error::Io(format!("falha ao simular tecla: {e}"));
                let mods: Vec<Key> = modifiers.iter().filter_map(|m| to_enigo(*m)).collect();
                let Some(key) = to_enigo(*key) else {
                    return Err(Error::Invalid(format!(
                        "tecla {key:?} não existe neste sistema"
                    )));
                };
                for m in &mods {
                    self.enigo.key(*m, Direction::Press).map_err(fail)?;
                    sleep(Duration::from_millis(20));
                }
                self.enigo.key(key, Direction::Click).map_err(fail)?;
                for m in mods.iter().rev() {
                    sleep(Duration::from_millis(20));
                    self.enigo.key(*m, Direction::Release).map_err(fail)?;
                }
                Ok(())
            }
            HostAction::OpenHome => {
                let home = std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
                    .unwrap_or_default();
                open(&home)
            }
            HostAction::OpenUrl(url) => open(url),
            HostAction::Command(argv) => spawn(argv[0], &argv[1..]),
        }
    }
}

fn open(target: &str) -> Result<(), Error> {
    match Os::current() {
        Os::MacOs => spawn("open", &[target]),
        Os::Linux => spawn("xdg-open", &[target]),
        Os::Windows => spawn("cmd", &["/C", "start", "", target]),
    }
}

fn spawn(program: &str, args: &[&str]) -> Result<(), Error> {
    Command::new(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|e| Error::Io(format!("{program}: {e}")))
}

fn to_enigo(k: HostKey) -> Option<Key> {
    Some(match k {
        HostKey::Ctrl => Key::Control,
        HostKey::Shift => Key::Shift,
        HostKey::Alt => Key::Alt,
        HostKey::Meta => Key::Meta,
        HostKey::Char(' ') => Key::Space,
        HostKey::Char(c) => Key::Unicode(c),
        HostKey::Tab => Key::Tab,
        HostKey::Escape => Key::Escape,
        HostKey::Left => Key::LeftArrow,
        HostKey::Right => Key::RightArrow,
        HostKey::Home => Key::Home,
        HostKey::F(2) => Key::F2,
        HostKey::F(4) => Key::F4,
        HostKey::F(5) => Key::F5,
        HostKey::F(11) => Key::F11,
        #[cfg(target_os = "windows")]
        HostKey::BrowserBack => Key::BrowserBack,
        #[cfg(target_os = "windows")]
        HostKey::BrowserForward => Key::BrowserForward,
        #[cfg(target_os = "windows")]
        HostKey::BrowserStop => Key::BrowserStop,
        #[cfg(target_os = "windows")]
        HostKey::BrowserRefresh => Key::BrowserRefresh,
        #[cfg(target_os = "windows")]
        HostKey::BrowserSearch => Key::BrowserSearch,
        #[cfg(target_os = "windows")]
        HostKey::BrowserFavorites => Key::BrowserFavorites,
        #[cfg(target_os = "windows")]
        HostKey::BrowserHome => Key::BrowserHome,
        #[cfg(target_os = "windows")]
        HostKey::LaunchMail => Key::LaunchMail,
        _ => return None,
    })
}

/// O usuário escolheu um mouse e o serviço está em outro (o escolhido estava fora).
fn wants_back(wanted: Option<&str>, opened: &str) -> bool {
    wanted.is_some_and(|w| !w.eq_ignore_ascii_case(opened))
}

/// Escuta o mouse até `stop` virar `true`. Executa os atalhos e repassa todo
/// aviso para `on_event` (a interface usa para mostrar troca de perfil e DPI).
/// Se o mouse sumir, espera ele voltar. `chosen` devolve o modelo que o usuário
/// escolheu (com mais de um mouse); quando muda, o serviço reabre no novo.
pub fn run(
    stop: &AtomicBool,
    chosen: impl Fn() -> Option<String>,
    mut on_event: impl FnMut(&DeviceEvent),
    mut on_error: impl FnMut(&Error),
) {
    let mut executor = match Executor::new() {
        Ok(e) => Some(e),
        Err(e) => {
            on_error(&e);
            None
        }
    };
    while !stop.load(Ordering::Relaxed) {
        let wanted = chosen();
        let choice = salamandra_hid::pick_model(wanted.as_deref());
        let device = match HidTransport::open_model(choice.as_deref()) {
            Ok(opened) => opened,
            Err(e) => {
                // "Modelo não conectado" com escolha feita é o mouse que saiu entre a
                // enumeração e a abertura: o laço tenta de novo.
                let vanished = wanted.is_some() && matches!(e, Error::Invalid(_));
                if e != Error::NotFound && !vanished {
                    on_error(&e);
                }
                sleep(Duration::from_secs(2));
                continue;
            }
        };
        let (device, opened) = device;
        let mut last_check = std::time::Instant::now();
        while !stop.load(Ordering::Relaxed) {
            // Caiu em outro mouse enquanto o escolhido estava fora: volta quando ele reaparece.
            if wants_back(wanted.as_deref(), opened.id)
                && last_check.elapsed() >= Duration::from_secs(2)
            {
                last_check = std::time::Instant::now();
                if salamandra_hid::pick_model(wanted.as_deref()) == wanted {
                    break;
                }
            }
            // Só compara a escolha do usuário (um lock); nada de listar o HID a cada leitura.
            if chosen() != wanted {
                break;
            }
            match device.read_event(250) {
                Ok(Some(report)) => {
                    let Some(event) = parse_event(&report) else {
                        continue;
                    };
                    if let (DeviceEvent::ShortcutPressed(s), Some(ex)) = (&event, executor.as_mut())
                    {
                        if let Err(e) = ex.run(&host_action(*s, Os::current())) {
                            on_error(&e);
                        }
                    }
                    // Evento de outro mouse (o escolhido está fora) não vai para a tela.
                    if !wants_back(wanted.as_deref(), opened.id) {
                        on_event(&event);
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    on_error(&e);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salamandra_core::Shortcut;

    #[test]
    fn the_service_goes_back_to_the_chosen_mouse_only_when_it_is_on_another() {
        assert!(wants_back(Some("m901-1"), "m711-cobra"));
        assert!(!wants_back(Some("m901-1"), "m901-1"));
        assert!(!wants_back(None, "m711-cobra"));
    }

    #[test]
    fn every_action_of_this_os_maps_to_real_keys() {
        for s in Shortcut::ALL {
            if let HostAction::Keys { modifiers, key } = host_action(s, Os::current()) {
                assert!(to_enigo(key).is_some(), "{s:?}: {key:?}");
                for m in modifiers {
                    assert!(to_enigo(m).is_some(), "{s:?}: {m:?}");
                }
            }
        }
    }
}
