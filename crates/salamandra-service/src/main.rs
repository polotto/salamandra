//! `salamandra-service`: roda o serviço no terminal, mostrando os avisos do mouse.

use std::sync::atomic::AtomicBool;

fn main() {
    let stop = AtomicBool::new(false);
    // `SALAMANDRA_ACTIONS` aponta para o custom-actions.json (ver docs/custom-actions.md).
    if let Ok(path) = std::env::var("SALAMANDRA_ACTIONS") {
        match salamandra_service::load_custom_actions(std::path::Path::new(&path)) {
            Ok(actions) => {
                if let Err(e) = salamandra_service::set_custom_actions(actions) {
                    eprintln!("erro: {e}");
                }
            }
            Err(e) => eprintln!("erro: {e}"),
        }
    }
    println!("salamandra-service: escutando o mouse (Ctrl+C para sair)");
    salamandra_service::run(
        &stop,
        || None,
        |event| println!("{event:?}"),
        |error| eprintln!("erro: {error}"),
    );
}
