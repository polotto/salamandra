# Funções do software oficial

Definição de "100%" para a v1. Fonte: textos da interface do instalador oficial
(telas e opções), e o código de `hid.exe` (comunicação) e `ldcfg.exe`
(interface), lidos por engenharia reversa. Cada linha diz onde a função mora.

Onde: **mouse** = memória do mouse (`docs/protocol.md`); **PC** = executado
pelo programa no computador; **SO** = ajuste do sistema operacional.

| Tela | Função | Onde | Situação no Salamandra |
| --- | --- | --- | --- |
| Perfis | 5 perfis, trocar o ativo | mouse `0x2C` | núcleo pronto |
| Perfis | salvar perfil em arquivo, carregar arquivo | PC | pronto (JSON próprio, validado ao carregar) |
| Perfis | restaurar perfil atual para fábrica | mouse | pronto (app: carrega no editor e grava) |
| Perfis | resetar todos os perfis | mouse | pronto (app, com confirmação) |
| Geral | taxa de polling 125/250/500/1000 Hz | mouse `0x32 + 2p` | núcleo pronto |
| Geral | velocidade de rolagem (linhas por passo) | mouse `0x20 + 2p` | núcleo pronto |
| Geral | velocidade do ponteiro | SO | a fazer (Windows, macOS, Linux) |
| Geral | aceleração | SO | a fazer |
| Geral | velocidade do duplo clique | SO | a fazer |
| DPI | 5 níveis, ligar e desligar cada um | mouse `base + 4 + 6i` | núcleo pronto |
| DPI | X e Y separados ("link XY") | mouse | núcleo pronto |
| DPI | nível ativo | mouse `base + 2` | núcleo pronto |
| Luz | fixo, respirar, piscar | mouse `0x449 + 8p` | núcleo pronto |
| Luz | cor da paleta e cor personalizada | mouse | núcleo pronto |
| Luz | brilho baixo, médio, alto | mouse `0x44F + 8p` | núcleo pronto |
| Luz | velocidade de respirar e de piscar (baixa, média, alta) | mouse | núcleo pronto |
| Botões | clique esquerdo, direito, meio, avançar, voltar | mouse | núcleo pronto |
| Botões | tecla única | mouse | núcleo pronto |
| Botões | combinação (Win, Ctrl, Shift, Alt + tecla) | mouse | núcleo pronto |
| Botões | básicas: recortar, copiar, colar, selecionar tudo, buscar, novo, imprimir, salvar | PC (mouse avisa, PC executa) | serviço pronto (`salamandra-service`); testado no macOS |
| Botões | avançadas: trocar janela, fechar janela, abrir gerenciador de arquivos, executar, mostrar mesa, bloquear, navegador (início, voltar, avançar, parar, recarregar, buscar, favoritos), e-mail | PC | serviço pronto; Linux só X11 |
| Botões | mídia: play/pausa, parar, anterior, próxima, volume +/-, mudo, player | mouse | núcleo pronto ("player" a mapear) |
| Botões | macro, com repetições ou repetir enquanto segura | mouse | núcleo pronto |
| Botões | tiro rápido: botão L/R/M ou tecla, vezes, atraso | mouse | núcleo pronto |
| Botões | DPI: ciclo, +, - | mouse | núcleo pronto |
| Botões | trocar perfil | mouse | núcleo pronto |
| Botões | polling +, - | mouse | núcleo pronto |
| Botões | rolar para cima, para baixo, trocar modo do LED | mouse | núcleo pronto |
| Botões | desabilitar | mouse | núcleo pronto |
| Botões | regra: pelo menos um clique esquerdo | PC (validação) | núcleo pronto |
| Macros | até 15 macros, 63 ações cada | mouse `0x478 + 200i` | núcleo pronto |
| Macros | gravar teclado com tempo real entre ações | PC (gravação) + mouse | pronto (app) |
| Macros | inserir atraso padrão, ou gravar sem atraso | PC + mouse | pronto (app) |
| Macros | ações de mouse: L, R, M (apertar e soltar) | mouse | núcleo pronto |
| Macros | editar: novo, apagar, gravar depois, mover, copiar, recortar, colar | PC | pronto (app) |
| Macros | nome da macro | PC (só no computador) | pronto (app) |
| Bandeja | ícone na bandeja, abrir configuração, sair | PC | a fazer |
| Bandeja | refletir na tela o que muda pelos botões do mouse (perfil, DPI, polling, LED) | PC (report `0x08`) | eventos prontos no serviço; falta a tela |
| Info | versão do software, modelo | PC | pronto (tela Sobre) |

Fora do oficial e fora da v1: atualização de firmware, versão do firmware (o
oficial mostra só a versão do programa).
