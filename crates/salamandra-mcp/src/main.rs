//! Servidor MCP por stdio: uma mensagem JSON-RPC por linha. Só o protocolo vai
//! para a saída padrão; qualquer aviso vai para a de erro.

use std::io::{BufRead, Write};

use salamandra_mcp::Server;

fn main() {
    let mut server = Server::new(|args: &[String]| salamandra_cli::run(args));
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = server.handle_line(&line) {
            if writeln!(stdout, "{reply}")
                .and_then(|()| stdout.flush())
                .is_err()
            {
                break;
            }
        }
    }
}
