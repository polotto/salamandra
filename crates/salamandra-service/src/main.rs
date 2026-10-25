//! `salamandra-service`: roda o serviço no terminal, mostrando os avisos do mouse.

use std::sync::atomic::AtomicBool;

fn main() {
    let stop = AtomicBool::new(false);
    println!("salamandra-service: escutando o mouse (Ctrl+C para sair)");
    salamandra_service::run(
        &stop,
        || None,
        |event| println!("{event:?}"),
        |error| eprintln!("erro: {error}"),
    );
}
