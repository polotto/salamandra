import * as api from "./api";
import { ask, open, save } from "@tauri-apps/plugin-dialog";
import type { DeviceEvent, MacroEvent, Profile } from "./types";
import { t } from "./i18n.svelte";

export type Page = { kind: "profile"; index: number } | { kind: "macros" } | { kind: "about" };

const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v));
const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

class AppState {
  connected = $state(false);
  busy = $state(false);
  message = $state<string | null>(null);
  error = $state<string | null>(null);
  activeProfile = $state(0);
  /** Como está no mouse. */
  loaded = $state<Profile[]>([]);
  /** Como está na tela. */
  edited = $state<Profile[]>([]);
  macros = $state<MacroEvent[][]>([]);
  page = $state<Page>({ kind: "profile", index: 0 });

  isDirty(p: number) {
    return this.loaded[p] !== undefined && !same(this.loaded[p], this.edited[p]);
  }

  private async run<T>(label: string, work: () => Promise<T>): Promise<T | undefined> {
    if (this.busy) return undefined;
    this.busy = true;
    this.error = null;
    this.message = t(label);
    try {
      const result = await work();
      this.message = null;
      return result;
    } catch (e) {
      this.message = null;
      this.error = String(e);
      if (String(e).includes("não encontrado")) this.connected = false;
      return undefined;
    } finally {
      this.busy = false;
    }
  }

  async connect() {
    const snap = await this.run("Lendo o mouse...", api.connect);
    if (!snap) return;
    this.connected = true;
    this.applySnapshot(snap);
  }

  async save(p: number) {
    const fresh = await this.run("Gravando o perfil...", () =>
      api.saveProfile(p, $state.snapshot(this.edited[p])),
    );
    if (!fresh) return;
    this.loaded[p] = fresh;
    this.edited[p] = clone(fresh);
    this.message = `${t("Perfil")} ${p + 1}: ${t("gravado")}`;
  }

  discard(p: number) {
    this.edited[p] = clone($state.snapshot(this.loaded[p]));
  }

  /** Só na tela: o usuário confere e grava. */
  async loadFactory(p: number) {
    const f = await this.run("Carregando o padrão de fábrica...", () => api.factoryDefaults(p));
    if (f) this.edited[p] = f;
  }

  async activate(p: number) {
    const active = await this.run("Ativando o perfil...", () => api.setActiveProfile(p));
    if (active !== undefined) this.activeProfile = active;
  }

  async saveMacro(i: number, events: MacroEvent[]) {
    const fresh = await this.run("Gravando a macro...", () => api.setMacro(i, events));
    if (!fresh) return false;
    this.macros[i] = fresh;
    this.message = `${t("Macro")} ${i + 1}: ${t("gravada")}`;
    return true;
  }

  async exportProfile(p: number) {
    const path = await save({
      defaultPath: `perfil-${p + 1}.salamandra.json`,
      filters: [{ name: "Salamandra", extensions: ["json"] }],
    });
    if (!path) return;
    if ((await this.run("Salvando...", () => api.exportProfile(path, $state.snapshot(this.edited[p])))) !== undefined)
      this.message = t("Perfil salvo em arquivo");
  }

  /** Carrega no editor; o usuário confere e grava. */
  async importProfile(p: number) {
    const path = await open({ filters: [{ name: "Salamandra", extensions: ["json"] }] });
    if (!path) return;
    const profile = await this.run("Carregando...", () => api.importProfile(path));
    if (profile) {
      this.edited[p] = profile;
      this.message = t("Perfil carregado. Confira e grave no mouse.");
    }
  }

  async backup() {
    const path = await save({ defaultPath: "salamandra-backup.json", filters: [{ name: "JSON", extensions: ["json"] }] });
    if (!path) return;
    if ((await this.run("Fazendo backup...", () => api.backup(path))) !== undefined) this.message = t("Backup salvo");
  }

  private applySnapshot(snap: { active_profile: number; profiles: Profile[]; macros: MacroEvent[][] }) {
    this.activeProfile = snap.active_profile;
    this.loaded = snap.profiles;
    this.edited = clone(snap.profiles);
    this.macros = snap.macros;
  }

  async restore() {
    const path = await open({ filters: [{ name: "JSON", extensions: ["json"] }] });
    if (!path) return;
    const snap = await this.run("Restaurando...", () => api.restore(path));
    if (snap) {
      this.applySnapshot(snap);
      this.message = t("Backup restaurado");
    }
  }

  async resetAll(question: string) {
    if (!(await ask(question, { kind: "warning" }))) return;
    const snap = await this.run("Voltando ao padrão de fábrica...", api.resetAll);
    if (snap) {
      this.applySnapshot(snap);
      this.message = t("Todos os perfis voltaram ao padrão de fábrica");
    }
  }

  /** Mudanças feitas pelos botões do próprio mouse. */
  onDeviceEvent(e: DeviceEvent) {
    const p = this.activeProfile;
    switch (e.type) {
      case "ProfileChanged":
        this.activeProfile = e.value;
        break;
      case "DpiLevelChanged":
        if (this.loaded[p]) {
          const dirty = this.isDirty(p);
          this.loaded[p].active_dpi_level = e.value;
          if (!dirty) this.edited[p].active_dpi_level = e.value;
        }
        break;
      case "PollingRateChanged":
        if (this.loaded[p] && e.value) {
          const dirty = this.isDirty(p);
          this.loaded[p].polling_rate = e.value;
          if (!dirty) this.edited[p].polling_rate = e.value;
        }
        break;
    }
  }
}

export const app = new AppState();
