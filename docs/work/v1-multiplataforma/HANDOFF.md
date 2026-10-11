# Handoff: v1-multiplataforma

Atualizado em: 11/10/2026

## Objetivo
Salamandra v1: 100% das funções do software oficial do M901-1 (só Windows),
rodando em Windows, Linux e macOS. Rust (núcleo, HID, serviço, CLI) + Tauri 2
com Svelte 5 (app).

## Estado atual
- branch: `main`, tudo commitado e enviado (fluxo: commit e push a cada task)
- concluído: Trilha 0, A.1 a A.7, C.1 a C.6
- testado no mouse (macOS), com confirmação do usuário: leitura, escrita
  (LED, DPI, polling, botões, macro, perfil ativo, reset, restore), eventos,
  atalho "copiar" pelo serviço, gravação de macro pelo app com tempo real,
  gravação de perfil pelo app, app empacotado (`Salamandra.app`, 4,4 MB)
- falta: C.7 (arquitetura do front), Trilha B (ajustes do sistema
  operacional), Trilha D (CI e instaladores), I.2 (Windows e Linux)

## Decisões tomadas
- `Mouse` no `salamandra-core`, genérica sobre `Transport`: lógica testável
  sem mouse. `salamandra-hid` só implementa o transporte (`hidapi`, feature
  `macos-shared-device`).
- Macros no formato do editor oficial: atraso `06 lo hi` em unidades de
  10 ms, até 63 eventos, mouse só L/R/M.
- Atalhos embutidos rodam no PC (`salamandra-service`, `enigo`); o app sobe o
  serviço numa thread e repassa os avisos do mouse para a tela.
- Troca de perfil e restore fazem uma sessão `F5` a mais, senão o LED não
  acompanha (conferido no mouse).
- Arquivo de perfil: JSON próprio `{format: "salamandra-profile", version: 1}`,
  validado ao carregar. Nomes das macros em `macro-names.json` na pasta de
  configuração do app (como no oficial, ficam só no PC).
- Licença MIT.

## Próximos passos
1. D.1: GitHub Actions com build e testes em Windows, Linux e macOS (é o
   único jeito de saber se compila nos outros sistemas sem máquina).
2. Trilha B: velocidade do ponteiro, aceleração e duplo clique por sistema.
3. C.7: trocar os singletons do front por instâncias injetadas.
4. D.2: instaladores e regra udev do Linux.

## Bloqueios e dúvidas em aberto
- Mídia "player" do oficial ainda sem código mapeado.
- Wayland: o serviço de atalhos usa `enigo` com X11.
- Sem máquina Windows e Linux para teste com o mouse (I.2).
- CI bloqueado: Actions não roda neste repo privado por faturamento da conta.
- Menu da bandeja e erros do backend só em pt-BR.

## Como verificar
- `verify .` (testes Rust, clippy, fmt, `svelte-check`, checagem de i18n)
- com o mouse: `cargo run -p salamandra-cli -- show`
- app: `cd app && npm run tauri dev`; release: `npm run tauri build -- --bundles app`

## Arquivos principais
- `docs/protocol.md`, `docs/official-features.md`
- `crates/salamandra-core/src/`: protocolo, modelos, eventos, `Mouse`
- `crates/salamandra-hid`, `crates/salamandra-service`, `crates/salamandra-cli`
- `app/src-tauri/src/lib.rs`: comandos Tauri, bandeja, serviço
- `app/src/lib/`: `state.svelte.ts`, `ProfileEditor.svelte`,
  `ButtonsTab.svelte`, `MacrosView.svelte`, `i18n.svelte.ts`, `nativeFeel.ts`

## Engenharia reversa
- Ghidra 12.1 em modo headless (`analyzeHeadless` com um script que despeja
  todas as funções em C), sobre `hid.exe` e `ldcfg.exe` do instalador oficial.
- Nada da decompilação é versionado; só os fatos, em `docs/protocol.md`.
