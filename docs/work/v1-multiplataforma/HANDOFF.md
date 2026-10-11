# Handoff: v1-multiplataforma

Atualizado em: 11/10/2026

## Objetivo
Salamandra v1: 100% das funções do software oficial do M901-1 (só Windows),
rodando em Windows, Linux e macOS. Rust (núcleo e CLI) + Tauri (app).

## Estado atual
- branch: `main`, só o README commitado e enviado; o resto está local, sem commit
- concluído: Trilha 0 (engenharia reversa e matriz) e A.1 a A.5 (núcleo, HID e CLI)
- testado no mouse (macOS): leitura de tudo pelo `hidapi`, backup idêntico ao do
  protótipo Swift (159 blocos), eventos `08 02 n` ao apertar DPI+/-
- ainda não testado pelo Rust: escrita no mouse (o protótipo Swift já validou
  os mesmos pacotes)
- próximo: A.7 (escrita pelo CLI Rust no mouse), depois A.6 (serviço) e Trilha C (app)

## Decisões tomadas
- Rust + `hidapi` (feature `macos-shared-device`, para o macOS continuar
  recebendo as teclas de mídia com o app aberto) e Tauri para a interface.
- `Mouse` fica no `salamandra-core`, genérica sobre `Transport`: toda a lógica
  testa sem mouse. O `salamandra-hid` só implementa o transporte.
- Macros no formato do editor oficial (`ldcfg.exe`): atraso `06 lo hi` em
  unidades de 10 ms, até 63 eventos, mouse só L/R/M. O mouse_m908 usa outro
  formato de atraso e eventos de movimento; não seguimos ele.
- Atalhos embutidos (copiar, colar, navegador...) são executados pelo PC, não
  pelo mouse: precisam do serviço em segundo plano (A.6).
- Pointer speed, aceleração e duplo clique do oficial são ajustes do Windows,
  não do mouse: viram Trilha B, por sistema.
- Licença ainda não escolhida (campo `license` fora dos `Cargo.toml`).

## Próximos passos
1. A.7: com o usuário, testar `salamandra led`, `button`, `macro`, `reset 4` e
   `restore` no mouse; backup antes em `~/Documents/perdition-backups/`.
2. A.6: serviço que lê `HidTransport::read_event` e simula as teclas dos atalhos
   (`docs/protocol.md`, seção de eventos) em cada sistema.
3. C.1: esqueleto Tauri em `app/`.

## Bloqueios e dúvidas em aberto
- Mídia "player" (MEDIA 8 do oficial) não está na tabela de códigos: mapear.
- Sem máquina Windows e Linux para teste com hardware (I.2).
- Licença do projeto: perguntar ao usuário antes de abrir o repo.

## Como verificar
- `verify .` (testes, clippy, fmt)
- com o mouse: `cargo run -p salamandra-cli -- show`

## Arquivos principais
- `docs/protocol.md`: protocolo, eventos, padrões de fábrica
- `docs/official-features.md`: matriz das funções oficiais e situação
- `crates/salamandra-core/src/`: `packet`, `map`, `model`, `buttons`, `macros`, `keys`, `defaults`, `mouse`
- `crates/salamandra-core/tests/mouse.rs`: `Mouse` contra transporte falso
- `crates/salamandra-hid/src/lib.rs`: transporte `hidapi`
- `crates/salamandra-cli/src/main.rs`: CLI `salamandra`

## Engenharia reversa
- Ghidra 12.1 em modo headless (`analyzeHeadless` com um script que despeja
  todas as funções em C), sobre `hid.exe` e `ldcfg.exe` do instalador oficial.
- Nada da decompilação é versionado; só os fatos, em `docs/protocol.md`.
