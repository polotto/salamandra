<script lang="ts">
  import { onMount } from "svelte";
  import type { ButtonAction, FireTarget, Shortcut, Special } from "./types";
  import { t } from "./i18n.svelte";
  import { keys } from "./keys.svelte";
  import { app } from "./state.svelte";

  let { buttons = $bindable() }: { buttons: ButtonAction[] } = $props();

  onMount(() => {
    keys.load();
  });

  const names = [
    "Clique esquerdo", "Clique direito", "Botão da roda", "Tiro rápido", "DPI +", "DPI -",
    "Lateral 1", "Lateral 2", "Lateral 3", "Lateral 4", "Lateral 5", "Lateral 6",
    "Lateral 7", "Lateral 8", "Lateral 9", "Lateral 10", "Lateral 11", "Lateral 12",
    "Roda para cima", "Roda para baixo",
  ];

  type Kind = ButtonAction["type"];
  const kinds: [Kind, string][] = [
    ["LeftClick", "Clique esquerdo"],
    ["RightClick", "Clique direito"],
    ["MiddleClick", "Clique do meio"],
    ["Back", "Voltar"],
    ["Forward", "Avançar"],
    ["Key", "Tecla"],
    ["Combo", "Combinação"],
    ["Media", "Mídia"],
    ["Shortcut", "Atalho"],
    ["Special", "Função do mouse"],
    ["Macro", "Macro"],
    ["Fire", "Tiro rápido"],
    ["Disabled", "Desabilitado"],
  ];

  const media: [number, string][] = [
    [0xcd, "Play/pausa"],
    [0xb7, "Parar"],
    [0xb6, "Faixa anterior"],
    [0xb5, "Próxima faixa"],
    [0xe9, "Volume +"],
    [0xea, "Volume -"],
    [0xe2, "Mudo"],
  ];

  const specials: [Special, string][] = [
    ["DpiCycle", "Ciclo de DPI"],
    ["DpiUp", "DPI +"],
    ["DpiDown", "DPI -"],
    ["ScrollUp", "Rolar para cima"],
    ["ScrollDown", "Rolar para baixo"],
    ["ProfileSwitch", "Trocar perfil"],
    ["RateUp", "Polling +"],
    ["RateDown", "Polling -"],
    ["LedModeSwitch", "Trocar modo do LED"],
  ];

  const shortcuts: [Shortcut, string][] = [
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

  const modifiers: [number, string][] = [
    [1, "Ctrl"],
    [2, "Shift"],
    [4, "Alt"],
    [8, "Win/Cmd"],
  ];

  function defaultFor(kind: Kind): ButtonAction {
    switch (kind) {
      case "Key":
        return { type: "Key", value: 0x04 };
      case "Combo":
        return { type: "Combo", value: { modifiers: 1, key: 0x06 } };
      case "Media":
        return { type: "Media", value: 0xcd };
      case "Shortcut":
        return { type: "Shortcut", value: "Copy" };
      case "Special":
        return { type: "Special", value: "DpiCycle" };
      case "Macro":
        return { type: "Macro", value: { index: 0, repeat_while_held: false, count: 1 } };
      case "Fire":
        return { type: "Fire", value: { target: { type: "Left" }, times: 3, delay_ms: 0 } };
      default:
        return { type: kind } as ButtonAction;
    }
  }

  const fireKind = (f: FireTarget) => (f.type === "Key" ? "Key" : f.type);
  function setFireTarget(i: number, kind: string) {
    const a = buttons[i];
    if (a.type !== "Fire") return;
    a.value.target = kind === "Key" ? { type: "Key", value: 0x04 } : ({ type: kind } as FireTarget);
  }

  const present = $derived(app.model?.buttons ?? buttons.map((_, i) => i));
  const hasLeftClick = $derived(buttons.some((b, i) => present.includes(i) && b.type === "LeftClick"));
</script>

{#if !hasLeftClick}
  <p class="warning">{t("Pelo menos um botão precisa ficar com o clique esquerdo. O mouse não grava sem isso.")}</p>
{/if}

<div class="card">
  {#each buttons as action, i}
   {#if present.includes(i)}
    <div class="button-row">
      <span class="name">{t(names[i])}</span>
      <select
        value={action.type}
        onchange={(e) => (buttons[i] = defaultFor(e.currentTarget.value as Kind))}
      >
        {#each kinds as [kind, label]}
          <option value={kind}>{t(label)}</option>
        {/each}
        {#if action.type === "Raw"}<option value="Raw">{t("Código desconhecido")}</option>{/if}
      </select>

      {#if action.type === "Key"}
        <select bind:value={action.value}>
          {#each keys.list as [name, usage]}<option value={usage}>{name}</option>{/each}
        </select>
      {:else if action.type === "Combo"}
        {#each modifiers as [bit, label]}
          {@const combo = action.value}
          <label class="mod">
            <input
              type="checkbox"
              checked={(combo.modifiers & bit) !== 0}
              onchange={(e) => (combo.modifiers = e.currentTarget.checked ? combo.modifiers | bit : combo.modifiers & ~bit)}
            />
            {label}
          </label>
        {/each}
        <select bind:value={action.value.key}>
          {#each keys.list as [name, usage]}<option value={usage}>{name}</option>{/each}
        </select>
      {:else if action.type === "Media"}
        <select bind:value={action.value}>
          {#each media as [usage, label]}<option value={usage}>{t(label)}</option>{/each}
        </select>
      {:else if action.type === "Shortcut"}
        <select bind:value={action.value}>
          {#each shortcuts as [value, label]}<option {value}>{t(label)}</option>{/each}
        </select>
      {:else if action.type === "Special"}
        <select bind:value={action.value}>
          {#each specials as [value, label]}<option {value}>{t(label)}</option>{/each}
        </select>
      {:else if action.type === "Macro"}
        <select bind:value={action.value.index}>
          {#each Array(15) as _, m}<option value={m}>{t("Macro")} {m + 1}</option>{/each}
        </select>
        <label class="mod">
          <input type="checkbox" bind:checked={action.value.repeat_while_held} />
          {t("Repetir enquanto segura")}
        </label>
        {#if !action.value.repeat_while_held}
          <input class="num" type="number" min="1" max="255" bind:value={action.value.count} />
          <span class="muted">{t("vezes")}</span>
        {/if}
      {:else if action.type === "Fire"}
        {@const fire = action.value}
        <select value={fireKind(fire.target)} onchange={(e) => setFireTarget(i, e.currentTarget.value)}>
          <option value="Left">{t("Esquerdo")}</option>
          <option value="Right">{t("Direito")}</option>
          <option value="Middle">{t("Meio")}</option>
          <option value="Key">{t("Tecla")}</option>
        </select>
        {#if fire.target.type === "Key"}
          {@const target = fire.target}
          <select bind:value={target.value}>
            {#each keys.list as [name, usage]}<option value={usage}>{name}</option>{/each}
          </select>
        {/if}
        <input class="num" type="number" min="1" max="255" bind:value={fire.times} />
        <span class="muted">{t("vezes")}</span>
        <input class="num" type="number" min="0" max="2550" step="10" bind:value={fire.delay_ms} />
        <span class="muted">ms</span>
      {:else if action.type === "Raw"}
        <code class="muted">{action.value.map((b) => b.toString(16).padStart(2, "0")).join(" ")}</code>
      {/if}
    </div>
   {/if}
  {/each}
</div>

<style>
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 6px 16px;
  }
  .button-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border);
  }
  .button-row:last-child {
    border-bottom: none;
  }
  .name {
    width: 130px;
    flex-shrink: 0;
  }
  .mod {
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }
  .num {
    width: 70px;
    text-align: right;
  }
  .muted {
    color: var(--muted);
  }
  .warning {
    color: var(--error);
    margin: 0;
  }
</style>
