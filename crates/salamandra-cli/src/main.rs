use std::collections::BTreeMap;
use std::process::ExitCode;

use salamandra_core::buttons::{FireTarget, Modifiers, Shortcut, Special};
use salamandra_core::defaults::factory_profile;
use salamandra_core::{
    keys, macros, map, ButtonAction, DpiLevel, Error, LedMode, Mouse, PollingRate, Transport,
};
use salamandra_hid::HidTransport;

const USAGE: &str = "\
uso: salamandra <comando>

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

fn parse_action(s: &str) -> Result<ButtonAction, Error> {
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

fn describe(a: &ButtonAction) -> String {
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

fn show<T: Transport>(mouse: &mut Mouse<T>, p: usize, active: usize) -> Result<(), Error> {
    let prof = mouse.profile(p)?;
    println!(
        "Perfil {}{}",
        p + 1,
        if p == active { " (ativo)" } else { "" }
    );
    println!(
        "  polling: {}",
        prof.polling_rate
            .map_or("?".into(), |r| format!("{} Hz", r.hertz()))
    );
    println!("  rolagem: {} linha(s)", prof.scroll_speed);
    let [r, g, b] = prof.led.rgb;
    println!(
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
        println!(
            "  DPI {}{mark} {value}{}",
            i + 1,
            if l.enabled { "" } else { " (desligado)" }
        );
    }
    for (i, b) in prof.buttons.iter().enumerate() {
        println!("  botão {:2}: {}", i + 1, describe(b));
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

fn run(args: &[String]) -> Result<(), Error> {
    let Some(cmd) = args.first() else {
        print!("{USAGE}");
        return Ok(());
    };
    if cmd == "--help" || cmd == "-h" {
        print!("{USAGE}");
        return Ok(());
    }
    let a: Vec<&str> = args[1..].iter().map(String::as_str).collect();
    if cmd == "events" {
        let t = HidTransport::open()?;
        println!("escutando o mouse (Ctrl+C para sair)");
        loop {
            if let Some(r) = t.read_event(1000)? {
                println!("{}", salamandra_core::packet::hex(&r));
            }
        }
    }
    let mut mouse = salamandra_hid::open()?;
    match (cmd.as_str(), a.as_slice()) {
        ("show", rest) if rest.len() <= 1 => {
            let active = mouse.active_profile()?;
            let list: Vec<usize> = match rest.first() {
                Some(p) => vec![profile_arg(p)?],
                None => (0..map::PROFILE_COUNT).collect(),
            };
            for p in list {
                show(&mut mouse, p, active)?;
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
            edit(&mut mouse, p, |prof| {
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
            edit(&mut mouse, p, |prof| {
                prof.dpi_levels[i - 1] = l;
                Ok(())
            })?;
        }
        ("dpi-active", [p, level]) => {
            let i: usize = number(level, "nível")?;
            if !(1..=5).contains(&i) {
                return Err(invalid("nível deve ser 1 a 5"));
            }
            edit(&mut mouse, profile_arg(p)?, |prof| {
                prof.active_dpi_level = i - 1;
                Ok(())
            })?;
        }
        ("rate", [p, hz]) => {
            let rate = PollingRate::from_hertz(number(hz, "taxa")?)
                .ok_or_else(|| invalid(format!("taxa inválida: {hz}")))?;
            edit(&mut mouse, profile_arg(p)?, |prof| {
                prof.polling_rate = Some(rate);
                Ok(())
            })?;
        }
        ("scroll", [p, lines]) => {
            let lines: u8 = number(lines, "linhas")?;
            edit(&mut mouse, profile_arg(p)?, |prof| {
                prof.scroll_speed = lines;
                Ok(())
            })?;
        }
        ("button", [p, b, action]) => {
            let b: usize = number(b, "botão")?;
            if !(1..=map::BUTTON_COUNT).contains(&b) {
                return Err(invalid("botão deve ser 1 a 20"));
            }
            let action = parse_action(action)?;
            edit(&mut mouse, profile_arg(p)?, |prof| {
                prof.buttons[b - 1] = action;
                Ok(())
            })?;
        }
        ("macro", [n, rest @ ..]) => {
            let n: usize = number(n, "macro")?;
            if !(1..=map::MACRO_COUNT).contains(&n) {
                return Err(invalid(format!("macro deve ser 1 a {}", map::MACRO_COUNT)));
            }
            match rest {
                [] => {
                    let events = mouse.macro_events(n - 1)?;
                    if events.is_empty() {
                        println!("macro {n}: vazia");
                    } else {
                        println!("macro {n}: {}", macros::format(&events));
                    }
                }
                ["clear"] => mouse.set_macro(n - 1, &[])?,
                ["set", events @ ..] if !events.is_empty() => {
                    mouse.set_macro(n - 1, &macros::parse(&events.join(" "))?)?
                }
                _ => return Err(invalid("uso: macro <1-15> [clear | set <evento>...]")),
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
                mouse.update_profile(p, &old, &factory_profile(p))?;
            }
        }
        ("backup", [file]) => {
            let json = serde_json::to_string_pretty(&mouse.backup()?)
                .map_err(|e| invalid(e.to_string()))?;
            std::fs::write(file, json).map_err(|e| Error::Io(e.to_string()))?;
            println!("backup salvo em {file}");
        }
        ("restore", [file]) => {
            let text = std::fs::read_to_string(file).map_err(|e| Error::Io(e.to_string()))?;
            let json: BTreeMap<String, String> =
                serde_json::from_str(&text).map_err(|_| invalid("arquivo de backup inválido"))?;
            mouse.restore(&json)?;
            println!("backup restaurado");
        }
        _ => return Err(invalid("comando inválido. Veja: salamandra --help")),
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::FAILURE
        }
    }
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
}
