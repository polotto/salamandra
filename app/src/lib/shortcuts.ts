import type { Shortcut } from "./types";

/** Os 22 atalhos embutidos do mouse, com o rótulo em pt-BR. */
export const shortcuts: [Shortcut, string][] = [
  ["Cut", "Recortar"],
  ["Copy", "Copiar"],
  ["Paste", "Colar"],
  ["SelectAll", "Selecionar tudo"],
  ["Find", "Buscar"],
  ["New", "Novo"],
  ["Print", "Imprimir"],
  ["Save", "Salvar"],
  ["SwitchWindow", "Trocar janela"],
  ["CloseWindow", "Fechar janela"],
  ["OpenFileManager", "Abrir gerenciador de arquivos"],
  ["Run", "Executar"],
  ["ShowDesktop", "Mostrar mesa"],
  ["Lock", "Bloquear"],
  ["BrowserHome", "Navegador: início"],
  ["BrowserBack", "Navegador: voltar"],
  ["BrowserForward", "Navegador: avançar"],
  ["BrowserStop", "Navegador: parar"],
  ["BrowserRefresh", "Navegador: recarregar"],
  ["BrowserSearch", "Navegador: buscar"],
  ["BrowserFavorites", "Navegador: favoritos"],
  ["Mail", "E-mail"],
];
