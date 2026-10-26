# Ações customizadas

Troque o que cada atalho embutido do mouse faz no seu computador: combinação
de teclas, Mission Control, uma Hyper Key, abrir app ou URL, rodar comando ou
uma sequência com pausas. Inspirado no BetterTouchTool.

## Como funciona

O mouse só avisa o PC quando um dos 22 atalhos embutidos é apertado (Copiar,
Colar, Mostrar a área de trabalho...). Grave um deles num botão na aba
**Botões** e ligue-o a uma ação customizada no `custom-actions.json`. O serviço
em segundo plano executa a sua ação no lugar da embutida.

No app, use a aba **Ações**: escolha uma ação por atalho e salve. O arquivo fica na pasta de configuração do app (`custom-actions.json`) e vale na hora. Sequências se editam no arquivo.

Para rodar só o serviço com um arquivo:

```
SALAMANDRA_ACTIONS=custom-actions.json ./target/release/salamandra-service
```

Sem o arquivo, os atalhos embutidos seguem valendo. Arquivo inválido é
recusado com o motivo, e nada muda.

## Formato do arquivo

```json
{
  "format": "salamandra-actions",
  "version": 1,
  "bindings": {
    "ShowDesktop": { "type": "Builtin", "name": "MissionControl" },
    "Mail":        { "type": "Hyper" },
    "Find":        { "type": "Keys", "modifiers": ["Cmd", "Shift"], "key": "4" }
  }
}
```

As chaves de `bindings` são os nomes dos atalhos: `Cut`, `Copy`, `Paste`,
`SelectAll`, `Find`, `New`, `Print`, `Save`, `SwitchWindow`, `CloseWindow`,
`OpenFileManager`, `Run`, `ShowDesktop`, `Lock`, `BrowserHome`, `BrowserBack`,
`BrowserForward`, `BrowserStop`, `BrowserRefresh`, `BrowserSearch`,
`BrowserFavorites`, `Mail`.

## Ações

| `type` | Campos | O que faz |
| --- | --- | --- |
| `Keys` | `modifiers` (`Ctrl`, `Shift`, `Alt`, `Cmd`), `key` | Segura os modificadores e toca a tecla |
| `Builtin` | `name`: `MissionControl`, `Launchpad` (macOS), `ShowDesktop` | Ação pronta, resolvida por sistema |
| `Hyper` | nenhum | Segura Ctrl+Alt+Shift+Cmd enquanto o botão está apertado |
| `OpenApp` | `name` | Abre um app (`open -a` no macOS; no Windows e no Linux `name` é o programa, no PATH ou caminho completo) |
| `OpenUrl` | `url` | Abre com o programa padrão |
| `Command` | `argv` | Roda um programa com argumentos, sem shell |
| `Wait` | `ms` | Pausa, só dentro de uma `Sequence` (10 s no total) |
| `Sequence` | `steps` (1 a 64) | Roda os passos em ordem. Sem aninhar e sem `Hyper` |

`key` aceita um caractere, `f1` a `f12`, `space`, `tab`, `escape`, `enter`,
`backspace`, `delete`, `left`, `right`, `up`, `down`, `home`, `end`, `pageup`
e `pagedown`.

Com o `Hyper` segurado, qualquer ação `Keys` reaproveita os modificadores, então
Hyper mais uma tecla de outro botão funciona. As teclas são soltas quando o
botão é solto ou o mouse desconecta.

## Limites

- Só os 22 atalhos embutidos chegam ao PC. Clique comum e combinação de teclas
  gravada no mouse não chegam.
- O fim do Hyper usa o aviso genérico de "solto" do mouse. Segurar dois botões
  de atalho ao mesmo tempo não foi testado no físico.
- No Windows, o Hyper apertado e solto sem outra tecla dispara o atalho da tecla Office.
- O macOS exige a permissão de Acessibilidade. O Linux funciona em X11, não em
  Wayland.
