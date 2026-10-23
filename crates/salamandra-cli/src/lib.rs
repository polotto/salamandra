//! Comandos do `salamandra`. Cada comando devolve o texto da resposta, para o
//! binário imprimir e para o servidor MCP reaproveitar.

use std::collections::BTreeMap;
use std::fmt::Write;

use salamandra_core::buttons::{FireTarget, Modifiers, Shortcut, Special};
use salamandra_core::defaults::factory_profile_for;
use salamandra_core::{
    keys, macros, map, ButtonAction, DpiLevel, Error, LedMode, Mouse, PollingRate, Transport,
};
use salamandra_hid::HidTransport;

pub const USAGE: &str = "\
uso: salamandra <comando>

  --model <id>                         escolhe o modelo quando há mais de um mouse (ou SALAMANDRA_MODEL)
  show [perfil]                        mostra a configuração (todos os perfis sem argumento)
  profile <1-5>                        troca o perfil ativo
  led <perfil> <RRGGBB> [modo] [vel]   modo: steady, breathing, flash, off; vel 1-8
      [--brightness 1-3]
  dpi <perfil> <nível 1-5> <dpi>[x<dpiY>] [--off]
  dpi-active <perfil> <nível 1-5>
  rate <perfil> <125|250|500|1000>
  scroll <perfil> <linhas>             velocidade de rolagem
  button <perfil> <botão 1-20> <ação>
  macro <1-15> [clear | set <evento>...]
  reset <perfil | all>                 volta para o padrão de fábrica
  backup <arquivo.json>                salva a configuração inteira, com as macros
  restore <arquivo.json>               grava de volta um backup
  events                               mostra os avisos que o mouse envia (Ctrl+C para sair)

botões: 1 esquerdo, 2 direito, 3 meio, 4 tiro rápido, 5 DPI+, 6 DPI-,
        7-18 laterais 1-12, 19 roda para cima, 20 roda para baixo
ações: left, right, middle, back, forward, disabled, key:<tecla>,
       combo:<ctrl+shift+alt+meta>:<tecla>, media:<play|stop|prev|next|volup|voldown|mute>,
       macro:<1-15>[:repeat|:<vezes>], fire:<left|right|middle|tecla>:<vezes>:<ms>,
       special:<dpi-cycle|dpi-up|dpi-down|scroll-up|scroll-down|profile|rate-up|rate-down|led-mode>,
       shortcut:<cut|copy|paste|select-all|find|new|print|save|switch-window|close-window|
                 files|run|desktop|lock|home|back|forward|stop|refresh|search|favorites|mail>
macro: down:<tecla> up:<tecla> tap:<tecla> mdown:<left|right|middle> mup:<...> click:<...> delay:<ms>
       ex.: salamandra macro 3 set down:lmeta tap:v up:lmeta
tecla: a-z, 0-9, f1-f24, enter, esc, space, tab, lmeta (cmd/win), lshift, lalt, lctrl, kp_plus...
";

const MEDIA: [(&str, u8); 7] = [
    ("play", 0xCD),
    ("stop", 0xB7),
    ("prev", 0xB6),
    ("next", 0xB5),
    ("volup", 0xE9),
    ("voldown", 0xEA),
    ("mute", 0xE2),
];

const SPECIAL: [(&str, Special); 9] = [
    ("dpi-cycle", Special::DpiCycle),
    ("dpi-up", Special::DpiUp),
    ("dpi-down", Special::DpiDown),
    ("scroll-up", Special::ScrollUp),
    ("scroll-down", Special::ScrollDown),
    ("profile", Special::ProfileSwitch),
    ("rate-up", Special::RateUp),
    ("rate-down", Special::RateDown),
    ("led-mode", Special::LedModeSwitch),
];

const SHORTCUTS: [(&str, Shortcut); 22] = [
    ("cut", Shortcut::Cut),
    ("copy", Shortcut::Copy),
    ("paste", Shortcut::Paste),
    ("select-all", Shortcut::SelectAll),
    ("find", Shortcut::Find),
    ("new", Shortcut::New),
    ("print", Shortcut::Print),
    ("save", Shortcut::Save),
    ("switch-window", Shortcut::SwitchWindow),
    ("close-window", Shortcut::CloseWindow),
    ("files", Shortcut::OpenFileManager),
    ("run", Shortcut::Run),
    ("desktop", Shortcut::ShowDesktop),
    ("lock", Shortcut::Lock),
    ("home", Shortcut::BrowserHome),
    ("back", Shortcut::BrowserBack),
    ("forward", Shortcut::BrowserForward),
    ("stop", Shortcut::BrowserStop),
    ("refresh", Shortcut::BrowserRefresh),
    ("search", Shortcut::BrowserSearch),
    ("favorites", Shortcut::BrowserFavorites),
    ("mail", Shortcut::Mail),
];

/// Escreve uma linha na saída do comando (escrever em `String` não falha).
macro_rules! say {
    ($out:expr, $($arg:tt)*) => {{
        let _ = writeln!($out, $($arg)*);
    }};
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

fn profile_arg(s: &str) -> Result<usize, Error> {
    match s.parse::<usize>() {
        Ok(n @ 1..=5) => Ok(n - 1),
        _ => Err(invalid(format!("perfil inválido: {s}"))),
    }
}

fn number<T: std::str::FromStr>(s: &str, what: &str) -> Result<T, Error> {
    s.parse()
        .map_err(|_| invalid(format!("{what} inválido: {s}")))
}

fn key_arg(s: &str) -> Result<u8, Error> {
    keys::usage(s).ok_or_else(|| invalid(format!("tecla desconhecida: {s}")))
}

fn lookup<T: Copy>(table: &[(&str, T)], name: &str, what: &str) -> Result<T, Error> {
    table
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| *v)
        .ok_or_else(|| invalid(format!("{what} inválido: {name}")))
}

pub fn parse_action(s: &str) -> Result<ButtonAction, Error> {
    let parts: Vec<&str> = s.split(':').collect();
    Ok(match parts.as_slice() {
        ["left"] => ButtonAction::LeftClick,
        ["right"] => ButtonAction::RightClick,
        ["middle"] => ButtonAction::MiddleClick,
        ["back"] => ButtonAction::Back,
        ["forward"] => ButtonAction::Forward,
        ["disabled"] => ButtonAction::Disabled,
        ["key", k] => ButtonAction::Key(key_arg(k)?),
        ["combo", mods, k] => {
            let mut m = 0;
            for name in mods.split('+') {
                m |= match name {
                    "ctrl" => Modifiers::CTRL,
                    "shift" => Modifiers::SHIFT,
                    "alt" => Modifiers::ALT,
                    "meta" | "cmd" | "win" => Modifiers::META,
                    _ => return Err(invalid(format!("modificador inválido: {name}"))),
                };
            }
            ButtonAction::Combo {
                modifiers: Modifiers(m),
                key: key_arg(k)?,
            }
        }
        ["media", m] => ButtonAction::Media(lookup(&MEDIA, m, "mídia")?),
        ["special", sp] => ButtonAction::Special(lookup(&SPECIAL, sp, "função")?),
        ["shortcut", sc] => ButtonAction::Shortcut(lookup(&SHORTCUTS, sc, "atalho")?),
        ["macro", n, rest @ ..] => {
            let n: usize = number(n, "macro")?;
            if !(1..=map::MACRO_COUNT).contains(&n) {
                return Err(invalid(format!("macro deve ser 1 a {}", map::MACRO_COUNT)));
            }
            let index = (n - 1) as u8;
            match rest {
                [] => ButtonAction::Macro {
                    index,
                    repeat_while_held: false,
                    count: 1,
                },
                ["repeat"] => ButtonAction::Macro {
                    index,
                    repeat_while_held: true,
                    count: 1,
                },
                [times] => ButtonAction::Macro {
                    index,
                    repeat_while_held: false,
                    count: number(times, "repetição")?,
                },
                _ => return Err(invalid(format!("ação inválida: {s}"))),
            }
        }
        ["fire", target, times, ms] => {
            let target = match *target {
                "left" => FireTarget::Left,
                "right" => FireTarget::Right,
                "middle" => FireTarget::Middle,
                k => FireTarget::Key(key_arg(k)?),
            };
            ButtonAction::Fire {
                target,
                times: number(times, "vezes")?,
                delay_ms: number(ms, "atraso")?,
            }
        }
        _ => return Err(invalid(format!("ação inválida: {s}"))),
    })
}

pub fn describe(a: &ButtonAction) -> String {
    match a {
        ButtonAction::Disabled => "desabilitado".into(),
        ButtonAction::LeftClick => "clique esquerdo".into(),
        ButtonAction::RightClick => "clique direito".into(),
        ButtonAction::MiddleClick => "clique do meio".into(),
        ButtonAction::Back => "voltar".into(),
        ButtonAction::Forward => "avançar".into(),
        ButtonAction::Key(k) => format!("tecla {}", keys::name(*k)),
        ButtonAction::Combo { modifiers, key } => {
            let mut parts: Vec<String> = [
                (Modifiers::CTRL, "ctrl"),
                (Modifiers::SHIFT, "shift"),
                (Modifiers::ALT, "alt"),
                (Modifiers::META, "meta"),
            ]
            .iter()
            .filter(|(m, _)| modifiers.has(*m))
            .map(|(_, n)| n.to_string())
            .collect();
            parts.push(keys::name(*key));
            format!("combo {}", parts.join("+"))
        }
        ButtonAction::Media(u) => {
            format!(
                "mídia {}",
                MEDIA
                    .iter()
                    .find(|(_, v)| v == u)
                    .map_or_else(|| format!("0x{u:02x}"), |(n, _)| n.to_string())
            )
        }
        ButtonAction::Fire {
            target,
            times,
            delay_ms,
        } => {
            let t = match target {
                FireTarget::Left => "esquerdo".to_string(),
                FireTarget::Right => "direito".to_string(),
                FireTarget::Middle => "meio".to_string(),
                FireTarget::Key(k) => format!("tecla {}", keys::name(*k)),
            };
            format!("tiro rápido {t}, {times} vezes, {delay_ms} ms")
        }
        ButtonAction::Macro {
            index,
            repeat_while_held,
            count,
        } => {
            if *repeat_while_held {
                format!("macro {} (repete enquanto segura)", index + 1)
            } else {
                format!("macro {}, {count}x", index + 1)
            }
        }
        ButtonAction::Special(s) => format!(
            "função {}",
            SPECIAL.iter().find(|(_, v)| v == s).map_or("?", |(n, _)| n)
        ),
        ButtonAction::Shortcut(s) => format!(
            "atalho {}",
            SHORTCUTS
                .iter()
                .find(|(_, v)| v == s)
                .map_or("?", |(n, _)| n)
        ),
        ButtonAction::Raw(b) => {
            format!("código {:02x} {:02x} {:02x} {:02x}", b[0], b[1], b[2], b[3])
        }
    }
}

fn describe_led(mode: LedMode) -> String {
    match mode {
        LedMode::Off => "desligado".into(),
        LedMode::Steady => "fixo".into(),
        LedMode::Breathing { speed } => format!("respirar (vel {speed})"),
        LedMode::Flash { speed } => format!("piscar (vel {speed})"),
        LedMode::Cycle => "ciclo".into(),
    }
}

fn show<T: Transport>(
    out: &mut String,
    mouse: &mut Mouse<T>,
    p: usize,
    active: usize,
) -> Result<(), Error> {
    let prof = mouse.profile(p)?;
    say!(
        out,
        "Perfil {}{}",
        p + 1,
        if p == active { " (ativo)" } else { "" }
    );
    say!(
        out,
        "  polling: {}",
        prof.polling_rate
            .map_or("?".into(), |r| format!("{} Hz", r.hertz()))
    );
    say!(out, "  rolagem: {} linha(s)", prof.scroll_speed);
    let [r, g, b] = prof.led.rgb;
    say!(
        out,
        "  LED: #{r:02X}{g:02X}{b:02X}, {}, brilho {}",
        describe_led(prof.led.mode),
        prof.led.brightness
    );
    for (i, l) in prof.dpi_levels.iter().enumerate() {
        let value = if l.x == l.y {
            l.x.to_string()
        } else {
            format!("{}x{}", l.x, l.y)
        };
        let mark = if i == prof.active_dpi_level { "*" } else { " " };
        say!(
            out,
            "  DPI {}{mark} {value}{}",
            i + 1,
            if l.enabled { "" } else { " (desligado)" }
        );
    }
    for &i in mouse.spec().buttons {
        say!(out, "  botão {:2}: {}", i + 1, describe(&prof.buttons[i]));
    }
    Ok(())
}

/// Lê o perfil, aplica `change` e grava só o que mudou.
fn edit<T: Transport>(
    mouse: &mut Mouse<T>,
    p: usize,
    change: impl FnOnce(&mut salamandra_core::Profile) -> Result<(), Error>,
) -> Result<(), Error> {
    let old = mouse.profile(p)?;
    let mut new = old.clone();
    change(&mut new)?;
    mouse.update_profile(p, &old, &new)
}

/// Fica escutando os avisos do mouse e imprime cada um. Só volta com erro.
pub fn watch_events() -> Result<(), Error> {
    let t = HidTransport::open()?;
    println!("escutando o mouse (Ctrl+C para sair)");
    loop {
        if let Some(r) = t.read_event(1000)? {
            println!("{}", salamandra_core::packet::hex(&r));
        }
    }
}

/// Abre o mouse e executa um comando, devolvendo o texto da resposta.
/// `events` não passa por aqui: é um fluxo sem fim (ver [`watch_events`]).
pub fn run(args: &[String]) -> Result<String, Error> {
    let env_model = std::env::var("SALAMANDRA_MODEL").ok();
    let (model, args) = match args {
        [flag, id, rest @ ..] if flag == "--model" => (Some(id.clone()), rest),
        _ => (None, args),
    };
    let Some(cmd) = args.first() else {
        return Ok(USAGE.to_string());
    };
    if cmd == "--help" || cmd == "-h" {
        return Ok(USAGE.to_string());
    }
    let a: Vec<&str> = args[1..].iter().map(String::as_str).collect();
    if cmd == "events" {
        return Err(invalid("events só funciona no terminal"));
    }
    let mut mouse = salamandra_hid::open_model(model.as_deref().or(env_model.as_deref()))?;
    execute(&mut mouse, cmd, &a)
}

/// Executa um comando num mouse já aberto (os testes usam um transporte falso).
pub fn execute<T: Transport>(mouse: &mut Mouse<T>, cmd: &str, a: &[&str]) -> Result<String, Error> {
    let writes = !matches!(cmd, "show" | "backup") && !(cmd == "macro" && a.len() == 1);
    let text = run_command(mouse, cmd, a)?;
    match mouse.spec().warning() {
        Some(w) if writes => Ok(format!("{w}\n{text}")),
        _ => Ok(text),
    }
}

fn run_command<T: Transport>(mouse: &mut Mouse<T>, cmd: &str, a: &[&str]) -> Result<String, Error> {
    let mut out = String::new();
    match (cmd, a) {
        ("show", rest) if rest.len() <= 1 => {
            let active = mouse.active_profile()?;
            let list: Vec<usize> = match rest.first() {
                Some(p) => vec![profile_arg(p)?],
                None => (0..map::PROFILE_COUNT).collect(),
            };
            let spec = mouse.spec();
            say!(
                out,
                "Modelo: {} ({:04x}:{:04x})",
                spec.name,
                spec.vendor_id,
                spec.product_id
            );
            if let Some(w) = spec.warning() {
                say!(out, "{w}");
            }
            for p in list {
                show(&mut out, mouse, p, active)?;
            }
        }
        ("profile", [p]) => mouse.set_active_profile(profile_arg(p)?)?,
        ("led", [p, rest @ ..]) if !rest.is_empty() => {
            let p = profile_arg(p)?;
            let mut rest = rest.to_vec();
            let mut brightness = None;
            if let Some(i) = rest.iter().position(|x| *x == "--brightness") {
                brightness = Some(number::<u8>(
                    rest.get(i + 1).copied().unwrap_or(""),
                    "brilho",
                )?);
                rest.drain(i..(i + 2).min(rest.len()));
            }
            let hex = rest[0].trim_start_matches('#');
            let v = u32::from_str_radix(hex, 16)
                .ok()
                .filter(|_| hex.len() == 6)
                .ok_or_else(|| invalid(format!("cor inválida: {}", rest[0])))?;
            let speed: u8 = rest.get(2).map_or(Ok(4), |s| number(s, "velocidade"))?;
            let mode = match rest.get(1).copied().unwrap_or("steady") {
                "steady" => LedMode::Steady,
                "breathing" => LedMode::Breathing { speed },
                "flash" => LedMode::Flash { speed },
                "off" => LedMode::Off,
                m => return Err(invalid(format!("modo inválido: {m}"))),
            };
            edit(mouse, p, |prof| {
                prof.led.rgb = [(v >> 16) as u8, (v >> 8) as u8, v as u8];
                prof.led.mode = mode;
                if let Some(b) = brightness {
                    prof.led.brightness = b;
                }
                Ok(())
            })?;
        }
        ("dpi", [p, level, value, rest @ ..]) if rest.len() <= 1 => {
            let p = profile_arg(p)?;
            let i: usize = number(level, "nível")?;
            if !(1..=5).contains(&i) {
                return Err(invalid("nível deve ser 1 a 5"));
            }
            let xy: Vec<u32> = value
                .split('x')
                .map(|v| number(v, "DPI"))
                .collect::<Result<_, _>>()?;
            let mut l = match xy.as_slice() {
                [x] => DpiLevel::new(*x),
                [x, y] => DpiLevel::xy(*x, *y),
                _ => return Err(invalid(format!("DPI inválido: {value}"))),
            };
            l.enabled = rest.first() != Some(&"--off");
            edit(mouse, p, |prof| {
                prof.dpi_levels[i - 1] = l;
                Ok(())
            })?;
        }
        ("dpi-active", [p, level]) => {
            let i: usize = number(level, "nível")?;
            if !(1..=5).contains(&i) {
                return Err(invalid("nível deve ser 1 a 5"));
            }
            edit(mouse, profile_arg(p)?, |prof| {
                prof.active_dpi_level = i - 1;
                Ok(())
            })?;
        }
        ("rate", [p, hz]) => {
            let rate = PollingRate::from_hertz(number(hz, "taxa")?)
                .ok_or_else(|| invalid(format!("taxa inválida: {hz}")))?;
            edit(mouse, profile_arg(p)?, |prof| {
                prof.polling_rate = Some(rate);
                Ok(())
            })?;
        }
        ("scroll", [p, lines]) => {
            let lines: u8 = number(lines, "linhas")?;
            edit(mouse, profile_arg(p)?, |prof| {
                prof.scroll_speed = lines;
                Ok(())
            })?;
        }
        ("button", [p, b, action]) => {
            let b: usize = number(b, "botão")?;
            if b == 0 || !mouse.spec().buttons.contains(&(b - 1)) {
                let list: Vec<String> = mouse
                    .spec()
                    .buttons
                    .iter()
                    .map(|i| (i + 1).to_string())
                    .collect();
                return Err(invalid(format!(
                    "este modelo não tem o botão {b}. Botões: {}",
                    list.join(", ")
                )));
            }
            let action = parse_action(action)?;
            edit(mouse, profile_arg(p)?, |prof| {
                prof.buttons[b - 1] = action;
                Ok(())
            })?;
        }
        ("macro", [n, rest @ ..]) => {
            let n: usize = number(n, "macro")?;
            let slots = mouse.spec().macro_slots;
            if !(1..=slots).contains(&n) {
                return Err(invalid(format!("macro deve ser 1 a {slots}")));
            }
            match rest {
                [] => {
                    let events = mouse.macro_events(n - 1)?;
                    if events.is_empty() {
                        say!(out, "macro {n}: vazia");
                    } else {
                        say!(out, "macro {n}: {}", macros::format(&events));
                    }
                }
                ["clear"] => mouse.set_macro(n - 1, &[])?,
                ["set", events @ ..] if !events.is_empty() => {
                    mouse.set_macro(n - 1, &macros::parse(&events.join(" "))?)?
                }
                _ => return Err(invalid("uso: macro <número> [clear | set <evento>...]")),
            }
        }
        ("reset", [which]) => {
            let list: Vec<usize> = if *which == "all" {
                (0..map::PROFILE_COUNT).collect()
            } else {
                vec![profile_arg(which)?]
            };
            for p in list {
                let old = mouse.profile(p)?;
                let spec = mouse.spec();
                mouse.update_profile(p, &old, &factory_profile_for(spec, p))?;
            }
        }
        ("backup", [file]) => {
            let json = serde_json::to_string_pretty(&mouse.backup()?)
                .map_err(|e| invalid(e.to_string()))?;
            std::fs::write(file, json).map_err(|e| Error::Io(e.to_string()))?;
            say!(out, "backup salvo em {file}");
        }
        ("restore", [file]) => {
            let text = std::fs::read_to_string(file).map_err(|e| Error::Io(e.to_string()))?;
            let json: BTreeMap<String, String> =
                serde_json::from_str(&text).map_err(|_| invalid("arquivo de backup inválido"))?;
            mouse.restore(&json)?;
            say!(out, "backup restaurado");
        }
        _ => return Err(invalid("comando inválido. Veja: salamandra --help")),
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_actions() {
        assert_eq!(parse_action("key:a").unwrap(), ButtonAction::Key(0x04));
        assert_eq!(
            parse_action("combo:ctrl+shift:f").unwrap(),
            ButtonAction::Combo {
                modifiers: Modifiers(Modifiers::CTRL | Modifiers::SHIFT),
                key: 0x09
            }
        );
        assert_eq!(
            parse_action("macro:3:repeat").unwrap(),
            ButtonAction::Macro {
                index: 2,
                repeat_while_held: true,
                count: 1
            }
        );
        assert_eq!(
            parse_action("macro:3:5").unwrap(),
            ButtonAction::Macro {
                index: 2,
                repeat_while_held: false,
                count: 5
            }
        );
        assert_eq!(
            parse_action("fire:a:5:50").unwrap(),
            ButtonAction::Fire {
                target: FireTarget::Key(0x04),
                times: 5,
                delay_ms: 50
            }
        );
        assert_eq!(
            parse_action("shortcut:copy").unwrap(),
            ButtonAction::Shortcut(Shortcut::Copy)
        );
        assert_eq!(
            parse_action("special:dpi-up").unwrap(),
            ButtonAction::Special(Special::DpiUp)
        );
        assert!(parse_action("macro:16").is_err());
        assert!(parse_action("nada").is_err());
    }

    #[test]
    fn every_shortcut_and_special_has_a_name() {
        assert_eq!(SHORTCUTS.len(), Shortcut::ALL.len());
        assert_eq!(SPECIAL.len(), Special::ALL.len());
    }

    fn fake() -> Mouse<salamandra_core::fake::MemoryTransport> {
        let mut m = Mouse::new(salamandra_core::fake::MemoryTransport::default());
        m.session_delay = std::time::Duration::ZERO;
        // Memória zerada não tem clique esquerdo e o Salamandra se recusa a gravar assim.
        execute(&mut m, "reset", &["all"]).unwrap();
        m.transport_mut().sent.clear();
        m
    }

    fn cobra() -> Mouse<salamandra_core::fake::MemoryTransport> {
        let mut m = Mouse::with_spec(
            salamandra_core::fake::MemoryTransport::default(),
            &salamandra_core::spec::M711_COBRA,
        );
        m.session_delay = std::time::Duration::ZERO;
        execute(&mut m, "reset", &["all"]).unwrap();
        m.transport_mut().sent.clear();
        m
    }

    #[test]
    fn untested_model_warns_on_show_and_on_writes_only() {
        let mut m = cobra();
        let shown = cmd(&mut m, "show 1");
        assert!(shown.contains("Modelo: Redragon M711 Cobra (04d9:fc30)"));
        assert!(shown.contains("AVISO"));
        assert!(!shown.contains("botão  9"));
        assert!(shown.contains("botão 12"));
        let written = cmd(&mut m, "dpi 1 1 5200");
        assert!(written.starts_with("AVISO"));
        assert!(cmd(&mut m, "show 1").contains("5200"));
        assert!(!cmd(&mut m, "macro 1").contains("AVISO"));
        let mut tested = fake();
        assert!(!cmd(&mut tested, "show 1").contains("AVISO"));
        assert!(!cmd(&mut tested, "dpi 1 1 800").contains("AVISO"));
    }

    #[test]
    fn cobra_refuses_what_the_hardware_does_not_have() {
        let mut m = cobra();
        let try_cmd = |m: &mut Mouse<_>, line: &str| {
            let a: Vec<&str> = line.split_whitespace().collect();
            execute(m, a[0], &a[1..])
        };
        assert!(try_cmd(&mut m, "dpi 1 1 5100").is_err());
        assert!(try_cmd(&mut m, "dpi 1 1 12000").is_err());
        assert!(try_cmd(&mut m, "button 1 9 left").is_err());
        assert!(try_cmd(&mut m, "button 1 20 left").is_err());
        assert!(m.transport_mut().sent.iter().all(|p| p[1] != 0xF3));
    }

    fn cmd(m: &mut Mouse<salamandra_core::fake::MemoryTransport>, line: &str) -> String {
        let a: Vec<&str> = line.split_whitespace().collect();
        execute(m, a[0], &a[1..]).unwrap()
    }

    #[test]
    fn led_dpi_and_button_show_up_in_show() {
        let mut m = fake();
        cmd(&mut m, "led 1 FF8800 breathing 3 --brightness 2");
        cmd(&mut m, "dpi 1 2 1600x800");
        cmd(&mut m, "button 1 5 shortcut:copy");
        cmd(&mut m, "rate 1 500");
        let out = cmd(&mut m, "show 1");
        assert!(out.contains("#FF8800, respirar (vel 3), brilho 2"), "{out}");
        assert!(out.contains("DPI 2  1600x800"), "{out}");
        assert!(out.contains("botão  5: atalho copy"), "{out}");
        assert!(out.contains("500 Hz"), "{out}");
    }

    #[test]
    fn profile_switch_and_reset() {
        let mut m = fake();
        cmd(&mut m, "profile 3");
        assert!(cmd(&mut m, "show 3").contains("Perfil 3 (ativo)"));
        cmd(&mut m, "reset 3");
        assert!(cmd(&mut m, "show 3").contains("DPI 1"));
    }

    #[test]
    fn macro_set_get_clear() {
        let mut m = fake();
        cmd(&mut m, "macro 2 set down:lmeta tap:v up:lmeta");
        assert!(cmd(&mut m, "macro 2").starts_with("macro 2: "));
        cmd(&mut m, "macro 2 clear");
        assert_eq!(cmd(&mut m, "macro 2"), "macro 2: vazia\n");
    }

    #[test]
    fn backup_and_restore_round_trip() {
        let mut m = fake();
        cmd(&mut m, "led 2 00FF00 steady");
        let file = std::env::temp_dir().join("salamandra-cli-test-backup.json");
        let path = file.to_str().unwrap();
        cmd(&mut m, &format!("backup {path}"));
        cmd(&mut m, "led 2 0000FF steady");
        cmd(&mut m, &format!("restore {path}"));
        assert!(cmd(&mut m, "show 2").contains("#00FF00"));
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn bad_input_is_an_error_not_a_write() {
        let mut m = fake();
        assert!(execute(&mut m, "led", &["9", "FF0000"]).is_err());
        assert!(execute(&mut m, "dpi", &["1", "1", "150"]).is_err());
        assert!(execute(&mut m, "bogus", &[]).is_err());
        // Leituras (`F2`) podem acontecer; escrita e sessão, nunca.
        let wrote = m
            .transport_mut()
            .sent
            .iter()
            .any(|p| p[1] == 0xF3 || p[1] == 0xF5);
        assert!(!wrote);
    }
}
