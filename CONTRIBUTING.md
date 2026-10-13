# Como contribuir

[English](CONTRIBUTING.en.md) | **Português**

Obrigado pelo interesse. Este guia diz como montar o ambiente, o que conferir
antes de abrir um pull request e como pedir ou adicionar suporte a um mouse.

## Formas de ajudar

- **Testar em Windows ou Linux** com um mouse suportado e contar o resultado
  numa issue: o que funcionou, o que não, sistema e versão.
- **Reportar bug**: passos para reproduzir, sistema, modelo do mouse e, se
  possível, a saída de `salamandra show`.
- **Pedir suporte a um mouse**: abra uma issue com o modelo, o código USB
  (`vendor:product`) e o link do software oficial.
- **Código**: escolha uma task aberta em
  `docs/work/v1-multiplataforma/TASKS.md` ou uma issue, e comente antes de
  começar algo grande.
- **Tradução**: os textos ficam em `app/src/lib/i18n.svelte.ts`.

## Ambiente

Precisa de Rust estável e Node.js 22 ou mais novo. No Linux, as bibliotecas
listadas no [README](README.md#como-compilar).

```sh
cargo test --workspace                                   # testes do núcleo
cargo clippy --workspace --all-targets -- -D warnings    # lint
cargo fmt --all                                          # formatação
cd app && npm install && npm run check                   # front
node app/scripts/check-i18n.mjs                          # textos sem tradução
cd app && npm run tauri dev                              # app
```

O CI roda tudo isso em Windows, Linux e macOS a cada push e pull request.

## Organização do código

| Pasta | O que tem |
| --- | --- |
| `crates/salamandra-core` | protocolo, mapa de memória, modelos e a API `Mouse`. Sem I/O; testa sem mouse |
| `crates/salamandra-hid` | transporte USB HID por `hidapi` |
| `crates/salamandra-service` | serviço que escuta o mouse e executa os atalhos no PC |
| `crates/salamandra-cli` | linha de comando `salamandra` |
| `app` | app Tauri 2 com Svelte 5 |
| `docs/protocol.md` | protocolo do M901-1, cada campo com a origem |
| `docs/official-features.md` | funções do software oficial e situação de cada uma |

## Regras importantes

**Segurança do mouse.** Gravar no endereço errado pode deixar o mouse com
comportamento inesperado.

- Só grave endereços e tamanhos documentados em `docs/protocol.md`.
- Nunca deixe um perfil sem nenhum botão com clique esquerdo (o núcleo já
  recusa).
- Toda montagem de pacote é função pura no `salamandra-core` e tem teste.
- Antes de testar escrita num mouse de verdade, faça backup:
  `salamandra backup meu-mouse.json`.

**Nada do software oficial entra no repositório.** Nada de código decompilado,
binários, instaladores, imagens, ícones, arquivos `.ini` ou textos copiados.
Fatos sobre o protocolo (endereços, códigos de comando, padrões de fábrica)
entram como dados nossos, descritos com palavras nossas e com a origem citada.
Por isso o projeto também não usa nomes de marca em código, pacotes ou
binários: a marca só aparece para dizer com quais mouses ele é compatível.

**Textos da interface** sempre por `t("texto em pt-BR")`, com a tradução em
inglês no dicionário. A checagem de i18n falha se faltar.

**App sem cara de página web**: nada de seleção de texto fora de campos,
Cmd/Ctrl+A, menu de clique direito ou atalhos de navegador
(`app/src/lib/nativeFeel.ts`).

## Adicionar suporte a um mouse

1. **Identifique o mouse**: código USB e as interfaces HID. Em geral a de
   configuração tem usage page de fabricante (`0xFF00` a `0xFFFF`).
2. **Procure o que já existe**: projetos abertos do mesmo chip costumam ter o
   protocolo, ou parte dele.
3. **Estude o software oficial** por engenharia reversa, só para
   interoperabilidade e só na sua máquina. Anote cada campo com a origem.
4. **Comece só lendo**: leia os mesmos endereços que o software oficial lê e
   confira com o que ele mostra na tela.
5. **Documente** o protocolo em `docs/`, como o `docs/protocol.md`.
6. **Implemente** o mapa de memória e os testes no `salamandra-core`, com
   bytes reais lidos do mouse.
7. **Teste a escrita** com backup feito, um campo de cada vez, conferindo o
   efeito físico (LED, ponteiro, tecla), e restaure no fim.

Nenhuma dessas etapas pode trazer arquivos do software oficial para o repo.

## Commits e pull requests

- Commits em inglês, no padrão Conventional Commits, sem escopo:
  `feat: ...`, `fix: ...`, `docs: ...`, `test: ...`, `chore: ...`, `ci: ...`.
- Um assunto por pull request, com descrição curta: o que muda, por quê e como
  testar (com ou sem mouse).
- O CI precisa passar.
- Mudança que grava algo novo no mouse precisa dizer em qual mouse e sistema
  foi testada.
- Documentação muda nos dois idiomas: cada `.md` tem o par `.en.md`.

## Licença

Ao contribuir, você concorda que a sua contribuição será publicada sob a
licença MIT do projeto.
