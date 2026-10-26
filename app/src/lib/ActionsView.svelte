<script lang="ts">
  import { onMount } from "svelte";
  import { customActions, setCustomActions } from "./api";
  import { t } from "./i18n.svelte";
  import { shortcuts } from "./shortcuts";
  import type { Builtin, CustomAction, CustomActions, Modifier, Shortcut } from "./types";

  type Kind = "Default" | "Keys" | Builtin | "Hyper" | "OpenApp" | "OpenUrl" | "Command" | "Sequence";

  const isMac = navigator.userAgent.includes("Mac");
  const allKinds: [Kind, string][] = [
    ["Default", "Padrão do atalho"],
    ["Keys", "Combinação de teclas"],
    ["MissionControl", "Mission Control"],
    ["Launchpad", "Launchpad (macOS)"],
    ["ShowDesktop", "Mostrar a área de trabalho"],
    ["Hyper", "Hyper (Ctrl+Alt+Shift+Cmd)"],
    ["OpenApp", "Abrir app"],
    ["OpenUrl", "Abrir endereço"],
    ["Command", "Rodar comando"],
  ];
  const kinds = allKinds.filter(([k]) => isMac || k !== "Launchpad");
  const modifierLabel = (m: Modifier) => (m === "Cmd" && !isMac ? "Win" : m);
  const modifiers: Modifier[] = ["Ctrl", "Shift", "Alt", "Cmd"];

  let saved = $state<CustomActions | null>(null);
  let draft = $state<CustomActions | null>(null);
  let message = $state<string | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);

  const clone = <T,>(v: T): T => JSON.parse(JSON.stringify(v));
  // Ordem fixa dos atalhos: trocar para "Padrão" e voltar não suja o rascunho.
  const snapshot = (c: CustomActions | null) => JSON.stringify(shortcuts.map(([s]) => c?.bindings[s] ?? null));
  const dirty = $derived(snapshot(draft) !== snapshot(saved));

  onMount(async () => {
    try {
      const file = await customActions();
      saved = file.actions;
      draft = clone(saved);
      error = file.load_error;
    } catch (e) {
      error = String(e);
    }
  });

  function kindOf(a: CustomAction | undefined): Kind {
    if (!a) return "Default";
    if (a.type === "Builtin") return a.name;
    return a.type === "Wait" ? "Sequence" : (a.type as Kind);
  }

  function setKind(s: Shortcut, kind: Kind) {
    if (!draft) return;
    const b = draft.bindings;
    if (kind === "Default") delete b[s];
    else if (kind === "Keys") b[s] = { type: "Keys", modifiers: [], key: "" };
    else if (kind === "Hyper") b[s] = { type: "Hyper" };
    else if (kind === "OpenApp") b[s] = { type: "OpenApp", name: "" };
    else if (kind === "OpenUrl") b[s] = { type: "OpenUrl", url: "" };
    else if (kind === "Command") b[s] = { type: "Command", argv: [""] };
    else if (kind !== "Sequence") b[s] = { type: "Builtin", name: kind };
  }

  function toggle(a: Extract<CustomAction, { type: "Keys" }>, m: Modifier) {
    a.modifiers = a.modifiers.includes(m) ? a.modifiers.filter((x) => x !== m) : [...a.modifiers, m];
  }

  async function save() {
    if (!draft) return;
    busy = true;
    error = null;
    message = null;
    try {
      await setCustomActions(draft);
      saved = clone(draft);
      message = t("Ações salvas");
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="intro muted">
  {t("Troque o que cada atalho do mouse faz neste computador. Grave o atalho num botão na aba Botões.")}
</div>

{#if draft}
  <div class="rows">
    {#each shortcuts as [s, label]}
      {@const action = draft.bindings[s]}
      <div class="row">
        <span class="name">{t(label)}</span>
        <select
          aria-label={t(label)}
          value={kindOf(action)}
          onchange={(e) => setKind(s, e.currentTarget.value as Kind)}
        >
          {#each kinds as [k, kl]}<option value={k}>{t(kl)}</option>{/each}
          {#if action?.type === "Sequence"}<option value="Sequence">{t("Sequência (edite no arquivo)")}</option>{/if}
        </select>
        {#if action?.type === "Keys"}
          {@const keys = action}
          <span class="mods">
            {#each modifiers as m}
              <label><input type="checkbox" checked={keys.modifiers.includes(m)} onchange={() => toggle(keys, m)} />{modifierLabel(m)}</label>
            {/each}
          </span>
          <input class="field short" placeholder={t("Tecla")} bind:value={keys.key} />
        {:else if action?.type === "OpenApp"}
          <input class="field" placeholder={t("Nome do app")} bind:value={action.name} />
        {:else if action?.type === "OpenUrl"}
          <input class="field" placeholder={t("Endereço")} bind:value={action.url} />
        {:else if action?.type === "Command"}
          {@const cmd = action}
          <span class="args">
            {#each cmd.argv as _, i}
              <input
                class="field"
                placeholder={i === 0 ? t("Programa") : t("Argumento")}
                bind:value={cmd.argv[i]}
              />
            {/each}
            <button onclick={() => cmd.argv.push("")}>{t("Adicionar argumento")}</button>
            {#if cmd.argv.length > 1}<button onclick={() => cmd.argv.pop()}>{t("Remover argumento")}</button>{/if}
          </span>
        {:else if action?.type === "Hyper"}
          <span class="muted">{t("Hyper segura as quatro teclas enquanto o botão está apertado.")}</span>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<footer class="bar">
  <span class="status selectable">{error ?? message ?? (dirty ? t("Alterações não salvas") : "")}</span>
  <button disabled={!dirty || busy} onclick={() => (draft = clone(saved))}>{t("Descartar ações")}</button>
  <button class="primary" disabled={!dirty || busy || !draft} onclick={save}>{t("Salvar ações")}</button>
</footer>

<style>
  .intro {
    padding: 10px 24px 0;
  }
  .rows {
    flex: 1;
    overflow: auto;
    margin: 10px 24px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  .row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
    padding: 6px 14px;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child {
    border-bottom: none;
  }
  .name {
    width: 190px;
  }
  .mods {
    display: flex;
    gap: 10px;
  }
  .mods label {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .args {
    display: flex;
    flex: 1;
    flex-wrap: wrap;
    gap: 6px;
  }
  .field {
    flex: 1;
    min-width: 160px;
  }
  .field.short {
    flex: 0 0 90px;
    min-width: 0;
  }
  .muted {
    color: var(--muted);
  }
</style>
