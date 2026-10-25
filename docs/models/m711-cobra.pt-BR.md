# Redragon M711 Cobra (`04d9:fc30`)

Situação: **não testado**. Nunca foi conferido no mouse físico. A gravação
funciona como no M901-1, então o app e o CLI gravam direto, com aviso.

## De onde vem cada campo
| Campo | Valor | Origem |
| --- | --- | --- |
| Família / sensor | 0 / 3325 (`0xCFD`) | software oficial `RDCfg.exe`, `FUN_00431a30` e `FUN_004314d0` |
| Mapa de memória | igual ao do M901-1: bases de perfil, DPI em `base+4+6n`, botões em `base+0x42+4n`, LED em `0x449`, polling em `0x32`, macros em `0x478+200n` | captura do `mouse_m908` para este modelo (só endereços) e o software oficial, que usa o mesmo código para a família 0 |
| DPI | 75 valores: 100 a 5000 em passos de 100, 5200 a 10000 em passos de 200. Um byte de código mais uma flag por eixo | tabela do oficial (`FUN_00457b30`, sensor `0xCFD`); os 75 códigos batem com a captura do `mouse_m908` |
| Botões | 10 por perfil, posições 1 a 8, 11 e 12 (a captura não tem as posições 9 e 10) | captura do `mouse_m908` |
| Macros | 15 posições de 200 bytes, mesmos endereços | captura do `mouse_m908` |

## Diferenças para o M901-1
- Tabela e faixa de DPI (acima).
- Só 10 botões.

## Ainda sem confirmar
- O nome de cada um dos 10 botões na carcaça.
- Modos de LED: o app mostra os do M901-1. A família 0 tem o mesmo layout no código oficial, mas não foi conferido no mouse.
- A captura do `mouse_m908` usa para este modelo uma codificação de LED diferente da do oficial. Seguimos o modo do oficial (família 0) com o byte `sub` testado no M901-1; o programa oficial multimodelo (`RDCfg.exe`) grava `1` ali, dúvida em aberto.
- O oficial atual grava 5 bytes por nível de DPI, a captura antiga 4. Gravamos 5, como o oficial atual.
- Versão do dispositivo: o mesmo PID `fc30` cobre outros modelos com outros sensores (M711-FPS-1, M711-FPS-2, 2710-H). Esta spec só bate com a lista do oficial nas versões 1 a 767. O `salamandra show` imprime a versão; inclua no relato.

## Como ajudar
Abra o formulário "Suporte a um mouse" com a saída de `salamandra show` antes e depois de uma mudança e o que você viu no mouse (LED, DPI, botões).
