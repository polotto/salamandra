# Redragon M908 Impact (`04d9:fc4d`)

Situação: **não testado**. Nunca foi conferido no mouse físico. O app e o CLI gravam direto, com aviso.

## De onde vem cada campo
| Campo | Valor | Origem |
| --- | --- | --- |
| Família / sensor | 3 / 3327 (`0xCFF`) | software oficial `RDCfg.exe`, `FUN_00431a30` e `FUN_004314d0` |
| Mapa de memória | igual ao do M901-1: bases de perfil, DPI em `base+4+6n`, polling em `0x32`, LED em `0x449`, macros em `0x478+200n` | captura do `mouse_m908` para este modelo (só endereços) |
| Botões | 20 por perfil, mesmas posições e endereços do M901-1 (as duas posições da roda em `0x16` e `0x17`) | captura do `mouse_m908` |
| DPI | 93 valores: 100 a 6200 em passos de 100, 6400 a 12400 em passos de 200. Um byte de código mais uma flag por eixo | tabelas do oficial (`FUN_00457b30`, sensor `0xCFF`); os códigos de 200 a 12400 batem com a captura do `mouse_m908`, o de 100 DPI só existe na tabela do oficial |
| LED | `r g b modo vel sub`: apagado `0 0 0`, fixo `1 0 2`, respirar `1 v 4`, arco-íris `1 v 8`, onda `2 v 0`, alternado `6 v 0`, reativo `7 v 0`, piscar `1 v 0x10`. `v` é o byte do firmware, `9 -` o valor da interface, como no M901-1 | as oito posições da família 3 em `FUN_00456d90`; os nomes vêm da captura do `mouse_m908`, que concorda em todos os pares |
| Grupo do LED | antes do LED gravamos `00 00` em `0x446` | `FUN_00456d90` grava, com zeros para este modelo (valor diferente de zero só para outro modelo, "2850"); a captura do `mouse_m908` grava o mesmo pacote |

## Diferenças para o M901-1
- Tabela e faixa de DPI.
- Bytes do modo de LED (família 3).

## Ainda sem confirmar
- Fixo: o oficial grava velocidade `0`, a captura do `mouse_m908` grava `8`. Seguimos o oficial. O firmware provavelmente ignora.
- Modos de LED vistos só na captura do `mouse_m908`, que o oficial não oferece: respirar arco-íris (`1 v 1`), reativo ao botão (`3 v 0`) e aleatório (`4 v 0`). Não oferecemos.
- A tabela do oficial tem 21 entradas depois de 12400 que não entendemos. Ficamos em 12400.

## Como ajudar
Abra o formulário "Suporte a um mouse" com a saída de `salamandra show` antes e depois de uma mudança e o que você viu no mouse (LED, DPI, botões).
