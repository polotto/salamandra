//! Serviço em segundo plano: escuta os avisos do mouse e executa no PC os
//! atalhos embutidos (copiar, colar, navegador...), como o `hid.exe` oficial.
//!
//! No macOS a simulação de teclas precisa da permissão de Acessibilidade.
//! No Linux funciona em X11; Wayland ainda não.

use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Duration;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use salamandra_core::custom::{self, CustomActions, Op};
use salamandra_core::events::{host_action, parse_event, DeviceEvent, HostAction, HostKey, Os};
use salamandra_core::{Error, Shortcut};
use salamandra_hid::HidTransport;

/// Ações customizadas em uso. O app troca ao gravar; o serviço lê a cada atalho.
static CUSTOM: RwLock<Option<CustomActions>> = RwLock::new(None);

/// Troca as ações customizadas (`None` volta aos atalhos embutidos). Recusa
/// ação que não roda neste sistema, sem mexer nas atuais.
pub fn set_custom_actions(actions: Option<CustomActions>) -> Result<(), Error> {
    if let Some(a) = &actions {
        a.validate(Os::current())?;
    }
    if let Ok(mut guard) = CUSTOM.write() {
        *guard = actions;
    }
    Ok(())
}

/// Lê e valida `custom-actions.json`. Arquivo ausente não é erro: `Ok(None)`.
pub fn load_custom_actions(path: &Path) -> Result<Option<CustomActions>, Error> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Error::Io(format!("{}: {e}", path.display()))),
    };
    let actions: CustomActions = serde_json::from_str(&text)
        .map_err(|e| Error::Invalid(format!("{}: {e}", path.display())))?;
    actions.validate(Os::current())?;
    Ok(Some(actions))
}

fn custom_for(shortcut: Shortcut) -> Option<Vec<Op>> {
    let guard = CUSTOM.read().ok()?;
    let action = guard.as_ref()?.bindings.get(&shortcut)?;
    custom::plan(action, Os::current()).ok()
}

pub struct Executor {
    enigo: Enigo,
    hyper_down: bool,
}

impl Executor {
    pub fn new() -> Result<Self, Error> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| Error::Io(format!("não deu para simular teclas: {e}")))?;
        Ok(Self {
            enigo,
            hyper_down: false,
        })
    }

    /// Atalho apertado: a ação customizada, se houver, senão a embutida.
    pub fn shortcut_pressed(&mut self, shortcut: Shortcut) -> Result<(), Error> {
        match custom_for(shortcut) {
            Some(ops) => ops.iter().try_for_each(|op| self.op(op)),
            None => self.run(&host_action(shortcut, Os::current())),
        }
    }

    /// Fim do aviso de atalho: solta o Hyper, se estava segurado.
    pub fn shortcut_released(&mut self) -> Result<(), Error> {
        self.op(&Op::HyperUp)
    }

    fn op(&mut self, op: &Op) -> Result<(), Error> {
        match op {
            Op::Keys { modifiers, key } => self.keys(modifiers, *key),
            Op::HyperDown => self.hyper(Direction::Press),
            Op::HyperUp => self.hyper(Direction::Release),
            Op::Open(target) => open(target),
            Op::Spawn(argv) => {
                let args: Vec<&str> = argv[1..].iter().map(String::as_str).collect();
                spawn(&argv[0], &args)
            }
            Op::Wait(ms) => {
                sleep(Duration::from_millis(u64::from(*ms)));
                Ok(())
            }
        }
    }

    fn hyper(&mut self, direction: Direction) -> Result<(), Error> {
        let press = direction == Direction::Press;
        let Some(order) = custom::hyper_transition(self.hyper_down, press) else {
            return Ok(());
        };
        self.hyper_down = press;
        for k in order.iter().filter_map(|k| to_enigo(*k)) {
            self.enigo
                .key(k, direction)
                .map_err(|e| Error::Io(format!("falha ao simular tecla: {e}")))?;
        }
        Ok(())
    }

    fn keys(&mut self, modifiers: &[HostKey], key: HostKey) -> Result<(), Error> {
        let fail = |e: enigo::InputError| Error::Io(format!("falha ao simular tecla: {e}"));
        // Com o Hyper segurado, os quatro modificadores já estão apertados.
        let mods: Vec<Key> = custom::modifiers_to_press(modifiers, self.hyper_down)
            .iter()
            .filter_map(|m| to_enigo(*m))
            .collect();
        let Some(key) = to_enigo(key) else {
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

    pub fn run(&mut self, action: &HostAction) -> Result<(), Error> {
        match action {
            HostAction::Keys { modifiers, key } => self.keys(modifiers, *key),
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
        // Sem cmd: ele quebra URL com & (a=1&b=2).
        Os::Windows => spawn("rundll32", &["url.dll,FileProtocolHandler", target]),
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
        HostKey::End => Key::End,
        HostKey::Up => Key::UpArrow,
        HostKey::Down => Key::DownArrow,
        HostKey::PageUp => Key::PageUp,
        HostKey::PageDown => Key::PageDown,
        HostKey::Enter => Key::Return,
        HostKey::Backspace => Key::Backspace,
        HostKey::Delete => Key::Delete,
        HostKey::F(1) => Key::F1,
        HostKey::F(2) => Key::F2,
        HostKey::F(3) => Key::F3,
        HostKey::F(4) => Key::F4,
        HostKey::F(5) => Key::F5,
        HostKey::F(6) => Key::F6,
        HostKey::F(7) => Key::F7,
        HostKey::F(8) => Key::F8,
        HostKey::F(9) => Key::F9,
        HostKey::F(10) => Key::F10,
        HostKey::F(11) => Key::F11,
        HostKey::F(12) => Key::F12,
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
                    if let Some(ex) = executor.as_mut() {
                        let result = match &event {
                            DeviceEvent::ShortcutPressed(s) => ex.shortcut_pressed(*s),
                            DeviceEvent::ShortcutReleased => ex.shortcut_released(),
                            _ => Ok(()),
                        };
                        if let Err(e) = result {
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
                    // Mouse sumiu com o Hyper apertado: não deixa as teclas presas.
                    if let Some(ex) = executor.as_mut() {
                        let _ = ex.shortcut_released();
                    }
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
    fn the_file_loads_validates_and_a_missing_one_is_not_an_error() {
        let dir = std::env::temp_dir().join(format!("salamandra-actions-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("custom-actions.json");
        assert_eq!(load_custom_actions(&path), Ok(None));

        let mut file = CustomActions::new();
        file.bindings.insert(
            Shortcut::ShowDesktop,
            custom::CustomAction::Keys {
                modifiers: vec![custom::Modifier::Cmd],
                key: "f".into(),
            },
        );
        std::fs::write(&path, serde_json::to_string(&file).unwrap()).unwrap();
        assert_eq!(load_custom_actions(&path), Ok(Some(file)));

        std::fs::write(&path, r#"{"format":"salamandra-actions","version":1,"bindings":{"Copy":{"type":"Keys","modifiers":[],"key":"banana"}}}"#).unwrap();
        assert!(load_custom_actions(&path).is_err());
        std::fs::write(&path, "não é json").unwrap();
        assert!(load_custom_actions(&path).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_custom_action_wins_over_the_builtin_one() {
        let mut file = CustomActions::new();
        file.bindings
            .insert(Shortcut::Mail, custom::CustomAction::Hyper);
        set_custom_actions(Some(file)).unwrap();
        assert_eq!(custom_for(Shortcut::Mail), Some(vec![Op::HyperDown]));
        assert_eq!(custom_for(Shortcut::Copy), None);
        set_custom_actions(None).unwrap();
        assert_eq!(custom_for(Shortcut::Mail), None);
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
