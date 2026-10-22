# Handoff: multi-modelo e logo

Atualizado em: 22/10/2026

## Objetivo
Dar uma logo ao Salamandra e suportar mais mouses Redragon, os populares
primeiro, sempre marcando o que não foi testado no físico.

## Estado atual
- Planejado, nada implementado. `TASKS.md` aguardando o ok do Angelo.
- Repo: `~/GitHub/salamandra`, branch `main`, fluxo do repo: commit e push
  por task com `verify` passando.

## Decisões tomadas
Ver `TASKS.md`. Em resumo: lote M711, M908, M719, M607; grava direto com
aviso; logo geométrica; só fatos do mouse_m908 (GPL-3), sem copiar código.

## Achados
- A fonte (mouse_m908) usa o mesmo enquadramento de pacote, mas muda DPI,
  botões por perfil, macros e LED por modelo. Não dá para só trocar o PID.
- Capturas de referência: `documentation/default_m908.txt` e os `data.cpp`
  por modelo em `include/<modelo>/` do mouse_m908.
- `redragon.com.br/downloads` lista software de M711 King Cobra 2, Cobra V2,
  Storm Elite M988, ID711 e B703; o VID:PID deles é desconhecido (Trilha Q).

## Achados do REA (22/10/2026)
Software oficial analisado com `rea` 6.4 e Ghidra 12.1.4
(`GHIDRA_INSTALL_DIR=/opt/homebrew/Cellar/ghidra/12.1.4/libexec`; sem a variável
o `rea doctor` diz que o Ghidra não existe).
- Origem: links do `redragon.com.br/downloads`. King Cobra 2 = instalador Inno
  próprio (`RDCfg.exe`, Qt, `HidDevice.dll`, `HIDApi.dll`). Cobra V2 = pacote
  universal "REDRAGON Gaming Mouse" com a mesma base `RDCfg.exe` e uma pasta por
  modelo (IMPACT, INVADER, GRIFFIN, TIGER, M715, M721, LEGEND, RANGER, MAMMOTH,
  PERDITION, PERDITION3, M998, M808, TITANOBOA e outros). B703 = outro produto
  (`OemDrv.exe`, `Lowerdev.dll`), fora do escopo.
- O `RDCfg.exe` não tem VID/PID no código. Eles vêm do `Config.ini`
  (`[Mouse_List]`, com faixa de versão do dispositivo) e a escolha do modelo é
  por nome comparado em `FUN_0040e870`, que só monta a interface (posição de
  widgets). Ali não está o protocolo.
- Os `DefaultProfile.prof` são texto puro, mas todos têm o mesmo molde (5 DPI,
  42 botões, polling 2). Não diferenciam o protocolo por modelo.
- A camada que fala com o mouse está em `RDCfg.exe` (`0x456c40` a `0x45c910`, via
  hidapi). Protocolo e endereços são os do M901-1; muda por família (0 a 4) e por
  sensor. Resultado em `RE-FINDINGS.md`. Falta ler DPI por sensor, botões e macros.
- Arquivos de trabalho (não versionar, são do fabricante): scratchpad
  `.../vendor/{king-cobra2,cobra-v2}`.

## Bloqueios e dúvidas em aberto
- Sem nenhum desses mouses em mãos, tudo entra como `Untested`.
- Qual SKU exato é o "Cobra" da fonte (`fc30`) em relação ao King Cobra 2 e
  ao Cobra V2 do site.

## Como verificar
`verify .` e, para um modelo novo, o teste de montagem de pacote contra a captura.
