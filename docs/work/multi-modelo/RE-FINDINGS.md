# Achados da engenharia reversa (REA + Ghidra)

Fonte: `RDCfg.exe` do pacote "REDRAGON Gaming Mouse" (site da Redragon, link da
Cobra V2), sha256 `c0fe791f...c8c703`, analisado com `rea` 6.4 e Ghidra 12.1.4.
Para interoperabilidade. Nada do oficial entra no repo: só estes fatos, cada um
com a função de origem. Atualizado em 24/10/2026.

## Como o oficial escolhe o layout
O programa não decide por VID/PID. Ele guarda um **nome de modelo** (string,
`FUN_00423c30`) e deriva dois códigos dele:

- **família** (`FUN_00431a30`, retorna 0 a 4): define o layout de escrita (LED,
  botões...).
- **sensor** (`FUN_004314d0`, retorna 0xC20 a 0xD47): define a tabela de DPI.

Quase todo o protocolo é o mesmo do M901-1 (enquadramento `02 F3 addr_lo
addr_hi len`, `F1 02 m`, `F5`, mesmos endereços de rolagem `0x20`, polling
`0x32`/`0x38` e nível de DPI `base+2`). O que muda por família e sensor está
abaixo.

## Família por modelo (`FUN_00431a30`)
| Família | Modelos |
| --- | --- |
| 0 | PERDITION (M901), PERDITION3, M711 (Cobra, sem FPS), TIGER, MAMMOTH, 2825-A |
| 1 | 2535_3389 |
| 2 | INVADER, EMPEROR, 2776 3327, 2866 3327, 2903-1, 2713 3325, 2732 3327 |
| 3 | IMPACT, GRIFFIN, LEGEND, LEGEND2, TITANOBOA2/3/4, M711-FPS-1 (King Cobra 2), M711-FPS-2, 2710-G/H, RANGER, M998-RGB, M808-RGB, PHOENIX, M720-RGB, M721-PRO/RGB/RGB2, M912-RGB, SAMSARA2, INSPIRIT2, LAVAWOLF, 2722 3327, 2903 3389/3395/3399, 2913-A, 2805-P3333 |
| 4 | 2931-A V1 e mais um nome (`DAT_004b1b99`) |

O M901-1 já suportado é família 0 e sensor 0xD20. Bate com o que temos: a
codificação do DPI (`<= 0x79`) é a do ramo 0xD20 de `FUN_00457b30`.

## Sensor por modelo (`FUN_004314d0`, valor = modelo do sensor em decimal)
| Código | Sensor | Modelos relevantes |
| --- | --- | --- |
| 0xCFD (3325) | padrão | INVADER, M711 (Cobra), 2713 3325 |
| 0xCFF (3327) | | IMPACT, MAMMOTH, RANGER, PERDITION3, M808-RGB, EMPEROR, M912-RGB, SAMSARA2, INSPIRIT2 |
| 0xD20 (3360) | | PERDITION (M901), LEGEND, TITANOBOA2, M711-FPS-1 |
| 0xD3D (3389) | | M711-FPS-2, M998-RGB, LEGEND2, TITANOBOA4, 2710-G, 2535_3389 |
| 0xC8C (3212) | | GRIFFIN |
| outros | 0xD07, 0xC20, 0xD05, 0xD47, 0xD43 | M721-PRO, 2825-A, TITANOBOA3, ... |

Cada sensor tem tabela de DPI própria em `FUN_00457b30` (códigos de 8 bits
indexados pelo passo de 100 DPI, com limites 0x33, 0x5D, 0x79, 0x141, 0x190,
0x208 e outros).

## LED (`FUN_00456d90`, famílias 0 a 3: endereço `0x0449 + 8p`, len 7; a família 4 é outra)
Mesmo endereço do M901-1, mas o modo e a velocidade mudam por família:
- famílias 0 a 2: modo em `local_30` (apagado 0, fixo 1, respirar 2, piscar 4,
  ...) com o byte de velocidade `9 - v`. A família 0 já é o que temos, com o byte `sub` em aberto (seção abaixo).
- família 3: oito posições com outros pares (modo, sub), listadas abaixo na seção
  "Impact, LED". Velocidade `9 - v` em todas menos fixo e desligado.
- família 4: o caminho de gravação é outro: cinco gravações por zona, em `0x448`,
  `0x450`, `0x458`, `0x460` e `0x468` (seção "LED das famílias 1, 2 e 4").
Tabela de modos da família 3: feita (ver abaixo). Famílias 1 e 2: lidas em 24/10/2026, com
divergência de `sub` em aberto (abaixo). Família 4: lida, sem modelo para testar (abaixo).

## LED das famílias 1, 2 e 4, 24/10/2026
- Famílias 0, 1 e 2 caem no mesmo ramo de `FUN_00456d90` (`uVar4 < 3`), então a 1 e a
  2 têm o mesmo layout da 0, e o Invader (família 2) usa `LedCodec::Family0`. As
  posições de modo do oficial (0 a 8) viram (modo, sub) assim: 0 `2/1` respirar,
  1 `3/1`, 2 `1/1` fixo (sem velocidade), 3 `5/1`, 6 `4/1` piscar, 7 `0/0` desligado,
  8 `2/3`; velocidade `9 - v`.
  **Divergência, dúvida em aberto:** o nosso `Family0` grava sub `0` em fixo,
  respirar e piscar (como o M901-1, que foi testado no mouse), e este ramo do
  oficial grava sub `1`. O `Cycle` (`03 .. 03`) também não bate com nenhuma posição
  daqui (`3/1` e `2/3`). O firmware provavelmente ignora esse byte nos modelos de
  família 0, mas não está provado. Não mudamos o código: o que foi testado no M901-1
  continua valendo. Quem testar um Cobra ou Invader pode conferir se sub `1` muda
  algo. A leitura também só aceita sub `0`, então um perfil gravado pelo oficial com
  sub `1` aparece como fixo no Salamandra.
- Família 4 (só "2931-A V1" e mais um nome, `DAT_004b1b99`): o LED não é um bloco de
  7 bytes em `0x449 + 8p`. São cinco gravações de 8 bytes, uma por zona, em `0x448`,
  `0x450`, `0x458`, `0x460` e `0x468`, com dados `[0x81 (0x80 na zona 2), r, g, b,
  modo da zona, 9 - velocidade, flag, brilho]` e a quinta zona toda zero com `0x80`.
  Nenhum modelo do primeiro lote é família 4: não implementado, sem como testar.
- `FUN_00458530` também grava blocos de 8 bytes nos endereços de zona (ramos para as
  famílias 3 e 4); o papel dela no fluxo do oficial não foi identificado, então não é
  "botões" como se supunha. `FUN_00458f90` não é macro: só envia `F1 02 m` com
  m = `04`, `01`, `02`, `08`, `10` em sequência (parte do envelope de sessão, já
  coberto).

## Macros e botões por modelo, 24/10/2026
- Macros (`FUN_00459cf0`): grava em `0x478 + 200 * slot`, 200 bytes, 3 bytes por
  evento com terminador `06 00 01`. Não há ramo por família nem por nome de modelo:
  o layout é o do M901-1 para todos. Não explica o limite de 1 macro do Invader (vem
  só do autor do mouse_m908).
- Botões (`FUN_0045c570`): o endereço é `0x42 + base do perfil + 4 * slot`. O oficial
  troca o índice da interface pelo slot **conforme o nome do modelo**. IMPACT cai no
  padrão (slot = índice), como no M901-1. Os índices abaixo contam a partir de 0, como o código. INVADER: índice 3 vai para o
  slot 5 (`0x14`), 4 para o slot 3 (`0xc`) e 5 para o slot 4 (`0x10`); nas docs de
  modelo, que contam a partir de 1, são os botões 4, 5 e 6 indo para os slots 6, 4 e 5; os outros seguem o
  padrão. GRIFFIN: índice 6 vai para o slot 7 (`0x1c`). LEGEND e LEGEND2 deslocam os
  índices 7 a 0x16 em +1 slot (índice 7 vai para `0x20`, o slot 8), e esses índices
  passam dos 20 botões do mapa do M901-1 (0 a 19): é outro mouse, fora do nosso lote. Isso só muda qual botão da interface do oficial é qual
  slot; o Salamandra numera por slot, então a gravação não muda, mas o nome do botão
  na carcaça do Invader pode estar trocado entre os slots 3, 4 e 5 (a conferir).

## Primeiro lote: o que já se sabe
| Modelo | Família | Sensor | Quanto reaproveita do M901-1 |
| --- | --- | --- | --- |
| M711 Cobra (`fc30`) | 0 | 3325 | quase tudo; muda só a tabela de DPI |
| INVADER (`fc4f`) | 2 | 3325 | LED como a família 0 (`sub` em aberto); botões a confirmar; DPI de 3325 |
| IMPACT (`fc4d`) | 3 | 3327 | LED e botões da família 3 |
| GRIFFIN (`fc38`) | 3 | 3212 | família 3 com outra tabela de DPI |

Isso reordena o trabalho: **M711 Cobra** é o mais fácil e deve ir primeiro, e o
modelo do núcleo precisa ser `Familia` + `Sensor`, não um enum de mouses.

## Ainda não lido
- tabelas de DPI por sensor (dados em `DAT_004b9ddc`...`DAT_004ba29f`);
- botões e macros: lidos (seção "Macros e botões por modelo"); falta o formato dos
  bytes de cada ação por família em `FUN_0045c570` (acima de `0x45cf89`);
- o papel de `FUN_00458530` no fluxo do oficial;
- de onde sai o nome do modelo (provavelmente lido do mouse) e a tabela
  VID/PID para nome.

## Sensor 3325 (M711 Cobra, INVADER), 23/10/2026
- `FUN_00457b30`, ramo `0xCFD`: nível de DPI gravado em `base+4+6n`, 5 bytes
  (ativo, X, flag X, Y, flag Y). Índice = DPI/100 até 5000 e `50 + (DPI-5000)/200`
  acima. Índice < 51: código `T[índice]`, flag 0. Índice >= 51: `T[índice-25]`,
  flag 1. Tabela `T` (51 bytes em `DAT_004ba29f`): 0, 2, 4, 6, 8, 11, 13, 15,
  18, 20, 22, 25, 27, 29, 32, 34, 36, 39, 41, 43, 46, 48, 50, 52, 55, 57, 59,
  62, 64, 66, 69, 71, 73, 76, 78, 80, 83, 85, 87, 90, 92, 94, 97, 99, 101,
  104, 106, 108, 111, 113, 115.
- Conferido contra a captura do M711 no mouse_m908: os 75 pares batem.
- A captura do M711 mostra 10 botões nas posições 0 a 7, 10 e 11 (`0x82 + 4n`).
- Ramos de outros sensores já lidos: `0xCFF` (3327), `0xD20` (3360, o do M901-1),
  `0xD3D`, `0xD47`, `0xD43`, `0xC8C` (3212, Griffin), `0xC20`, `0xD05`, `0xD07`.
  Falta extrair as tabelas do `0xCFF` (`DAT_004ba0cc`, `DAT_004ba120`,
  `DAT_004ba19c`) e do `0xC8C` para o Impact e o Griffin.

## Sensor 3327 (M908 Impact), 23/10/2026
- `FUN_00457b30`, ramo `0xCFF`: índice 0..61 usa `DAT_004ba19c` (flag 0), 62..92 usa
  `DAT_004ba120` (flag 1). DPI do índice: `(i+1)*100` até 6200 (i=61), depois
  `6400 + 200*(i-62)` até 12400 (i=92). Os 92 códigos de 200 a 12400 batem com a
  captura do mouse_m908; o índice 0 (100 DPI, código 0) só existe no oficial.
- Depois do índice 92 o oficial lê mais 21 entradas (`DAT_004ba0cc`) que repetem
  códigos já usados; fora do que sabemos explicar, não usadas.
- O Impact usa o mesmo mapa de botões do M901-1 (20 posições, mesmos endereços),
  conforme a captura do mouse_m908.
- LED família 3 (`FUN_00456d90`), resolvido em 24/10/2026: os 2 bytes em `0x446` só
  ganham valor para o modelo "2850"; nos demais (inclusive IMPACT) vão `00 00`, igual
  ao pacote da captura do mouse_m908. A velocidade é `9 - v` como nas famílias 0 a 2.
  As 8 posições da interface são (modo, sub): 0 `1/4` respirar, 1 `1/8` arco-íris,
  2 `1/2` fixo (velocidade 0), 3 `2/0` onda, 4 `6/0` alternado, 5 `7/0` reativo,
  6 `1/0x10` piscar, 7 `0/0` desligado. Todas gravam a velocidade `9 - v`, menos
  fixo e desligado. Depois grava `0x449 + 8p`.
  Correção das notas acima: o valor `sub 1` (respirar arco-íris) aparece só na
  captura do mouse_m908, não nas 8 posições do oficial; a nota "4, 8, 2, 0 e 1" estava
  incompleta e o piscar é `0x10`. O `00 00` em `0x446` o oficial grava em todas as
  famílias, mas só gravamos na 3: o M901-1 e o Cobra (família 0) ficam sem, porque o
  M901-1 foi testado assim e o mouse já tem zeros ali. A captura do
  mouse_m908 dá o par `(modo, sub)`: apagado `00 00`, fixo `01 02`, respirar
  `01 04`, arco-íris `01 08`, piscar `01 10`, respirar arco-íris `01 01`, onda
  `02 00`, reativo ao botão `03 00`, aleatório `04 00`, alternado `06 00`,
  reativo `07 00`. Byte de velocidade entre os dois.

## M719 Invader e M607 Griffin, 23/10/2026
- Invader (família 2, sensor 3325): os 75 códigos de DPI da captura do mouse_m908
  são exatamente os do sensor 3325 do oficial. Botões nas posições 0 a 7, 10 e 11,
  como no Cobra. O autor do mouse_m908 só conseguiu 1 macro.
- Griffin (sensor 3212, ramo `0xC8C`): o oficial usa aritmética (código = índice
  abaixo de 0x40, `índice - 0x20` com flag 1 de 0x40 a 0x5E, `índice - 0x3F` com
  flag 2 acima), mas a captura do mouse_m908 dá códigos iguais aos do sensor 3325
  (100 -> 2, 900 -> 20, 6400 -> 73 flag 1, máximo 7200). Divergência: **dúvida em
  aberto**, o Griffin não entra até alguém com o mouse resolver.
- A captura do mouse_m908 codifica o LED do Cobra e do Invader diferente do
  oficial (par modo/sub genérico). Seguimos o modo do oficial com o `sub 0` testado no M901-1 (divergência de `sub` em
  "LED das famílias 1, 2 e 4").
