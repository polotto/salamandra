# Achados da engenharia reversa (REA + Ghidra)

Fonte: `RDCfg.exe` do pacote "REDRAGON Gaming Mouse" (site da Redragon, link da
Cobra V2), sha256 `c0fe791f...c8c703`, analisado com `rea` 6.4 e Ghidra 12.1.4.
Para interoperabilidade. Nada do oficial entra no repo: só estes fatos, cada um
com a função de origem. Atualizado em 22/10/2026.

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

## LED (`FUN_00456d90`, endereço `0x0449 + 8p`, len 7)
Mesmo endereço do M901-1, mas o modo e a velocidade mudam por família:
- famílias 0 a 2: modo em `local_30` (apagado 0, fixo 1, respirar 2, piscar 4,
  ...) com o byte de velocidade `9 - v`. Família 0 já é o que temos.
- família 3: outros códigos de modo (o byte de sub assume 4, 8, 2, 0 e 1 conforme o modo).
- família 4: o caminho de gravação é outro, com endereços `0x48`, `0x50`, `0x58`,
  `0x60`, `0x68` (uma gravação por zona ou perfil, ainda não confirmado).
Tabela completa de modos por família: pendente (R.4).

## Primeiro lote: o que já se sabe
| Modelo | Família | Sensor | Quanto reaproveita do M901-1 |
| --- | --- | --- | --- |
| M711 Cobra (`fc30`) | 0 | 3325 | quase tudo; muda só a tabela de DPI |
| INVADER (`fc4f`) | 2 | 3325 | LED e botões podem divergir; DPI de 3325 |
| IMPACT (`fc4d`) | 3 | 3327 | LED e botões da família 3 |
| GRIFFIN (`fc38`) | 3 | 3212 | família 3 com outra tabela de DPI |

Isso reordena o trabalho: **M711 Cobra** é o mais fácil e deve ir primeiro, e o
modelo do núcleo precisa ser `Familia` + `Sensor`, não um enum de mouses.

## Ainda não lido
- tabelas de DPI por sensor (dados em `DAT_004b9ddc`...`DAT_004ba29f`);
- botões por família (`FUN_00458530`, `FUN_00459cf0`, `FUN_0045c570`);
- macros por família (`FUN_00458f90`);
- de onde sai o nome do modelo (provavelmente lido do mouse) e a tabela
  VID/PID para nome.
