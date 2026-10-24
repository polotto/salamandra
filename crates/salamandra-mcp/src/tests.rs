use salamandra_cli::execute;
use salamandra_core::fake::MemoryTransport;
use salamandra_core::{Error, Mouse};
use serde_json::{json, Value};

use super::{tools, Server};

fn mouse() -> Mouse<MemoryTransport> {
    let mut m = Mouse::new(MemoryTransport::default());
    m.session_delay = std::time::Duration::ZERO;
    // Memória zerada não tem clique esquerdo; parte do padrão de fábrica.
    execute(&mut m, "reset", &["all"]).unwrap();
    m
}

/// Servidor ligado a um mouse em memória.
fn server(
    m: &mut Mouse<MemoryTransport>,
) -> Server<impl FnMut(&[String]) -> Result<String, Error> + '_> {
    Server::new(move |args: &[String]| {
        let a: Vec<&str> = args[1..].iter().map(String::as_str).collect();
        execute(m, &args[0], &a)
    })
}

fn send<F: FnMut(&[String]) -> Result<String, Error>>(s: &mut Server<F>, msg: Value) -> Value {
    let reply = s
        .handle_line(&msg.to_string())
        .expect("tinha que responder");
    serde_json::from_str(&reply).unwrap()
}

fn call<F: FnMut(&[String]) -> Result<String, Error>>(
    s: &mut Server<F>,
    name: &str,
    arguments: Value,
) -> Value {
    send(
        s,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": name, "arguments": arguments } }),
    )
}

fn text(reply: &Value) -> &str {
    reply["result"]["content"][0]["text"].as_str().unwrap()
}

#[test]
fn initialize_negotiates_version_and_announces_tools() {
    let mut m = mouse();
    let mut s = server(&mut m);
    let r = send(
        &mut s,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": { "protocolVersion": "2024-11-05", "capabilities": {},
                            "clientInfo": { "name": "t", "version": "0" } } }),
    );
    assert_eq!(r["result"]["protocolVersion"], "2024-11-05");
    assert!(r["result"]["capabilities"]["tools"].is_object());
    assert_eq!(r["result"]["serverInfo"]["name"], "salamandra");
    let r = send(
        &mut s,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "initialize", "params": { "protocolVersion": "1999-01-01" } }),
    );
    assert_eq!(r["result"]["protocolVersion"], "2025-06-18");
}

#[test]
fn notifications_get_no_reply() {
    let mut m = mouse();
    let mut s = server(&mut m);
    let msg = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
    assert!(s.handle_line(&msg.to_string()).is_none());
    let unknown_note = json!({ "jsonrpc": "2.0", "method": "algo/novo" });
    assert!(s.handle_line(&unknown_note.to_string()).is_none());
}

#[test]
fn protocol_errors_have_json_rpc_codes() {
    let mut m = mouse();
    let mut s = server(&mut m);
    let r: Value = serde_json::from_str(&s.handle_line("{nao é json").unwrap()).unwrap();
    assert_eq!(r["error"]["code"], -32700);
    let r = send(
        &mut s,
        json!({ "jsonrpc": "2.0", "id": 7, "method": "nada" }),
    );
    assert_eq!(r["error"]["code"], -32601);
    assert_eq!(r["id"], 7);
    let r = call(&mut s, "mouse_inexistente", json!({}));
    assert_eq!(r["error"]["code"], -32602);
}

#[test]
fn every_tool_has_a_valid_schema_and_unique_name() {
    let list = tools::list();
    assert_eq!(list.len(), 13);
    let mut names: Vec<&str> = list.iter().map(|t| t["name"].as_str().unwrap()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 13);
    for t in &list {
        let schema = &t["inputSchema"];
        assert_eq!(schema["type"], "object", "{}", t["name"]);
        let props = schema["properties"].as_object().unwrap();
        for req in schema["required"].as_array().unwrap() {
            assert!(props.contains_key(req.as_str().unwrap()), "{}", t["name"]);
        }
        assert!(!t["description"].as_str().unwrap().is_empty());
    }
}

#[test]
fn only_reset_and_restore_are_destructive_and_reads_are_marked() {
    let list = tools::list();
    let flag = |name: &str, hint: &str| {
        list.iter().find(|t| t["name"] == name).unwrap()["annotations"][hint]
            .as_bool()
            .unwrap()
    };
    assert!(flag("mouse_reset", "destructiveHint"));
    assert!(flag("mouse_restore", "destructiveHint"));
    assert!(!flag("mouse_set_led", "destructiveHint"));
    assert!(flag("mouse_show", "readOnlyHint"));
    assert!(flag("mouse_macro_get", "readOnlyHint"));
    assert!(!flag("mouse_set_dpi", "readOnlyHint"));
}

#[test]
fn set_led_refuses_a_mode_the_model_does_not_have() {
    let mut m = mouse();
    let mut s = server(&mut m);
    let r = call(
        &mut s,
        "mouse_set_led",
        json!({ "profile": 1, "color": "FF8800", "mode": "rainbow", "speed": 3 }),
    );
    assert_eq!(r["result"]["isError"], json!(true), "{r}");
}

#[test]
fn writes_are_visible_in_a_later_read() {
    let mut m = mouse();
    let mut s = server(&mut m);
    let r = call(
        &mut s,
        "mouse_set_led",
        json!({ "profile": 1, "color": "FF8800", "mode": "breathing", "speed": 3, "brightness": 2 }),
    );
    assert!(r["result"]["isError"].is_null(), "{r}");
    let r = call(
        &mut s,
        "mouse_set_dpi",
        json!({ "profile": 1, "level": 2, "dpi": "1600x800" }),
    );
    assert!(r["result"]["isError"].is_null(), "{r}");
    let r = call(
        &mut s,
        "mouse_set_dpi",
        json!({ "profile": 1, "level": 3, "dpi": 3200 }),
    );
    assert!(r["result"]["isError"].is_null(), "{r}");
    call(
        &mut s,
        "mouse_set_polling_rate",
        json!({ "profile": 1, "hz": 500 }),
    );
    call(
        &mut s,
        "mouse_set_button",
        json!({ "profile": 1, "button": 5, "action": "shortcut:copy" }),
    );
    let r = call(&mut s, "mouse_show", json!({ "profile": 1 }));
    let out = text(&r);
    assert!(out.contains("#FF8800, respirar (vel 3), brilho 2"), "{out}");
    assert!(out.contains("1600x800"), "{out}");
    assert!(out.contains("3200"), "{out}");
    assert!(out.contains("500 Hz"), "{out}");
    assert!(out.contains("atalho copy"), "{out}");
}

#[test]
fn macro_round_trip_and_clear() {
    let mut m = mouse();
    let mut s = server(&mut m);
    call(
        &mut s,
        "mouse_set_macro",
        json!({ "macro": 4, "events": "down:lmeta tap:v up:lmeta" }),
    );
    let r = call(&mut s, "mouse_macro_get", json!({ "macro": 4 }));
    assert!(text(&r).starts_with("macro 4: "), "{r}");
    call(
        &mut s,
        "mouse_set_macro",
        json!({ "macro": 4, "clear": true }),
    );
    let r = call(&mut s, "mouse_macro_get", json!({ "macro": 4 }));
    assert_eq!(text(&r).trim(), "macro 4: vazia");
}

#[test]
fn bad_arguments_return_is_error_and_never_write() {
    let mut m = mouse();
    m.transport_mut().sent.clear();
    {
        let mut s = server(&mut m);
        for (tool, args) in [
            ("mouse_set_led", json!({ "profile": 9, "color": "FF0000" })),
            ("mouse_set_led", json!({ "profile": 1 })),
            (
                "mouse_set_led",
                json!({ "profile": 1, "color": "vermelho" }),
            ),
            (
                "mouse_set_dpi",
                json!({ "profile": 1, "level": 1, "dpi": 150 }),
            ),
            (
                "mouse_set_dpi",
                json!({ "profile": 1, "level": 1, "dpi": "1600 x" }),
            ),
            ("mouse_set_polling_rate", json!({ "profile": 1, "hz": 300 })),
            (
                "mouse_set_polling_rate",
                json!({ "profile": "um", "hz": 500 }),
            ),
            (
                "mouse_set_button",
                json!({ "profile": 1, "button": 21, "action": "left" }),
            ),
            (
                "mouse_set_button",
                json!({ "profile": 1, "button": 1, "action": "nada" }),
            ),
            ("mouse_set_macro", json!({ "macro": 1 })),
            (
                "mouse_set_macro",
                json!({ "macro": 1, "events": "tap:a", "clear": true }),
            ),
            ("mouse_reset", json!({ "profile": "todos" })),
        ] {
            let r = call(&mut s, tool, args.clone());
            assert_eq!(r["result"]["isError"], true, "{tool} {args}: {r}");
            assert!(!text(&r).is_empty());
        }
    }
    let wrote = m
        .transport_mut()
        .sent
        .iter()
        .any(|p| p[1] == 0xF3 || p[1] == 0xF5);
    assert!(!wrote, "uma chamada inválida gravou no mouse");
}

#[test]
fn refuses_to_leave_the_profile_without_a_left_click() {
    let mut m = mouse();
    let mut s = server(&mut m);
    // No padrão de fábrica o botão 1 é o único clique esquerdo do perfil.
    let r = call(
        &mut s,
        "mouse_set_button",
        json!({ "profile": 1, "button": 1, "action": "disabled" }),
    );
    assert_eq!(r["result"]["isError"], true, "{r}");
    assert!(text(&r).contains("clique esquerdo"), "{r}");
}

#[test]
fn reset_restores_factory_values() {
    let mut m = mouse();
    let mut s = server(&mut m);
    call(
        &mut s,
        "mouse_set_led",
        json!({ "profile": 2, "color": "123456" }),
    );
    let r = call(&mut s, "mouse_reset", json!({ "profile": 2 }));
    assert!(r["result"]["isError"].is_null(), "{r}");
    let r = call(&mut s, "mouse_show", json!({ "profile": 2 }));
    assert!(!text(&r).contains("#123456"), "{r}");
}

#[test]
fn backup_and_restore_through_a_file() {
    let mut m = mouse();
    let mut s = server(&mut m);
    let file = std::env::temp_dir().join("salamandra-mcp-test-backup.json");
    let path = file.to_str().unwrap();
    call(
        &mut s,
        "mouse_set_led",
        json!({ "profile": 3, "color": "00FF00" }),
    );
    let r = call(&mut s, "mouse_backup", json!({ "file": path }));
    assert!(r["result"]["isError"].is_null(), "{r}");
    call(
        &mut s,
        "mouse_set_led",
        json!({ "profile": 3, "color": "0000FF" }),
    );
    let r = call(&mut s, "mouse_restore", json!({ "file": path }));
    assert!(r["result"]["isError"].is_null(), "{r}");
    let r = call(&mut s, "mouse_show", json!({ "profile": 3 }));
    assert!(text(&r).contains("#00FF00"), "{r}");
    let _ = std::fs::remove_file(file);
}
