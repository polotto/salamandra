# Salamandra

[![CI](https://github.com/polotto/salamandra/actions/workflows/ci.yml/badge.svg)](https://github.com/polotto/salamandra/actions/workflows/ci.yml)

Configurador aberto de mouses gamer para Windows, Linux e macOS.

Ajusta LED, DPI, taxa de polling, perfis, botões e macros direto na memória do
mouse, sem depender do software do fabricante, que só existe para Windows.

> **English:** open source configurator for gaming mice on Windows, Linux and
> macOS. It writes lighting, DPI, polling rate, profiles, buttons and macros
> straight to the mouse memory. See the compatibility list below and
> [CONTRIBUTING.md](CONTRIBUTING.md). Issues and pull requests in English are
> welcome.

## Situação

Versão de teste (pré-release). Baixe o instalador do seu sistema em
[Releases](https://github.com/polotto/salamandra/releases) e veja
[Como instalar](#como-instalar).

## Compatibilidade

### Mouses

| Mouse | USB | Situação |
| --- | --- | --- |
| Redragon M901-1 Perdition 2 | `04d9:fc40` | suportado e testado |

Outros modelos ainda não são reconhecidos. Mouses com o mesmo chip podem ser
parecidos, mas cada um precisa ser mapeado e testado antes, porque gravar no
endereço errado pode deixar o mouse com comportamento inesperado. Tem outro
mouse e quer ajudar? Abra uma issue com o modelo pedindo suporte (veja
[CONTRIBUTING.md](CONTRIBUTING.md)).

Para saber o código USB do seu mouse: no Linux, `lsusb`; no macOS, Informações
do Sistema > USB; no Windows, Gerenciador de Dispositivos > propriedades do
mouse > Detalhes > IDs de hardware.

### Sistemas

| Sistema | Configurar o mouse | Atalhos executados no PC | Testado com o mouse |
| --- | --- | --- | --- |
| macOS 13 ou mais novo | sim | sim (pede permissão de Acessibilidade) | sim (Apple Silicon) |
| Windows 10 e 11 | sim | sim | ainda não |
| Linux | sim (precisa da regra udev) | só em X11; Wayland ainda não | ainda não |

"Ainda não" quer dizer que compila e passa nos testes automáticos, mas ninguém
testou com o mouse naquele sistema. Se você testou, conte numa issue.

### Funções do software oficial

| Função | Situação |
| --- | --- |
| 5 perfis e troca do perfil ativo | pronto |
| Taxa de polling (125, 250, 500, 1000 Hz) e velocidade de rolagem | pronto |
| DPI: 5 níveis de 100 a 24000, X e Y separados, nível ativo | pronto |
| Luz: fixo, respirar, piscar, desligado, cor, brilho e velocidade | pronto |
| Botões: cliques, tecla, combinação, mídia, macro, tiro rápido, DPI, perfil, polling, rolagem, modo do LED, desabilitar | pronto |
| Atalhos (copiar, colar, navegador, bloquear...), executados pelo PC | pronto |
| Macros: 15, até 63 ações, gravação do teclado com tempo real, edição | pronto |
| Salvar e carregar perfil em arquivo, backup, restaurar, padrão de fábrica | pronto |
| Ícone na bandeja e avisos do mouse na tela (perfil, DPI, polling) | pronto |
| Velocidade do ponteiro, aceleração e duplo clique (ajustes do sistema) | a fazer |

A lista completa, com onde cada função fica no mouse, está em
[docs/official-features.md](docs/official-features.md).

## Como instalar

Na página de [Releases](https://github.com/polotto/salamandra/releases), baixe:

| Sistema | Arquivo |
| --- | --- |
| Windows | `Salamandra_x.y.z_x64-setup.exe` ou `Salamandra_x.y.z_x64_en-US.msi` |
| macOS (Intel e Apple Silicon) | `Salamandra_x.y.z_universal.dmg` |
| Linux (Debian, Ubuntu) | `Salamandra_x.y.z_amd64.deb` |
| Linux (Fedora, openSUSE) | `Salamandra-x.y.z-1.x86_64.rpm` |
| Linux (outros) | `Salamandra_x.y.z_amd64.AppImage` |

Os instaladores não são assinados, então o sistema avisa na primeira vez:

- **Windows**: no aviso do SmartScreen, clique em "Mais informações" e
  "Executar assim mesmo".
- **macOS**: arraste o app para Aplicativos. Na primeira vez, abra Ajustes do
  Sistema > Privacidade e Segurança e clique em "Abrir Mesmo Assim". Se o
  macOS disser que o app está danificado, rode
  `xattr -dr com.apple.quarantine /Applications/Salamandra.app`.
- **Linux**: o `.deb` e o `.rpm` já instalam a regra udev. Com o AppImage,
  instale a regra à mão (veja abaixo) e dê permissão de execução ao arquivo.

## Como compilar

Precisa de [Rust](https://rustup.rs) estável e [Node.js](https://nodejs.org) 22
ou mais novo. No Linux, também das bibliotecas do Tauri e do hidapi:

```sh
sudo apt-get install libudev-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev
```

App:

```sh
cd app
npm install
npm run tauri dev                  # para testar
npm run tauri build                # instalador do seu sistema
```

Linha de comando:

```sh
cargo run -p salamandra-cli -- --help
cargo run -p salamandra-cli -- show
```

### Linux: acesso ao mouse sem root

Só para o AppImage ou para quem compilou (o `.deb` e o `.rpm` já fazem isso):

```sh
sudo cp packaging/linux/70-salamandra.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Depois tire e coloque o mouse de novo.

### macOS: atalhos

Para os atalhos (copiar, colar, navegador) funcionarem, dê ao Salamandra a
permissão em Ajustes do Sistema > Privacidade e Segurança > Acessibilidade.
Configurar o mouse não precisa de permissão.

## Antes de mexer: faça backup

Na tela Sobre do app, ou pela linha de comando:

```sh
cargo run -p salamandra-cli -- backup meu-mouse.json
cargo run -p salamandra-cli -- restore meu-mouse.json
```

## Por que "Salamandra"

Nas lendas europeias, a salamandra é a criatura que vive no fogo. Plínio, o
Velho, escreveu que ela atravessava as chamas sem se queimar, e Paracelso a pôs
como o espírito elemental do fogo. É um ser de fogo, vermelho como brasa, da
mesma família de lendas que os dragões, mas pequeno e sem dono.

O nome lembra o mundo dos dragões vermelhos sem usar a marca de ninguém.

## Contribuir

Correções, testes em outros sistemas e suporte a novos mouses são bem-vindos.
Leia [CONTRIBUTING.md](CONTRIBUTING.md) antes.

## Aviso

Projeto independente, sem vínculo com a Redragon nem com nenhum fabricante.
Os nomes de marcas e produtos citados pertencem aos seus donos e aparecem só
para dizer com quais mouses o projeto é compatível.

O protocolo de configuração foi documentado por engenharia reversa, para fins
de interoperabilidade. Este repositório não contém código, binários, imagens
nem textos do software oficial.

Use por sua conta e risco: gravar uma configuração errada pode deixar o mouse
com comportamento inesperado. Faça backup antes de alterar.

## Licença

MIT. Veja [LICENSE](LICENSE).
