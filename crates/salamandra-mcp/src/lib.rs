//! Servidor MCP (Model Context Protocol) do Salamandra.
//!
//! Cada ferramenta vira um comando do CLI (`salamandra-cli`), então a validação
//! e a regra de segurança (nunca gravar perfil sem clique esquerdo) são as
//! mesmas nos dois. O servidor não conhece o mouse: recebe uma função que
//! executa os argumentos, o que permite testar sem hardware.

mod tools;

use salamandra_core::Error;
use serde_json::{json, Value};

/// Versões do protocolo que sabemos falar, da mais nova para a mais antiga.
const PROTOCOL_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "\
Configura mouses gamer pela memória do próprio mouse. As ferramentas set_* e \
restore GRAVAM no mouse; reset volta ao padrão de fábrica. Leia antes com \
mouse_show e confirme com o usuário antes de resetar ou restaurar. Perfis, \
níveis de DPI e botões começam em 1.";

pub struct Server<F> {
    run: F,
}

impl<F: FnMut(&[String]) -> Result<String, Error>> Server<F> {
    pub fn new(run: F) -> Self {
        Self { run }
    }

    /// Trata uma mensagem. `None` para notificação (não tem resposta).
    pub fn handle_line(&mut self, line: &str) -> Option<String> {
        let reply = match serde_json::from_str::<Value>(line) {
            Err(_) => Some(error(Value::Null, -32700, "JSON inválido")),
            Ok(msg) => self.handle(&msg),
        };
        reply.map(|v| v.to_string())
    }

    fn handle(&mut self, msg: &Value) -> Option<Value> {
        let Some(method) = msg.get("method").and_then(Value::as_str) else {
            // Respostas do cliente a pedidos nossos: não fazemos nenhum.
            return msg
                .get("id")
                .filter(|_| msg.get("result").is_none() && msg.get("error").is_none())
                .map(|id| error(id.clone(), -32600, "requisição inválida"));
        };
        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        let outcome = match method {
            "initialize" => Ok(initialize(&params)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tools::list() })),
            "tools/call" => self.call(&params),
            m if m.starts_with("notifications/") => return None,
            _ => Err((-32601, format!("método desconhecido: {method}"))),
        };
        // Sem id é notificação: nunca responde, nem com erro.
        let id = id?;
        Some(match outcome {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, message)) => error(id, code, &message),
        })
    }

    fn call(&mut self, params: &Value) -> Result<Value, (i64, String)> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or((-32602, "falta o nome da ferramenta".to_string()))?;
        let empty = json!({});
        let arguments = params.get("arguments").unwrap_or(&empty);
        let Some(tool) = tools::find(name) else {
            return Err((-32602, format!("ferramenta desconhecida: {name}")));
        };
        // Erro de uso ou do mouse volta como resultado com `isError`, para o
        // agente ver o texto e corrigir; erro de protocolo é só o de cima.
        let outcome = (tool.args)(arguments).and_then(|args| (self.run)(&args));
        Ok(match outcome {
            Ok(text) => json!({ "content": [{ "type": "text", "text": text_or_ok(text) }] }),
            Err(e) => {
                json!({ "content": [{ "type": "text", "text": e.to_string() }], "isError": true })
            }
        })
    }
}

fn text_or_ok(text: String) -> String {
    if text.trim().is_empty() {
        "ok".to_string()
    } else {
        text
    }
}

fn initialize(params: &Value) -> Value {
    let asked = params.get("protocolVersion").and_then(Value::as_str);
    let version = PROTOCOL_VERSIONS
        .iter()
        .find(|v| Some(**v) == asked)
        .unwrap_or(&PROTOCOL_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "salamandra", "version": env!("CARGO_PKG_VERSION") },
        "instructions": INSTRUCTIONS,
    })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests;
