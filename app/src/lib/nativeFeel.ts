// Comportamento de app nativo no app empacotado: sem seleção de texto, sem
// menu de navegador e sem atalhos de navegador. Campos de texto continuam
// funcionando normalmente. Fora do Tauri (navegador), nada muda.

const isDesktop = () => "__TAURI_INTERNALS__" in window;

const isEditable = (el: EventTarget | null) =>
  el instanceof HTMLElement &&
  (el.isContentEditable ||
    (el instanceof HTMLInputElement && !["checkbox", "radio", "range", "color", "button"].includes(el.type)) ||
    el instanceof HTMLTextAreaElement);

/** Atalhos de navegador bloqueados: recarregar, imprimir, buscar na página, zoom. */
function isBrowserShortcut(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey;
  if (e.key === "F5") return true;
  if (!mod) return false;
  return ["r", "p", "f", "g", "+", "=", "-", "0"].includes(e.key.toLowerCase());
}

export function installNativeFeel() {
  if (!isDesktop()) return;
  document.documentElement.classList.add("desktop");
  window.addEventListener("contextmenu", (e) => {
    if (!isEditable(e.target)) e.preventDefault();
  });
  window.addEventListener("keydown", (e) => {
    const mod = e.metaKey || e.ctrlKey;
    if (mod && e.key.toLowerCase() === "a" && !isEditable(e.target)) e.preventDefault();
    if (isBrowserShortcut(e)) e.preventDefault();
  });
}
