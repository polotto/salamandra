# v1-multiplataforma: 100% do software oficial em Windows, Linux e macOS

Contexto: protótipo em Swift (`~/GitHub/perdition-mac`, só local) já cobre LED,
DPI, polling, perfil ativo, botões, macros, backup e restore, testado no mouse.
Esta versão porta para Rust + Tauri e completa o que falta.

## Trilha 0: contrato
- [x] T0.1 matriz das funções oficiais em `docs/official-features.md`, cada uma
  com endereço/comando ou ajuste do sistema operacional
  - pronto quando: toda tela e opção do `en.ini` oficial está na matriz
- [x] T0.2 engenharia reversa do que falta: formato do atraso nas macros,
  tiro rápido com tecla, velocidade de rolagem (`0x20`), callback de leitura
  (`FUN_00410be0`), padrões de fábrica dos 5 perfis
- [x] T0.3 portar `docs/protocol.md` do protótipo e completar com T0.2

## Trilha A: núcleo Rust (depende de T0.3)
- [x] A.1 workspace cargo, `salamandra-core` com pacote e mapa de memória, testes portados do Swift
  - pronto quando: `cargo test` passa com os mesmos casos do protótipo
- [x] A.2 modelos: LED, DPI, polling, rolagem, botões, macros, padrões de fábrica
- [x] A.3 validação: clique esquerdo obrigatório, faixas de DPI, macro com até 63 eventos
- [x] A.4 `salamandra-hid`: `hidapi`, `Mouse` com leitura, `update`, macros, backup, restore
  - pronto quando: `salamandra show` lê o mouse no macOS (ok: backup idêntico ao do protótipo, 159 blocos; eventos `08 02 n` recebidos)
- [x] A.5 CLI com os mesmos comandos do protótipo
- [x] A.6 serviço em segundo plano: escuta os reports de entrada e executa os
  atalhos embutidos no PC (`01 FF <id>`), por sistema
  - pronto quando: botão com `shortcut:copy` copia texto no macOS (ok, conferido pelo usuário)
  - pendente: Wayland no Linux (o `enigo` padrão só faz X11)
- [x] A.7 teste de escrita pelo CLI Rust no mouse real (LED, botão, macro, reset, restore): tudo conferido pelo usuário; o restore ganhou a segunda sessão para o LED

## Trilha B: ajustes do sistema operacional (paralela à A)
- [ ] B.1 velocidade do ponteiro, aceleração, duplo clique e linhas de rolagem
  no Windows (`SystemParametersInfo`)
- [ ] B.2 o mesmo no macOS
- [ ] B.3 o mesmo no Linux (GNOME/KDE por `gsettings`/`kwriteconfig`; o resto avisa)

## Trilha C: app Tauri (depende de A.4)
- [x] C.1 esqueleto Tauri, lista de perfis, conexão, avisos do mouse na tela
- [x] C.2 telas: Geral (polling, rolagem), DPI (link XY), Luz (gravação conferida no mouse pelo app); ajustes do SO ficam na Trilha B
- [x] C.3 botões com todas as funções e configurações de tiro rápido, com o aviso do clique esquerdo
- [ ] C.4 gerenciador de macros: gravar teclado e mouse com tempo, editar,
  inserir atraso, repetições, repetir enquanto segura
- [ ] C.5 salvar/carregar perfil em arquivo, restaurar perfil, resetar tudo
- [ ] C.6 ícone na bandeja e i18n pt-BR/en

## Trilha D: distribuição (depende de C.1)
- [ ] D.1 GitHub Actions: build e testes nos três sistemas
- [ ] D.2 instaladores: `.msi`/`.exe`, `.dmg`, `.AppImage`/`.deb`, com regra udev no Linux

## Integração
- [ ] I.1 teste no mouse real no macOS (com o usuário)
- [ ] I.2 teste no Windows e no Linux (com o usuário, quando tiver máquina)

## Fora do escopo
- Atualização de firmware (o oficial não tem).
- Outros mouses (a arquitetura deve permitir, mas a v1 é só o M901-1).
