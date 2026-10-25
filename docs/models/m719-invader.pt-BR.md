# Redragon M719 Invader (`04d9:fc4f`)

Situação: **não testado**. Nunca foi conferido no mouse físico. O app e o CLI gravam direto, com aviso.

## De onde vem cada campo
| Campo | Valor | Origem |
| --- | --- | --- |
| Família / sensor | 2 / 3325 (`0xCFD`) | software oficial `RDCfg.exe`, `FUN_00431a30` e `FUN_004314d0` |
| Mapa de memória | igual ao do M901-1 | captura do `mouse_m908` para este modelo (só endereços) |
| Botões | 10 por perfil, posições 1 a 8, 11 e 12, como no M711 Cobra | captura do `mouse_m908` |
| DPI | os mesmos 75 valores do M711 Cobra (100 a 5000 de 100 em 100, 5200 a 10000 de 200 em 200) | tabela do oficial para o sensor `0xCFD`; os 75 códigos da captura do `mouse_m908` deste modelo batem |
| LED | mesma codificação do M901-1 (família 0) | software oficial: as famílias 0 a 2 compartilham |
| Macros | 1 posição | o autor do `mouse_m908` escreveu que o modelo "parece aceitar só uma macro" |

## Ainda sem confirmar
- O gravador de macros do oficial (`FUN_00459cf0`) não tem ramo por família nem por modelo, então o layout é o do M901-1. O limite de 1 posição vem só do autor do `mouse_m908`; por isso só a posição 1 é oferecida até alguém testar mais.
- O oficial leva os botões 4, 5 e 6 para os slots 6, 4 e 5 da numeração desta página (a partir de 1; offsets `0x14`, `0xc` e `0x10`) neste modelo (`FUN_0045c570`). Numeramos por slot, então a gravação não muda, mas o nome desses três botões na carcaça pode estar trocado. Confira qual botão físico é cada um.
- A captura do `mouse_m908` usa para este modelo uma codificação de LED diferente da do software oficial. Seguimos o modo do oficial, com o byte `sub` com que o M901-1 foi testado (o programa oficial multimodelo (`RDCfg.exe`) grava `1` ali nas famílias 0 a 2; ver as notas de engenharia reversa).

## Como ajudar
Abra o formulário "Suporte a um mouse" com a saída de `salamandra show` antes e depois de uma mudança e o que você viu no mouse.
