# salamandra

Configurador aberto de mouses gamer para Windows, Linux e macOS. Primeiro mouse:
Redragon M901-1 Perdition 2 (USB `04d9:fc40`, firmware Holtek). Meta da v1:
100% das funções do software oficial (só Windows), em três sistemas.

## Comandos
- testes do núcleo: `cargo test`
- lint: `cargo clippy --all-targets -- -D warnings` e `cargo fmt --check`
- CLI: `cargo run -p salamandra-cli -- --help`
- app: `cd app && npm install && npm run tauri dev`
- checagem do front: `cd app && npm run check`

## Arquitetura
- `crates/salamandra-core/`: protocolo (pacotes, mapa de memória, modelos),
  sem I/O. Tudo testável sem mouse.
- `crates/salamandra-hid/`: transporte por `hidapi` (Windows `HidD_SetFeature`,
  Linux `hidraw`, macOS IOKit) e a API de alto nível `Mouse`.
- `crates/salamandra-cli/`: CLI.
- `crates/salamandra-service/`: serviço em segundo plano. Escuta os avisos do
  mouse e executa os atalhos (copiar, navegador...) com `enigo`. No macOS
  precisa da permissão de Acessibilidade.
- `app/`: Tauri (Rust + front web). Chama o `salamandra-hid` por comandos Tauri.
- `docs/protocol.md`: protocolo do M901-1, cada campo com a função de origem.
- `docs/official-features.md`: matriz função oficial x onde está no protocolo
  x situação. É a definição de "100%".

## Regras do código
- IMPORTANT: escrever no mouse só nos endereços e tamanhos que o software
  oficial usa (`docs/protocol.md`). Região desconhecida pode inutilizar o mouse.
- IMPORTANT: nunca gravar perfil sem nenhum botão com clique esquerdo (regra do
  oficial: "at least one click function to be retained").
- Montagem de pacote é função pura no `salamandra-core`, com teste que compara
  com os bytes do software oficial ou com leitura real do mouse.
- Nada do software oficial entra no repo: código, binário, imagem, `.ini` ou
  texto. Valores (endereços, códigos, padrões de fábrica) são fatos e entram
  como dados nossos, citando a origem.
- Texto de usuário em pt-BR e inglês: nenhuma string fixa no front, sempre
  `t("texto em pt-BR")` com a tradução em `app/src/lib/i18n.svelte.ts`
  (`node app/scripts/check-i18n.mjs` falha se faltar).
- App empacotado sem comportamento de navegador (`app/src/lib/nativeFeel.ts`):
  nada de seleção de texto fora de campos, Cmd+A, clique direito ou atalhos de
  navegador. Texto que o usuário precisa copiar leva a classe `selectable`.

## Fluxo de trabalho
- Trabalho atual: `docs/work/v1-multiplataforma/TASKS.md` e `HANDOFF.md`
- Verificar com `verify` antes de dizer que terminou (passos em `.agents/verify.steps`)
- Neste repo, commitar e dar push no `main` ao fim de cada task, com `verify`
  passando, e seguir para a próxima sem pedir (autorizado pelo dono em 11/10/2026)
- Commits em inglês, Conventional Commits sem escopo (`feat: ...`)

## Armadilhas
- Teste no mouse real precisa do mouse e de confirmação humana do efeito
  físico (LED, ponteiro, tecla). Não entra no `verify`.
- Depois de gravar o perfil ativo, o LED só troca na sessão `F5` seguinte: a
  troca de perfil repete a sessão.
- Linux precisa de regra udev para o usuário abrir o `hidraw` sem root.
