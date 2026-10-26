# Tasks: ações customizadas (estilo BetterTouchTool)

Objetivo: o Salamandra deixa o usuário trocar o que cada atalho do mouse faz
no PC: macro de teclas, Mission Control, Hyper Key, abrir app ou URL, rodar
comando e sequência com pausa.

Decisões (26/10/2026):
- Formato: ações em JSON (`custom-actions.json`), sem plugin executável nesta
  fase. O contrato (`CustomAction`) fica aberto para uma API de plugins depois.
- Hyper Key: botão do mouse segura Ctrl+Alt+Shift+Cmd. Sem remapear Caps Lock
  (nada de `hidutil`, nada global no sistema).
- Limite do hardware: o mouse só avisa o PC dos 22 atalhos embutidos
  (`01 ff <id>` e `01 00 00`). O usuário grava um desses atalhos no botão e o
  serviço troca a ação no PC. Sem aviso, não há o que interceptar.

O que a engenharia reversa do BTT 5.536 mostrou (só nomes de ação e fatos):
- Ações: "Mission Control", "Launchpad", "Show Desktop", "Act as Hyper Key"
  (Shift, Control, Command, Option), Named Trigger ao soltar o Hyper sem uso.
- Hyper no BTT usa `HIDKeyboardModifierMappingSrc/Dst` (Caps Lock). Aqui não.
- Mission Control no macOS: `open -a "Mission Control"` basta, sem API privada.

## Trilha A: núcleo e serviço
- [x] A.1 `salamandra-core::custom`: tipos, nomes de tecla, plano por sistema
  - pronto quando: teste de cada ação em cada sistema
- [x] A.2 serviço executa o plano e segura o Hyper enquanto o botão está apertado
  - pronto quando: teste do carregamento do arquivo e do estado do Hyper
- [ ] A.3 teste no mouse real (Mission Control e Hyper), com confirmação humana

## Trilha B: interface
- [x] B.1 comandos Tauri para ler e gravar `custom-actions.json`
- [x] B.2 tela "Ações" no app (lista por atalho, editor de combo, escolha de ação pronta)
- [x] B.3 doc pública en e pt-BR (`docs/custom-actions.md`)
