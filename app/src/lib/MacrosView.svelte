<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { app } from "./state.svelte";
  import { keys } from "./keys.svelte";
  import { hidUsage } from "./hidcodes";
  import { macroNames, setMacroNames } from "./api";
  import { t } from "./i18n.svelte";
  import type { MacroEvent, MouseButton } from "./types";

  const MAX = 63;
  const MAX_DELAY = 655350;

  let selected = $state(0);
  let names = $state<string[]>([]);
  let events = $state<MacroEvent[]>([]);
  let row = $state<number | null>(null);
  let clipboard = $state<MacroEvent[]>([]);
  let recording = $state(false);
  let delayMode = $state<"record" | "default" | "none">("record");
  let defaultDelay = $state(50);
  let mouseButton = $state<MouseButton>("Left");
  let lastTime = 0;
  let insertAt = 0;

  const clone = <T,>(v: T): T => JSON.parse(JSON.stringify(v));
  const dirty = $derived(JSON.stringify(events) !== JSON.stringify(app.macros[selected] ?? []));

  onMount(async () => {
    keys.load();
    names = await macroNames();
    select(0);
  });
  onDestroy(stopRecording);

  function select(i: number) {
    stopRecording();
    selected = i;
    events = clone(app.macros[i] ?? []);
    row = null;
  }

  function insert(list: MacroEvent[]) {
    const room = MAX - events.length;
    if (room <= 0) return;
    const at = row === null ? events.length : row + 1;
    events.splice(at, 0, ...list.slice(0, room));
    row = at + Math.min(list.length, room) - 1;
  }

  // Gravação do teclado

  function delayBefore(): MacroEvent[] {
    const now = performance.now();
    const elapsed = now - lastTime;
    const first = lastTime === 0;
    lastTime = now;
    if (first || delayMode === "none") return [];
    const ms = delayMode === "default" ? defaultDelay : elapsed;
    const rounded = Math.min(MAX_DELAY, Math.max(10, Math.round(ms / 10) * 10));
    return [{ type: "Delay", value: rounded }];
  }

  function onKey(e: KeyboardEvent) {
    if (e.repeat) {
      e.preventDefault();
      return;
    }
    const usage = hidUsage(e.code);
    e.preventDefault();
    e.stopPropagation();
    if (usage === undefined) return;
    const action: MacroEvent = e.type === "keydown" ? { type: "KeyDown", value: usage } : { type: "KeyUp", value: usage };
    const add = [...delayBefore(), action];
    if (events.length + add.length > MAX) {
      stopRecording();
      return;
    }
    events.splice(insertAt, 0, ...add);
    insertAt += add.length;
    row = insertAt - 1;
  }

  function startRecording() {
    insertAt = row === null ? events.length : row + 1;
    lastTime = 0;
    recording = true;
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("keyup", onKey, true);
  }

  function stopRecording() {
    recording = false;
    window.removeEventListener("keydown", onKey, true);
    window.removeEventListener("keyup", onKey, true);
  }

  // Edição

  function remove() {
    if (row === null) return;
    events.splice(row, 1);
    row = events.length ? Math.min(row, events.length - 1) : null;
  }

  function move(delta: number) {
    if (row === null) return;
    const to = row + delta;
    if (to < 0 || to >= events.length) return;
    [events[row], events[to]] = [events[to], events[row]];
    row = to;
  }

  function copy(cut = false) {
    if (row === null) return;
    clipboard = [clone(events[row])];
    if (cut) remove();
  }

  async function save() {
    stopRecording();
    await setMacroNames($state.snapshot(names));
    if (await app.saveMacro(selected, $state.snapshot(events))) events = clone(app.macros[selected]);
  }

  async function saveName() {
    await setMacroNames($state.snapshot(names));
  }

  function describe(e: MacroEvent): string {
    switch (e.type) {
      case "KeyDown":
        return `${t("Apertar")} ${keys.name(e.value)}`;
      case "KeyUp":
        return `${t("Soltar")} ${keys.name(e.value)}`;
      case "MouseDown":
        return `${t("Apertar botão")} ${t(buttonName(e.value))}`;
      case "MouseUp":
        return `${t("Soltar botão")} ${t(buttonName(e.value))}`;
      case "Delay":
        return t("Atraso");
    }
  }

  const buttonName = (b: MouseButton) => ({ Left: "esquerdo", Right: "direito", Middle: "do meio" })[b];
</script>

<div class="layout">
  <aside class="list">
    {#each Array(app.model?.macro_slots ?? 15) as _, i}
      <button class="macro" class:selected={selected === i} onclick={() => select(i)}>
        <span class="n">{i + 1}</span>
        <span class="label">{names[i] || t("Macro") + " " + (i + 1)}</span>
        <span class="count">{app.macros[i]?.length ? app.macros[i].length : ""}</span>
      </button>
    {/each}
  </aside>

  <section class="editor">
    <div class="top">
      <label class="name">
        {t("Nome")}
        <input bind:value={names[selected]} placeholder={t("Macro") + " " + (selected + 1)} onchange={saveName} />
      </label>
      <span class="muted">{events.length}/{MAX} {t("ações")}</span>
    </div>

    <div class="card options">
      {#if recording}
        <button class="primary rec" onclick={stopRecording}>{t("Parar gravação")}</button>
        <span class="rec-hint">{t("Gravando: digite no teclado. As teclas não chegam a outros programas.")}</span>
      {:else}
        <button class="primary" disabled={events.length >= MAX} onclick={startRecording}>
          {row === null ? t("Gravar teclado") : t("Gravar depois da linha selecionada")}
        </button>
      {/if}
      <label class="opt"><input type="radio" bind:group={delayMode} value="record" />{t("Gravar o tempo real entre as ações")}</label>
      <label class="opt">
        <input type="radio" bind:group={delayMode} value="default" />{t("Atraso padrão")}
        <input class="num" type="number" min="10" max={MAX_DELAY} step="10" bind:value={defaultDelay} /> ms
      </label>
      <label class="opt"><input type="radio" bind:group={delayMode} value="none" />{t("Sem atraso")}</label>
    </div>

    <div class="toolbar">
      <select bind:value={mouseButton}>
        <option value="Left">{t("Botão esquerdo")}</option>
        <option value="Right">{t("Botão direito")}</option>
        <option value="Middle">{t("Botão do meio")}</option>
      </select>
      <button
        disabled={recording}
        onclick={() => insert([{ type: "MouseDown", value: mouseButton }, { type: "MouseUp", value: mouseButton }])}
        >{t("Clique")}</button
      >
      <button disabled={recording} onclick={() => insert([{ type: "MouseDown", value: mouseButton }])}>{t("Apertar")}</button>
      <button disabled={recording} onclick={() => insert([{ type: "MouseUp", value: mouseButton }])}>{t("Soltar")}</button>
      <button disabled={recording} onclick={() => insert([{ type: "Delay", value: defaultDelay }])}>+ {t("Atraso")}</button>
      <span class="sep"></span>
      <button disabled={recording || row === null} onclick={() => move(-1)}>{t("Subir")}</button>
      <button disabled={recording || row === null} onclick={() => move(1)}>{t("Descer")}</button>
      <button disabled={recording || row === null} onclick={() => copy()}>{t("Copiar")}</button>
      <button disabled={recording || row === null} onclick={() => copy(true)}>{t("Recortar")}</button>
      <button disabled={recording || clipboard.length === 0} onclick={() => insert(clone(clipboard))}>{t("Colar")}</button>
      <button disabled={recording || row === null} onclick={remove}>{t("Apagar")}</button>
      <button disabled={recording || events.length === 0} onclick={() => ((events = []), (row = null))}>{t("Limpar")}</button>
    </div>

    <div class="card rows">
      {#if events.length === 0}
        <p class="muted empty">{t("Macro vazia. Grave pelo teclado ou insira ações.")}</p>
      {/if}
      {#each events as e, i}
        <div class="row" class:selected={row === i} role="button" tabindex="0" onclick={() => (row = row === i ? null : i)} onkeydown={() => {}}>
          <span class="n">{i + 1}</span>
          <span class="what">{describe(e)}</span>
          {#if e.type === "Delay"}
            <input
              class="num"
              type="number"
              min="10"
              max={MAX_DELAY}
              step="10"
              value={e.value}
              onclick={(ev) => ev.stopPropagation()}
              onchange={(ev) => (e.value = Math.min(MAX_DELAY, Math.max(10, Math.round(+ev.currentTarget.value / 10) * 10)))}
            />
            <span class="muted">ms</span>
          {/if}
        </div>
      {/each}
    </div>

    <footer class="bar">
      <span class="status selectable">{app.error ?? app.message ?? ""}</span>
      <button disabled={!dirty || recording} onclick={() => select(selected)}>{t("Descartar")}</button>
      <button class="primary" disabled={!dirty || recording || app.busy} onclick={save}>{t("Gravar no mouse")}</button>
    </footer>
  </section>
</div>

<style>
  .layout {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .list {
    width: 210px;
    border-right: 1px solid var(--border);
    overflow: auto;
    padding: 8px;
  }
  .macro {
    display: flex;
    gap: 8px;
    width: 100%;
    border: none;
    background: none;
    text-align: left;
    padding: 6px 8px;
  }
  .macro.selected {
    background: var(--selected);
  }
  .macro .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .n,
  .count {
    color: var(--muted);
    min-width: 18px;
  }
  .editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px 24px 0;
  }
  .name {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    margin: 10px 24px 0;
    padding: 10px 14px;
  }
  .options {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px;
  }
  .opt {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .rec {
    background: var(--error);
    border-color: var(--error);
  }
  .rec-hint {
    color: var(--error);
  }
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 10px 24px 0;
  }
  .sep {
    width: 12px;
  }
  .rows {
    flex: 1;
    overflow: auto;
    margin-bottom: 10px;
    padding: 4px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 10px;
    border-radius: 6px;
    cursor: pointer;
  }
  .row.selected {
    background: var(--selected);
  }
  .what {
    flex: 1;
  }
  .num {
    width: 80px;
    text-align: right;
  }
  .muted {
    color: var(--muted);
  }
  .empty {
    padding: 12px;
  }
</style>
