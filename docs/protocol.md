# Protocolo do M901-1 Perdition 2 (`04d9:fc40`)

Fonte: engenharia reversa, para interoperabilidade, do software oficial:
`hid.exe` (comunicação com o mouse) e `ldcfg.exe` (interface), com REA 6.4 e
Ghidra 12.1.4. Cada linha cita a função de origem. Conferido por leitura no
mouse real em 11/10/2026. O mesmo firmware (Holtek) aparece no
[mouse_m908](https://github.com/dokutan/mouse_m908), que serviu de segunda fonte
para o código de polling.

## Dispositivo

- USB `04d9:fc40`, produto `USB Gaming Mouse` (`FUN_00410fe0`)
- Interface de configuração: usage page `0xFFA0`, usage `0x01` (3ª interface HID)
- Só Feature Reports. No macOS o buffer inclui o report ID no byte 0
  (`IOHIDDeviceSetReport`/`GetReport`, `kIOHIDReportTypeFeature`)
- Report `0x02` = 16 bytes; report `0x03` = 64 bytes (do descritor HID)
- Não precisa de permissão de Input Monitoring

## Pacote

```
byte  0     1    2        3        4    5..7  8..
      rid   cmd  addr_lo  addr_hi  len  0     dados
```

| cmd | uso | origem |
| --- | --- | --- |
| `F3` | escreve `len` bytes em `addr` | `FUN_0040e9f0` e outras |
| `F2` | pede leitura de `len` bytes; a resposta vem no GetFeature do mesmo report: `rid 08 addr_lo 60 len 00 FA FA dados...` | `FUN_0040f510` |
| `F5 00` / `F5 01` | abre e fecha a sessão de escrita | `FUN_0040f1f0`, chamado por `FUN_00412e90` |
| `F1 02 <m>` | aplica uma área: `01` perfil, `02` DPI, `04` botões, `08` polling, `10` nível de DPI | `FUN_004102c0`, `FUN_0040ff40`, `FUN_004103c0`, `FUN_004100a0`, `FUN_00412e90` |

Escrita completa do oficial (`FUN_00412e90`, "SendAll2Mouse"): `F5 00`, escritas,
`F1 02 04`, `F1 02 01`, `F1 02 02`, `F1 02 08`, `F5 01`.

Armadilha: depois de gravar o perfil ativo (`0x2C`), o mouse passa a usar os
botões do perfil novo na hora, mas o LED só troca na próxima sessão `F5`.
Conferido no hardware: a troca de perfil e o restore fazem uma sessão a mais por isso.

## Mapa de memória

Perfis `p = 0..4`, bases `0x040 0x100 0x1B0 0x260 0x310` (tabela em `0x47E35C`).

| Endereço | Tamanho | Conteúdo | Origem |
| --- | --- | --- | --- |
| `0x002C` | 2 | perfil ativo (0..4), `00` | `FUN_004102c0` |
| `0x0020 + 2p` | 1 (+0) | linhas de rolagem (`WheelScrollLines`), report 3 | `FUN_004101e0` |
| `0x0032 + 2p` | 1 (+0) | polling: `08`=125 `04`=250 `02`=500 `01`=1000 Hz | `FUN_004103c0`, mouse_m908 |
| `0x003E` | 2 | sempre `00 00` | `FUN_0040e960` |
| `0x0446` | 2 | sempre `00 00` | `FUN_0040e9f0` |
| `base + 2` | 1 (+0) | nível de DPI ativo (0..4) | `FUN_004100a0`, `FUN_0040fc50` |
| `base + 4 + 6i` | 5 (+0) | nível `i`: `ativo, x, xf, y, yf` | `FUN_0040ff40` |
| `base + 0x42 + 4b` | 4 | ação do botão `b` (slots `0x12`, `0x13` vão para `0x16`, `0x17`) | `FUN_00412980` |
| `0x0449 + 8p` | 6 | LED: `R G B modo velocidade sub` | `FUN_0040e9f0`, `FUN_0040fa00` |
| `0x044F + 8p` | 1 | brilho do LED (1..3) | `FUN_0040e9f0` |

### DPI

`índice = DPI / 100` (100 a 24000). Codificação (`FUN_0040ff40`):
`índice <= 0x79`: `byte = índice - 1`, `flag = 0`. Senão `byte = índice - 0x3D`,
`flag = 1`. Padrão de fábrica (`RSProfile1.pfd`): 1000, 2000, 4000, 8000, 12000.

### LED (modo da UI -> firmware)

| UI | modo | velocidade | sub |
| --- | --- | --- | --- |
| fixo | `01` | 0 | 0 |
| respirar | `02` | 1..8 | 0 |
| piscar | `04` | 1..8 | 0 |
| desligado | `00` | 0 | 0 |
| `4` (sem nome na UI) | `03` | 0 | `03` |

### Botões (`base + 0x42 + 4b`, tabela `0x47D3D0`)

| Ação | Bytes |
| --- | --- |
| clique esquerdo / direito / meio | `81` / `82` / `83` |
| voltar / avançar | `84` / `85` |
| DPI e perfil (vários) | `88`, `89`, `8A`, `8B`, `8C`, `8D` |
| tecla | `90 00 <uso HID> 00` |
| combinação | `8F <mods> <uso HID> 00`, mods `ctrl=1 shift=2 alt=4 win=8` |
| mídia | `8E 01 <uso consumer>` (`CD` play, `B5`/`B6` faixa, `E9`/`EA` volume, `E2` mudo, `B7` stop) |
| atalho embutido | `8E 01 FF <id>` |
| tiro rápido | `99 <alvo> <vezes> <atraso/10 ms>`; alvo `81`/`82`/`84` = L/R/M, outro valor = uso HID da tecla (`FUN_0040f5e0`) |
| macro | `91 <índice, bit7 = repetir> <cont_lo> <cont_hi>` |
| desabilitado | `00 00 00 00` |

## Macros (`0x478 + 200i`, 15 slots)

Gravadas com report `0x04` (256 bytes): `04 F3 <addr> C8 00 00 00 <200 bytes>`,
entre `F5 00` e `F5 01`, sem `F1` (`FUN_004125b0`). Até 63 eventos de 3 bytes
(o editor oficial guarda 189 bytes); o resto é zero.

| Evento | Bytes | Origem |
| --- | --- | --- |
| tecla aperta / solta | `84 <uso HID> 00` / `04 <uso HID> 00` | `ldcfg.exe` `FUN_00427030` |
| botão do mouse aperta / solta | `81 <b> 00` / `01 <b> 00`, b: `1` L, `2` R, `4` M | `ldcfg.exe` `FUN_0042a600` |
| atraso | `06 <lo> <hi>`, tempo em unidades de 10 ms (16 bits), mínimo 10 ms | `ldcfg.exe` `FUN_00427030` |

Conferido com as macros gravadas no mouse real (Shift duas vezes; Win+V).

## Eventos que o mouse envia ao PC

O `hid.exe` escuta os reports de entrada da interface `0xFFA0` (`FUN_00410be0`).

| Report | Significado | O oficial faz |
| --- | --- | --- |
| `08 01 <p>` | perfil ativo mudou pelo botão do mouse | troca o perfil na tela |
| `08 02 <n>` | nível de DPI mudou (n de 0 a 4; conferido no mouse: `08 02 04`, `08 02 03`) | mostra o nível |
| `08 06 <código>` | polling mudou (`1/2/4/8`) | atualiza a tela |
| `08 09 …` | modo do LED mudou | atualiza a tela |
| `01 FF <id>` e depois `01 00 00` | botão com atalho embutido (`8E 01 FF <id>`) apertado e solto; vem no report de consumidor (ID 1). Conferido no mouse: `01 ff 12` para copiar | executa no PC (tabela abaixo) |

Atalhos executados pelo PC (`hid.exe` `FUN_00401050` em diante): `0x11`
Ctrl+X, `0x12` Ctrl+C, `0x13` Ctrl+V, `0x14` Ctrl+A, `0x15` Ctrl+F, `0x16`
Ctrl+N, `0x17` Ctrl+P, `0x18` Ctrl+S, `0x19` Alt+Tab, `0x1A` Alt+F4, `0x1B`
abrir gerenciador de arquivos, `0x1C` executar, `0x1D` mostrar mesa, `0x1E`
bloquear, `0x1F` página inicial do navegador, `0x20`..`0x25` teclas de
navegador (voltar, avançar, parar, recarregar, buscar, favoritos), `0x26`
cliente de e-mail.

## Padrões de fábrica

Dos perfis do instalador, conferidos com o mouse: 500 Hz, 1 linha de rolagem,
DPI 1000/2000/4000/8000/12000 todos ligados com o nível ativo do perfil `p`
igual a `p`, LED fixo brilho 2, cores vermelho/azul/verde/magenta/amarelo.
Botões: L, R, M, tiro rápido L 3x, DPI+, DPI-, teclas 1..0, `+` e `-` do
teclado numérico, rolar para cima e para baixo. Ver `crates/salamandra-core/src/defaults.rs`.
