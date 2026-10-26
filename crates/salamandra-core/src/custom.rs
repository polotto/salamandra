//! Ações customizadas: o usuário troca o que cada atalho do mouse faz no PC.
//! O mouse só avisa o atalho (`events`); aqui está o contrato em JSON e o plano
//! puro (`plan`) que o serviço executa. Sem I/O.
//!
//! Inspirado nas ações do BetterTouchTool (Mission Control, Hyper Key, macro de
//! teclas); só os nomes e o comportamento, nenhum código ou dado dele.

use std::collections::BTreeMap;

use crate::buttons::Shortcut;
use crate::events::{HostKey, Os};
use crate::Error;

pub const FORMAT: &str = "salamandra-actions";
pub const VERSION: u32 = 1;
const MAX_STEPS: usize = 64;
/// Soma das pausas de uma sequência: o serviço espera na mesma thread que escuta o mouse.
const MAX_WAIT_MS: u32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Modifier {
    Ctrl,
    Shift,
    Alt,
    /// Windows, Cmd ou Super.
    Cmd,
}

/// Ação pronta, que cada sistema resolve do seu jeito.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Builtin {
    MissionControl,
    /// Só macOS.
    Launchpad,
    ShowDesktop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum CustomAction {
    /// Segura os modificadores, toca a tecla e solta (`key`: ver [`parse_key`]).
    Keys {
        modifiers: Vec<Modifier>,
        key: String,
    },
    Builtin {
        name: Builtin,
    },
    /// Segura Ctrl+Alt+Shift+Cmd enquanto o botão está apertado.
    Hyper,
    OpenApp {
        name: String,
    },
    OpenUrl {
        url: String,
    },
    /// Programa e argumentos, sem shell.
    Command {
        argv: Vec<String>,
    },
    Wait {
        ms: u32,
    },
    /// Passos em ordem. Não aninha e não leva `Hyper`.
    Sequence {
        steps: Vec<CustomAction>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomActions {
    pub format: String,
    pub version: u32,
    pub bindings: BTreeMap<Shortcut, CustomAction>,
}

impl CustomActions {
    pub fn new() -> Self {
        Self {
            format: FORMAT.into(),
            version: VERSION,
            bindings: BTreeMap::new(),
        }
    }

    /// Confere formato, versão e cada ação (inclusive se roda neste sistema).
    pub fn validate(&self, os: Os) -> Result<(), Error> {
        if self.format != FORMAT || self.version != VERSION {
            return Err(Error::Invalid(format!(
                "arquivo de ações inválido: esperado {FORMAT} v{VERSION}"
            )));
        }
        for (shortcut, action) in &self.bindings {
            plan(action, os).map_err(|e| Error::Invalid(format!("{shortcut:?}: {e}")))?;
        }
        Ok(())
    }
}

/// Uma operação concreta para o serviço executar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Keys {
        modifiers: Vec<HostKey>,
        key: HostKey,
    },
    HyperDown,
    HyperUp,
    Open(String),
    Spawn(Vec<String>),
    Wait(u32),
}

/// Teclas do Hyper, na ordem em que são apertadas.
pub const HYPER: [HostKey; 4] = [HostKey::Ctrl, HostKey::Alt, HostKey::Shift, HostKey::Meta];

/// Teclas a apertar (ou soltar) para o Hyper ir para `press`, ou `None` se já está assim.
/// Solta na ordem inversa da que apertou.
pub fn hyper_transition(down: bool, press: bool) -> Option<Vec<HostKey>> {
    if down == press {
        return None;
    }
    let mut keys = HYPER.to_vec();
    if !press {
        keys.reverse();
    }
    Some(keys)
}

/// Modificadores que ainda precisam ser apertados: com o Hyper segurado, os
/// quatro já estão apertados (soltá-los no fim da ação quebraria o Hyper).
pub fn modifiers_to_press(modifiers: &[HostKey], hyper_down: bool) -> Vec<HostKey> {
    modifiers
        .iter()
        .copied()
        .filter(|m| !(hyper_down && HYPER.contains(m)))
        .collect()
}

/// Nome de tecla do JSON: uma letra, dígito ou símbolo, `f1`..`f12` ou um nome.
pub fn parse_key(name: &str) -> Option<HostKey> {
    let lower = name.to_lowercase();
    let mut chars = lower.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        return Some(HostKey::Char(c));
    }
    if let Some(n) = lower.strip_prefix('f').and_then(|n| n.parse::<u8>().ok()) {
        return (1..=12).contains(&n).then_some(HostKey::F(n));
    }
    Some(match lower.as_str() {
        "space" => HostKey::Char(' '),
        "tab" => HostKey::Tab,
        "escape" | "esc" => HostKey::Escape,
        "enter" | "return" => HostKey::Enter,
        "backspace" => HostKey::Backspace,
        "delete" => HostKey::Delete,
        "left" => HostKey::Left,
        "right" => HostKey::Right,
        "up" => HostKey::Up,
        "down" => HostKey::Down,
        "home" => HostKey::Home,
        "end" => HostKey::End,
        "pageup" => HostKey::PageUp,
        "pagedown" => HostKey::PageDown,
        _ => return None,
    })
}

fn modifier_key(m: Modifier) -> HostKey {
    match m {
        Modifier::Ctrl => HostKey::Ctrl,
        Modifier::Shift => HostKey::Shift,
        Modifier::Alt => HostKey::Alt,
        Modifier::Cmd => HostKey::Meta,
    }
}

fn keys(modifiers: &[HostKey], key: HostKey) -> Op {
    Op::Keys {
        modifiers: modifiers.to_vec(),
        key,
    }
}

fn builtin(b: Builtin, os: Os) -> Result<Op, Error> {
    Ok(match (b, os) {
        (Builtin::MissionControl, Os::MacOs) => {
            Op::Spawn(vec!["open".into(), "-a".into(), "Mission Control".into()])
        }
        (Builtin::MissionControl, Os::Windows) => keys(&[HostKey::Meta], HostKey::Tab),
        // Super sozinho abre a visão geral no GNOME e no KDE.
        (Builtin::MissionControl, Os::Linux) => keys(&[], HostKey::Meta),
        (Builtin::Launchpad, Os::MacOs) => {
            Op::Spawn(vec!["open".into(), "-a".into(), "Launchpad".into()])
        }
        (Builtin::Launchpad, _) => {
            return Err(Error::Invalid("Launchpad só existe no macOS".into()))
        }
        (Builtin::ShowDesktop, Os::MacOs) => keys(&[], HostKey::F(11)),
        (Builtin::ShowDesktop, _) => keys(&[HostKey::Meta], HostKey::Char('d')),
    })
}

fn open_app(name: &str, os: Os) -> Op {
    match os {
        Os::MacOs => Op::Spawn(vec!["open".into(), "-a".into(), name.into()]),
        // Windows e Linux: o nome é o programa (no PATH ou caminho), sem shell.
        _ => Op::Spawn(vec![name.into()]),
    }
}

fn flatten(action: &CustomAction, os: Os, top: bool, out: &mut Vec<Op>) -> Result<(), Error> {
    match action {
        CustomAction::Keys { modifiers, key } => {
            let k = parse_key(key)
                .ok_or_else(|| Error::Invalid(format!("tecla desconhecida: {key:?}")))?;
            let mods: Vec<HostKey> = modifiers.iter().map(|m| modifier_key(*m)).collect();
            out.push(Op::Keys {
                modifiers: mods,
                key: k,
            });
        }
        CustomAction::Builtin { name } => out.push(builtin(*name, os)?),
        CustomAction::Hyper if top => {
            out.push(Op::HyperDown);
        }
        CustomAction::Hyper => {
            return Err(Error::Invalid(
                "Hyper só vale sozinho, não dentro de sequência".into(),
            ))
        }
        CustomAction::OpenApp { name } if name.trim().is_empty() => {
            return Err(Error::Invalid("nome do app vazio".into()))
        }
        CustomAction::OpenApp { name } => out.push(open_app(name, os)),
        CustomAction::OpenUrl { url } if url.trim().is_empty() => {
            return Err(Error::Invalid("URL vazia".into()))
        }
        CustomAction::OpenUrl { url } => out.push(Op::Open(url.clone())),
        CustomAction::Command { argv } if argv.first().is_none_or(|p| p.trim().is_empty()) => {
            return Err(Error::Invalid("comando vazio".into()))
        }
        CustomAction::Command { argv } => out.push(Op::Spawn(argv.clone())),
        CustomAction::Wait { .. } if top => {
            return Err(Error::Invalid(
                "pausa só vale dentro de uma sequência".into(),
            ))
        }
        CustomAction::Wait { ms } => out.push(Op::Wait(*ms)),
        CustomAction::Sequence { .. } if !top => {
            return Err(Error::Invalid("sequência não aninha".into()))
        }
        CustomAction::Sequence { steps } if steps.is_empty() || steps.len() > MAX_STEPS => {
            return Err(Error::Invalid(format!(
                "sequência de 1 a {MAX_STEPS} passos"
            )))
        }
        CustomAction::Sequence { steps } => {
            for s in steps {
                flatten(s, os, false, out)?;
            }
        }
    }
    Ok(())
}

/// Resolve uma ação nas operações deste sistema, ou diz por que não roda.
pub fn plan(action: &CustomAction, os: Os) -> Result<Vec<Op>, Error> {
    let mut out = Vec::new();
    flatten(action, os, true, &mut out)?;
    let waited: u64 = out
        .iter()
        .map(|op| {
            if let Op::Wait(ms) = op {
                u64::from(*ms)
            } else {
                0
            }
        })
        .sum();
    if waited > u64::from(MAX_WAIT_MS) {
        return Err(Error::Invalid(format!(
            "pausas somam {waited} ms, o limite é {MAX_WAIT_MS} ms"
        )));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys_action(mods: &[Modifier], key: &str) -> CustomAction {
        CustomAction::Keys {
            modifiers: mods.to_vec(),
            key: key.into(),
        }
    }

    #[test]
    fn key_names_cover_letters_function_keys_and_names() {
        assert_eq!(parse_key("A"), Some(HostKey::Char('a')));
        assert_eq!(parse_key("f12"), Some(HostKey::F(12)));
        assert_eq!(parse_key("f13"), None);
        assert_eq!(parse_key("Space"), Some(HostKey::Char(' ')));
        assert_eq!(parse_key("pagedown"), Some(HostKey::PageDown));
        assert_eq!(parse_key("banana"), None);
    }

    #[test]
    fn every_kind_of_action_plans_on_every_system() {
        let simple = [
            keys_action(&[Modifier::Ctrl], "c"),
            CustomAction::Builtin {
                name: Builtin::ShowDesktop,
            },
            CustomAction::OpenUrl {
                url: "https://x.com/?a=1&b=2".into(),
            },
            CustomAction::OpenApp {
                name: "Notes".into(),
            },
            CustomAction::Command {
                argv: vec!["echo".into(), "oi".into()],
            },
        ];
        for os in [Os::Windows, Os::MacOs, Os::Linux] {
            for a in &simple {
                assert_eq!(plan(a, os).unwrap().len(), 1, "{a:?} no {os:?}");
            }
        }
        assert_eq!(
            plan(&simple[3], Os::MacOs).unwrap(),
            vec![Op::Spawn(vec!["open".into(), "-a".into(), "Notes".into()])]
        );
        assert_eq!(
            plan(&simple[3], Os::Linux).unwrap(),
            vec![Op::Spawn(vec!["Notes".into()])]
        );
    }

    #[test]
    fn mission_control_follows_the_platform() {
        let a = CustomAction::Builtin {
            name: Builtin::MissionControl,
        };
        assert_eq!(
            plan(&a, Os::MacOs).unwrap(),
            vec![Op::Spawn(vec![
                "open".into(),
                "-a".into(),
                "Mission Control".into()
            ])]
        );
        assert_eq!(
            plan(&a, Os::Windows).unwrap(),
            vec![keys(&[HostKey::Meta], HostKey::Tab)]
        );
        assert!(plan(&a, Os::Linux).is_ok());
    }

    #[test]
    fn launchpad_only_runs_on_macos() {
        let a = CustomAction::Builtin {
            name: Builtin::Launchpad,
        };
        assert!(plan(&a, Os::MacOs).is_ok());
        assert!(plan(&a, Os::Windows).is_err());
    }

    #[test]
    fn hyper_holds_all_four_modifiers_and_stays_out_of_sequences() {
        assert_eq!(
            plan(&CustomAction::Hyper, Os::MacOs).unwrap(),
            vec![Op::HyperDown]
        );
        assert_eq!(HYPER.len(), 4);
        let nested = CustomAction::Sequence {
            steps: vec![CustomAction::Hyper],
        };
        assert!(plan(&nested, Os::MacOs).is_err());
    }

    #[test]
    fn hyper_state_changes_only_on_a_real_transition() {
        assert_eq!(hyper_transition(false, true), Some(HYPER.to_vec()));
        let up = hyper_transition(true, false).unwrap();
        assert_eq!(up.first(), Some(&HostKey::Meta));
        assert_eq!(up.last(), Some(&HostKey::Ctrl));
        assert_eq!(hyper_transition(true, true), None);
        assert_eq!(hyper_transition(false, false), None);
    }

    #[test]
    fn held_hyper_keeps_its_modifiers_out_of_the_press_list() {
        let mods = [HostKey::Shift, HostKey::Char('x')];
        assert_eq!(modifiers_to_press(&mods, true), vec![HostKey::Char('x')]);
        assert_eq!(modifiers_to_press(&mods, false), mods.to_vec());
    }

    #[test]
    fn huge_waits_do_not_wrap_around_the_limit() {
        let a = CustomAction::Sequence {
            steps: vec![
                CustomAction::Wait { ms: 2_147_483_648 },
                CustomAction::Wait { ms: 2_147_483_648 },
            ],
        };
        assert!(plan(&a, Os::MacOs).is_err());
    }

    #[test]
    fn a_sequence_runs_in_order_with_waits() {
        let a = CustomAction::Sequence {
            steps: vec![
                keys_action(&[Modifier::Cmd], "c"),
                CustomAction::Wait { ms: 100 },
                keys_action(&[Modifier::Cmd], "v"),
            ],
        };
        let ops = plan(&a, Os::MacOs).unwrap();
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[1], Op::Wait(100));
    }

    #[test]
    fn bad_actions_are_refused_with_a_reason() {
        let os = Os::MacOs;
        assert!(plan(&keys_action(&[], "banana"), os).is_err());
        assert!(plan(&CustomAction::Wait { ms: 100 }, os).is_err());
        let slow = CustomAction::Sequence {
            steps: vec![
                CustomAction::Wait { ms: 6_000 },
                CustomAction::Wait { ms: 6_000 },
            ],
        };
        assert!(plan(&slow, os).is_err());
        assert!(plan(&CustomAction::Command { argv: vec![] }, os).is_err());
        assert!(plan(&CustomAction::OpenUrl { url: " ".into() }, os).is_err());
        assert!(plan(&CustomAction::Sequence { steps: vec![] }, os).is_err());
        let nested = CustomAction::Sequence {
            steps: vec![CustomAction::Sequence {
                steps: vec![CustomAction::Wait { ms: 1 }],
            }],
        };
        assert!(plan(&nested, os).is_err());
    }

    #[test]
    fn validate_names_the_shortcut_that_is_wrong() {
        let mut file = CustomActions::new();
        file.bindings
            .insert(Shortcut::Copy, keys_action(&[], "banana"));
        let err = file.validate(Os::MacOs).unwrap_err().to_string();
        assert!(err.contains("Copy"), "{err}");
        let mut other = CustomActions::new();
        other.version = 9;
        assert!(other.validate(Os::MacOs).is_err());
    }
}
