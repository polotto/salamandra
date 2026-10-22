//! As ferramentas: nome, descrição, esquema e a tradução para um comando do CLI.

use salamandra_core::Error;
use serde_json::{json, Value};

type Args = Result<Vec<String>, Error>;

pub struct Tool {
    pub name: &'static str,
    description: &'static str,
    schema: fn() -> Value,
    /// Só lê do mouse e não muda nada nele.
    read_only: bool,
    /// Apaga ou troca a configuração inteira.
    destructive: bool,
    pub args: fn(&Value) -> Args,
}

pub fn list() -> Vec<Value> {
    TOOLS
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": (t.schema)(),
                "annotations": {
                    "title": t.name,
                    "readOnlyHint": t.read_only,
                    "destructiveHint": t.destructive,
                    "idempotentHint": true,
                    "openWorldHint": false,
                },
            })
        })
        .collect()
}

pub fn find(name: &str) -> Option<&'static Tool> {
    TOOLS.iter().find(|t| t.name == name)
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

fn require<'a>(v: &'a Value, key: &str) -> Result<&'a Value, Error> {
    v.get(key)
        .filter(|x| !x.is_null())
        .ok_or_else(|| invalid(format!("falta o campo \"{key}\"")))
}

fn int(v: &Value, key: &str, min: i64, max: i64) -> Result<i64, Error> {
    let n = require(v, key)?
        .as_i64()
        .ok_or_else(|| invalid(format!("\"{key}\" deve ser um número inteiro")))?;
    if (min..=max).contains(&n) {
        Ok(n)
    } else {
        Err(invalid(format!("\"{key}\" deve ser de {min} a {max}")))
    }
}

fn opt_int(v: &Value, key: &str, min: i64, max: i64) -> Result<Option<i64>, Error> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => int(v, key, min, max).map(Some),
    }
}

/// Texto de uma palavra só: sem espaço nem quebra de linha, para não virar dois argumentos.
fn word(v: &Value, key: &str) -> Result<String, Error> {
    let s = require(v, key)?
        .as_str()
        .ok_or_else(|| invalid(format!("\"{key}\" deve ser texto")))?;
    if s.is_empty() || s.chars().any(char::is_whitespace) {
        return Err(invalid(format!("\"{key}\" não pode ter espaço")));
    }
    Ok(s.to_string())
}

fn opt_word(v: &Value, key: &str) -> Result<Option<String>, Error> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => word(v, key).map(Some),
    }
}

fn profile() -> Value {
    json!({ "type": "integer", "minimum": 1, "maximum": 5, "description": "Perfil, de 1 a 5" })
}

fn level() -> Value {
    json!({ "type": "integer", "minimum": 1, "maximum": 5, "description": "Nível de DPI, de 1 a 5" })
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": properties, "required": required })
}

fn strs<const N: usize>(parts: [String; N]) -> Vec<String> {
    parts.into()
}

fn s(x: impl ToString) -> String {
    x.to_string()
}

static TOOLS: [Tool; 13] = [
    Tool {
        name: "mouse_show",
        description: "Mostra a configuração do mouse: polling, rolagem, LED, DPI e botões. \
                      Sem perfil, mostra os cinco.",
        schema: || object(json!({ "profile": profile() }), &[]),
        read_only: true,
        destructive: false,
        args: |v| {
            let mut a = strs([s("show")]);
            if let Some(p) = opt_int(v, "profile", 1, 5)? {
                a.push(s(p));
            }
            Ok(a)
        },
    },
    Tool {
        name: "mouse_macro_get",
        description: "Lê as ações gravadas numa macro (1 a 15).",
        schema: || object(json!({ "macro": { "type": "integer", "minimum": 1, "maximum": 15 } }), &["macro"]),
        read_only: true,
        destructive: false,
        args: |v| Ok(strs([s("macro"), s(int(v, "macro", 1, 15)?)])),
    },
    Tool {
        name: "mouse_backup",
        description: "Salva a configuração inteira do mouse, com as macros, num arquivo JSON do computador.",
        schema: || object(json!({ "file": { "type": "string", "description": "Caminho do arquivo .json, sem espaços" } }), &["file"]),
        read_only: false,
        destructive: false,
        args: |v| Ok(strs([s("backup"), word(v, "file")?])),
    },
    Tool {
        name: "mouse_set_profile",
        description: "Troca o perfil ativo do mouse.",
        schema: || object(json!({ "profile": profile() }), &["profile"]),
        read_only: false,
        destructive: false,
        args: |v| Ok(strs([s("profile"), s(int(v, "profile", 1, 5)?)])),
    },
    Tool {
        name: "mouse_set_led",
        description: "Grava a cor e o efeito do LED de um perfil.",
        schema: || {
            object(
                json!({
                    "profile": profile(),
                    "color": { "type": "string", "pattern": "^#?[0-9a-fA-F]{6}$", "description": "RRGGBB em hexadecimal" },
                    "mode": { "type": "string", "enum": ["steady", "breathing", "flash", "off"], "description": "Padrão: steady" },
                    "speed": { "type": "integer", "minimum": 1, "maximum": 8, "description": "Só para breathing e flash" },
                    "brightness": { "type": "integer", "minimum": 1, "maximum": 3 },
                }),
                &["profile", "color"],
            )
        },
        read_only: false,
        destructive: false,
        args: |v| {
            let mut a = strs([s("led"), s(int(v, "profile", 1, 5)?), word(v, "color")?]);
            let mode = opt_word(v, "mode")?;
            let speed = opt_int(v, "speed", 1, 8)?;
            if mode.is_some() || speed.is_some() {
                a.push(mode.unwrap_or_else(|| s("steady")));
            }
            if let Some(sp) = speed {
                a.push(s(sp));
            }
            if let Some(b) = opt_int(v, "brightness", 1, 3)? {
                a.extend([s("--brightness"), s(b)]);
            }
            Ok(a)
        },
    },
    Tool {
        name: "mouse_set_dpi",
        description: "Grava um nível de DPI (100 a 24000, de 100 em 100). Use \"1600x800\" para X e Y diferentes.",
        schema: || {
            object(
                json!({
                    "profile": profile(),
                    "level": level(),
                    "dpi": { "type": ["integer", "string"], "description": "Número, ou \"XxY\"" },
                    "enabled": { "type": "boolean", "description": "false desliga o nível. Padrão: true" },
                }),
                &["profile", "level", "dpi"],
            )
        },
        read_only: false,
        destructive: false,
        args: |v| {
            let dpi = match require(v, "dpi")? {
                Value::Number(n) => n.to_string(),
                _ => word(v, "dpi")?,
            };
            let mut a = strs([s("dpi"), s(int(v, "profile", 1, 5)?), s(int(v, "level", 1, 5)?), dpi]);
            if v.get("enabled").and_then(Value::as_bool) == Some(false) {
                a.push(s("--off"));
            }
            Ok(a)
        },
    },
    Tool {
        name: "mouse_set_active_dpi_level",
        description: "Escolhe qual nível de DPI fica ativo num perfil.",
        schema: || object(json!({ "profile": profile(), "level": level() }), &["profile", "level"]),
        read_only: false,
        destructive: false,
        args: |v| Ok(strs([s("dpi-active"), s(int(v, "profile", 1, 5)?), s(int(v, "level", 1, 5)?)])),
    },
    Tool {
        name: "mouse_set_polling_rate",
        description: "Grava a taxa de polling de um perfil: 125, 250, 500 ou 1000 Hz.",
        schema: || {
            object(
                json!({ "profile": profile(), "hz": { "type": "integer", "enum": [125, 250, 500, 1000] } }),
                &["profile", "hz"],
            )
        },
        read_only: false,
        destructive: false,
        args: |v| Ok(strs([s("rate"), s(int(v, "profile", 1, 5)?), s(int(v, "hz", 1, 1000)?)])),
    },
    Tool {
        name: "mouse_set_scroll",
        description: "Grava quantas linhas a roda rola por clique, num perfil.",
        schema: || {
            object(
                json!({ "profile": profile(), "lines": { "type": "integer", "minimum": 1, "maximum": 255 } }),
                &["profile", "lines"],
            )
        },
        read_only: false,
        destructive: false,
        args: |v| Ok(strs([s("scroll"), s(int(v, "profile", 1, 5)?), s(int(v, "lines", 1, 255)?)])),
    },
    Tool {
        name: "mouse_set_button",
        description: "Grava a ação de um botão. Botões: 1 esquerdo, 2 direito, 3 meio, 4 tiro rápido, \
                      5 DPI+, 6 DPI-, 7 a 18 laterais, 19 roda para cima, 20 roda para baixo. \
                      Ações: left, right, middle, back, forward, disabled, key:<tecla>, \
                      combo:<ctrl+shift+alt+meta>:<tecla>, media:<play|stop|prev|next|volup|voldown|mute>, \
                      macro:<1-15>[:repeat|:<vezes>], fire:<left|right|middle|tecla>:<vezes>:<ms>, \
                      special:<dpi-cycle|dpi-up|dpi-down|scroll-up|scroll-down|profile|rate-up|rate-down|led-mode>, \
                      shortcut:<copy|paste|cut|select-all|find|new|print|save|switch-window|close-window|files|run|desktop|lock|home|back|forward|stop|refresh|search|favorites|mail>. \
                      O mouse precisa manter ao menos um botão com clique esquerdo.",
        schema: || {
            object(
                json!({
                    "profile": profile(),
                    "button": { "type": "integer", "minimum": 1, "maximum": 20 },
                    "action": { "type": "string", "description": "Ex.: left, key:a, combo:ctrl+shift:f, shortcut:copy" },
                }),
                &["profile", "button", "action"],
            )
        },
        read_only: false,
        destructive: false,
        args: |v| {
            Ok(strs([
                s("button"),
                s(int(v, "profile", 1, 5)?),
                s(int(v, "button", 1, 20)?),
                word(v, "action")?,
            ]))
        },
    },
    Tool {
        name: "mouse_set_macro",
        description: "Grava uma macro (1 a 15) ou a esvazia. Eventos separados por espaço: \
                      down:<tecla> up:<tecla> tap:<tecla> mdown:<left|right|middle> mup:<...> \
                      click:<...> delay:<ms>. Ex.: \"down:lmeta tap:v up:lmeta\". Até 63 eventos.",
        schema: || {
            object(
                json!({
                    "macro": { "type": "integer", "minimum": 1, "maximum": 15 },
                    "events": { "type": "string", "description": "Eventos separados por espaço" },
                    "clear": { "type": "boolean", "description": "true esvazia a macro (sem events)" },
                }),
                &["macro"],
            )
        },
        read_only: false,
        destructive: false,
        args: |v| {
            let n = int(v, "macro", 1, 15)?;
            let clear = v.get("clear").and_then(Value::as_bool) == Some(true);
            let events = v.get("events").and_then(Value::as_str).unwrap_or("").trim();
            match (clear, events.is_empty()) {
                (true, true) => Ok(strs([s("macro"), s(n), s("clear")])),
                (false, false) => {
                    let mut a = strs([s("macro"), s(n), s("set")]);
                    a.extend(events.split_whitespace().map(str::to_string));
                    Ok(a)
                }
                _ => Err(invalid("passe \"events\" para gravar ou \"clear\": true para esvaziar, não os dois")),
            }
        },
    },
    Tool {
        name: "mouse_reset",
        description: "Volta um perfil (ou todos) ao padrão de fábrica. Apaga a configuração atual: \
                      confirme com o usuário antes.",
        schema: || {
            object(
                json!({ "profile": { "type": ["integer", "string"], "description": "1 a 5, ou \"all\"" } }),
                &["profile"],
            )
        },
        read_only: false,
        destructive: true,
        args: |v| {
            let which = match require(v, "profile")? {
                Value::String(x) if x == "all" => s("all"),
                _ => s(int(v, "profile", 1, 5)?),
            };
            Ok(strs([s("reset"), which]))
        },
    },
    Tool {
        name: "mouse_restore",
        description: "Grava no mouse um backup feito por mouse_backup. Troca a configuração inteira: \
                      confirme com o usuário antes.",
        schema: || object(json!({ "file": { "type": "string", "description": "Caminho do .json, sem espaços" } }), &["file"]),
        read_only: false,
        destructive: true,
        args: |v| Ok(strs([s("restore"), word(v, "file")?])),
    },
];
