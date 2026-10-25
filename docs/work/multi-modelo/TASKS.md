# Tasks: multi-modelo e logo

Objetivo: o Salamandra ganha logo e passa a reconhecer outros mouses Redragon,
começando pelos mais populares, deixando claro o que nunca foi testado no físico.

Decisões (22/10/2026):
- Primeiro lote: M711 Cobra (`04d9:fc30`), M908 Impact (`04d9:fc4d`),
  M719 Invader (`04d9:fc4f`), M607 Griffin (`04d9:fc38`).
- Modelo não testado grava direto, com aviso (banner no app, aviso no CLI,
  selo na documentação). Não há modo só leitura.
- Logo: salamandra estilizada e geométrica, uma cor de destaque, legível em 16 px.
- Fonte do protocolo dos outros modelos: [mouse_m908](https://github.com/dokutan/mouse_m908)
  (GPL-3). O Salamandra é MIT: só entram fatos (endereços, bytes, tabelas)
  como dados nossos, citando a origem. Nenhum código é copiado.

O que a fonte mostra (base das tasks):
- Mesmo enquadramento de pacote do M901-1 (`02 F3 addr len`, `F1 02 m`,
  `F5 00/01`), então `packet.rs` e o transporte servem.
- Muda por modelo: codificação do DPI (tabela de códigos, M711 com X e Y
  separados, M719 com escala própria), botões por perfil (10 no M719/M607/M711,
  20 no M908), slots de macro (1 no M719, 15 nos outros), modos de LED
  (M908 tem ondas e reativo, sem equivalente no M901-1) e endereços.
- O `model.rs` atual assume um modelo só e precisa virar dados por modelo.

## Trilha L: logo (independente das outras)
- [x] L.1 SVG mestre da salamandra (`assets/logo/salamandra.svg`), versão
  monocromática (`salamandra-mono.svg`) e fundo claro/escuro
  - pronto quando: legível em 16, 32 e 512 px, conferido por captura
- [x] L.2 gerar os ícones do Tauri (`npm run tauri icon`), favicon do front e
  ícone da bandeja (template monocromático no macOS)
  - pronto quando: `Salamandra.app` e a bandeja mostram o novo ícone
  - bandeja do macOS: silhueta template (`tray-template.png`) feita em 24/10/2026; conferido a olho nos temas claro e escuro em 24/10/2026
- [x] L.3 logo no topo do `README.md` e `README.pt-BR.md` e no cabeçalho do app
- [ ] L.4 atualizar `docs/work` e o card do projeto no polotto.dev só se o
  Angelo pedir (imagem em card está fora do escopo do blog)

## Trilha M: núcleo multi-modelo (base das trilhas P e S)
- [x] M.1 `DeviceSpec` em `salamandra-core` (família 0 a 4 e sensor, como no oficial, ver `RE-FINDINGS.md`): id, nome, VID/PID, `status`
  (`Tested` ou `Untested`), fonte, perfis, níveis e faixa de DPI, X/Y
  separado, botões por perfil, slots de macro, modos de LED aceitos e mapa
  de endereços. O M901-1 vira a primeira spec sem mudar nenhum byte
  - pronto quando: todos os testes atuais passam sem alteração de expectativa
- [x] M.2 o `Mouse` e o `model.rs` passam a receber a spec; codificação de DPI
  por estratégia (`DpiCodec`) em vez de função fixa
  - pronto quando: teste de bytes do M901-1 idêntico ao de hoje
- [x] M.3 detecção (feito: `--model`/`SALAMANDRA_MODEL` escolhem entre vários; o app tem seletor na barra lateral quando há mais de um; o serviço de atalhos segue o mesmo mouse): `salamandra-hid` enumera todos os PIDs da tabela; com mais
  de um mouse ligado o CLI pede qual usar e o app abre o primeiro e oferece o seletor; `Error::NotFound` e o
  texto "mouse 04d9:fc40 não encontrado" deixam de citar um modelo fixo
- [ ] M.4 (feito para o M711 Cobra) `docs/models/<slug>.md` (en e pt-BR) por modelo: origem de cada campo,
  diferença para o M901-1, o que falta confirmar

## Trilha S: sinalização de "não testado" (depende de M.1)
- [x] S.1 backend expõe `tested: bool` e `source` do modelo conectado
- [x] S.2 app: banner fixo "Modelo não testado no mouse físico. Pode se
  comportar de forma inesperada" em todas as telas, com link para a issue de
  teste; texto em pt-BR e en no i18n
- [x] S.3 CLI: aviso em qualquer comando que grava; sem confirmação, porque a
  decisão foi gravar direto (o MCP herda o aviso, o que fecha o N.2)
- [x] S.4 README (template `model-test.yml`) (en e pt-BR): tabela de mouses com coluna Status
  (`tested`, `untested: mapped from public docs`) e como relatar teste;
  template de issue "testei o modelo X" com o que anexar (saída de `show`,
  efeito físico de LED, DPI e botão)
- [x] S.5 promoção de status: regra no `CONTRIBUTING` (um relato com `show`
  antes e depois e confirmação do efeito físico muda `Untested` para `Tested`)

## Trilha P: um modelo por vez (depende de M.1 a M.3; ordem de popularidade)
Cada modelo só vira PR/commit quando tem spec, teste de montagem de pacote
contra os bytes de captura da fonte, `docs/models/` e linha na tabela do
README como `untested`.
- [x] P.1 M711 Cobra `04d9:fc30` (família 0, sensor 3325: reaproveita o M901-1, muda a tabela de DPI)
- [x] P.2 M908 Impact (feito: DPI, 20 botões, LED família 3 com os 8 modos e o `0x446`) `04d9:fc4d` (20 botões por perfil, LED com ondas e reativo)
- [x] P.3 M719 Invader `04d9:fc4f` (1 macro; a escala de DPI é a mesma do sensor 3325, conferida contra a captura)
- [ ] P.4 (bloqueado: o oficial e o mouse_m908 divergem na tabela de DPI do sensor 3212, ver RE-FINDINGS) M607 Griffin `04d9:fc38` (a fonte diz que a leitura não funciona
  direito: gravar com aviso extra, leitura tratada como melhor esforço)

## Trilha R: engenharia reversa com o REA (alimenta a trilha P)
Para interoperabilidade, como no M901-1: nada do oficial entra no repo, só fatos
citando a função de origem.
- [x] R.1 baixar e extrair os instaladores (King Cobra 2, Cobra V2, B703)
- [x] R.2 rodar `rea analyze` no `RDCfg.exe`; achar que a seleção de modelo é
  por nome e que `FUN_0040e870` é só interface
- [x] R.3a achar a camada que fala com o mouse (`hid_send_feature_report`, `0x456c40` a `0x45c910`) e as tabelas de família e sensor (ver `RE-FINDINGS.md`)
- [ ] R.3 decompilar `HidDevice.dll` e rastrear os chamadores de envio/leitura
  no `RDCfg.exe` (`rea trace` em "pollrate", "DPI_INFO", `F3`)
- [ ] R.4 (LED das famílias lido, com a divergência de sub das famílias 0 a 2 em aberto; macros e mapa de botões por modelo lidos em 24/10/2026; faltam os bytes de cada ação por família em `FUN_0045c570` e o papel de `FUN_00458530`) por modelo do primeiro lote: endereço do perfil, codificação do DPI,
  botões por perfil, macros e LED, cada campo com a função de origem
  em `docs/models/<slug>.md`
  - pronto quando: o teste de pacote de cada modelo bate com os bytes que a
    função do oficial gera, e com a captura do mouse_m908 quando houver
- [ ] R.5 cruzar o resultado do R.4 com o mouse_m908; divergência vira dúvida
  em aberto, não dado

## Trilha Q: lista do site da Redragon (depende do usuário)
O `redragon.com.br/downloads` lista software para M711 King Cobra 2, M711 Cobra V2,
M988 Storm Elite, ID711 Ryu e B703 Cerberus. Não sabemos o VID:PID deles
nem se o protocolo é o mesmo.
- [ ] Q.1 Angelo informa `lsusb`/Informações do Sistema de cada um que tiver,
  ou o assunto vira issue "peço suporte" na comunidade
- [ ] Q.2 com o VID:PID, conferir se cai em um modelo da trilha P; se não,
  decidir se vale baixar o software oficial e fazer a engenharia reversa
  como foi feito no M901-1 (decisão do Angelo, trabalho grande)

## Trilha K: CLI sem Tauri (independente)
- [x] K.1 `salamandra-cli` vira biblioteca (comandos devolvem texto) mais binário
  fino; sem mudança de comportamento
  - pronto quando: testes atuais passam e `salamandra show` sai igual
- [x] K.2 `default-members` do workspace sem o app: `cargo build` e `cargo test`
  na raiz não compilam o Tauri; `./run.sh cli` gera `salamandra` e
  `salamandra-mcp` em release
  - pronto quando: `cargo tree -p salamandra-cli` não tem tauri, wry, tao nem webkit
- [x] K.3 job `cli` no CI (Linux só com `libudev-dev`, Windows e macOS sem nada)
- [x] K.4 binários do CLI e do MCP anexados à Release por tag, por sistema (job `cli` do
  `release.yml`, depois do `build`; o `gh release upload` em rascunho só será
  provado na próxima tag)

## Trilha N: servidor MCP (depende de K.1)
- [x] N.1 crate `salamandra-mcp`: JSON-RPC por stdio escrito à mão (só `serde_json`),
  ferramentas que chamam a biblioteca do CLI: ler (`show`, `macro_get`,
  `backup`) e gravar (`set_led`, `set_dpi`, `set_rate`, `set_scroll`,
  `set_button`, `set_macro`, `set_profile`, `reset`, `restore`)
  - pronto quando: teste de protocolo (`initialize`, `tools/list`, `tools/call`)
    passa sem mouse, usando um transporte falso
- [x] N.2 anotações das ferramentas (feito: `readOnlyHint`, `destructiveHint` para
  `reset` e `restore`); falta o aviso de modelo não testado na resposta de gravação
  (depende da trilha S)
- [x] N.3 README (en e pt-BR): como registrar o servidor no Claude Code e em
  outros clientes MCP

## Fora do escopo
- Mouses de outros protocolos (M601-RGB, M801 Mammoth, M913 sem fio e a família
  SinoWealth `25a7`): o protocolo é outro e só dá para mapear com o software
  oficial ou captura USB.
- Modo só leitura para modelo não testado (decidido: grava direto, com aviso).
- Teclados, headsets e o resto da página de downloads.

## Integração
- [ ] I.1 `verify` verde com os 5 modelos na tabela
- [ ] I.2 nova versão: subir em `Cargo.toml`, `app/package.json` e
  `tauri.conf.json`, tag `vX.Y.Z`, conferir o rascunho da release
