import { keyNames } from "./api";

/** Tabela de teclas do núcleo (`salamandra_core::keys`), carregada uma vez. */
class Keys {
  list = $state<[string, number][]>([]);

  async load() {
    if (this.list.length === 0) this.list = await keyNames();
  }

  name(usage: number) {
    return this.list.find(([, u]) => u === usage)?.[0] ?? `0x${usage.toString(16).padStart(2, "0")}`;
  }
}

export const keys = new Keys();
