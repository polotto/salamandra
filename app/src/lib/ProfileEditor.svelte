<script lang="ts">
  import type { LedMode, PollingRate, Profile } from "./types";
  import { t } from "./i18n.svelte";
  import ButtonsTab from "./ButtonsTab.svelte";
  import { app } from "./state.svelte";

  let { profile = $bindable() }: { profile: Profile } = $props();
  let tab = $state<"general" | "dpi" | "light" | "buttons">("general");

  const rates: [PollingRate, string][] = [
    ["Hz125", "125 Hz"],
    ["Hz250", "250 Hz"],
    ["Hz500", "500 Hz"],
    ["Hz1000", "1000 Hz"],
  ];

  // Paleta da tela de luz do oficial, mais cores comuns.
  const palette: [number, number, number][] = [
    [255, 0, 0],
    [255, 128, 0],
    [255, 255, 0],
    [0, 255, 0],
    [0, 255, 255],
    [0, 0, 255],
    [255, 0, 255],
    [255, 255, 255],
  ];

  const toHex = (c: number[]) => "#" + c.map((v) => v.toString(16).padStart(2, "0")).join("");
  const fromHex = (h: string): [number, number, number] => [
    parseInt(h.slice(1, 3), 16),
    parseInt(h.slice(3, 5), 16),
    parseInt(h.slice(5, 7), 16),
  ];

  type ModeKind = LedMode["type"];
  const modeLabels: Record<string, string> = {
    Steady: "Fixo",
    Breathing: "Respirar",
    Rainbow: "Arco-íris",
    Wave: "Onda",
    Alternating: "Alternado",
    Reactive: "Reativo",
    Flash: "Piscar",
    Off: "Desligado",
  };
  /** Só os modos que o modelo oferece. */
  const modes = $derived(
    (app.model?.led_modes ?? ["Steady", "Breathing", "Flash", "Off"]).map(
      (kind) => [kind, modeLabels[kind] ?? kind] as [ModeKind, string],
    ),
  );
  const withSpeed = ["Breathing", "Flash", "Rainbow", "Wave", "Alternating", "Reactive"];

  function setMode(kind: ModeKind) {
    const speed = "value" in profile.led.mode ? profile.led.mode.value.speed : 5;
    profile.led.mode = withSpeed.includes(kind) ? ({ type: kind, value: { speed } } as LedMode) : ({ type: kind } as LedMode);
  }

  function setSpeed(speed: number) {
    const m = profile.led.mode;
    if ("value" in m) profile.led.mode = { type: m.type, value: { speed } } as LedMode;
  }

  /** Valores que o sensor do modelo aceita. */
  const dpiValues = $derived(app.model?.dpi_values ?? []);
  const dpiMin = $derived(dpiValues[0] ?? 100);
  const dpiMax = $derived(dpiValues[dpiValues.length - 1] ?? 24000);
  const clampDpi = (v: number) =>
    dpiValues.reduce((best, c) => (Math.abs(c - v) < Math.abs(best - v) ? c : best), dpiValues[0] ?? 100);
  const dpiIndex = (v: number) => Math.max(0, dpiValues.indexOf(clampDpi(v)));
  const dpiStepAt = (v: number) => {
    const i = dpiIndex(v);
    return (dpiValues[i + 1] ?? v + 100) - dpiValues[i];
  };

  function setX(i: number, v: number) {
    const l = profile.dpi_levels[i];
    const linked = l.x === l.y;
    l.x = clampDpi(v);
    if (linked) l.y = l.x;
  }
</script>

<div class="tabs">
  <button class:on={tab === "general"} onclick={() => (tab = "general")}>{t("Geral")}</button>
  <button class:on={tab === "dpi"} onclick={() => (tab = "dpi")}>{t("DPI")}</button>
  <button class:on={tab === "light"} onclick={() => (tab = "light")}>{t("Luz")}</button>
  <button class:on={tab === "buttons"} onclick={() => (tab = "buttons")}>{t("Botões")}</button>
</div>

<div class="body">
  {#if tab === "general"}
    <div class="card">
      <h3>{t("Taxa de polling")}</h3>
      <div class="segmented">
        {#each rates as [value, label]}
          <button class:on={profile.polling_rate === value} onclick={() => (profile.polling_rate = value)}>{label}</button>
        {/each}
      </div>
    </div>
    <div class="card">
      <h3>{t("Velocidade de rolagem")}</h3>
      <div class="row">
        <input type="range" min="1" max="10" bind:value={profile.scroll_speed} />
        <span class="value">{profile.scroll_speed} {t("linha(s) por passo")}</span>
      </div>
    </div>
    <p class="hint">
      {t("Velocidade do ponteiro, aceleração e duplo clique são ajustes do sistema operacional e chegam numa próxima versão.")}
    </p>
  {:else if tab === "dpi"}
    <div class="card">
      <h3>{t("Níveis de DPI")}</h3>
      <p class="hint">{dpiMin} {t("a")} {dpiMax}. {t("Os botões DPI + e DPI - percorrem só os níveis ligados.")}</p>
      {#each profile.dpi_levels as level, i}
        {@const linked = level.x === level.y}
        <div class="dpi-row" class:active={profile.active_dpi_level === i}>
          <input
            type="radio"
            name="active-dpi"
            title={t("Nível ativo")}
            checked={profile.active_dpi_level === i}
            disabled={!level.enabled}
            onchange={() => (profile.active_dpi_level = i)}
          />
          <label class="check">
            <input type="checkbox" bind:checked={level.enabled} disabled={profile.active_dpi_level === i} />
            {t("Nível")} {i + 1}
          </label>
          <input
            type="range"
            min="0"
            max={dpiValues.length - 1}
            step="1"
            value={dpiIndex(level.x)}
            oninput={(e) => setX(i, dpiValues[+e.currentTarget.value])}
          />
          <input
            class="num"
            type="number"
            min={dpiMin}
            max={dpiMax}
            step={dpiStepAt(level.x)}
            value={level.x}
            onchange={(e) => setX(i, +e.currentTarget.value)}
          />
          {#if !linked}
            <span class="muted">Y</span>
            <input
              class="num"
              type="number"
              min={dpiMin}
              max={dpiMax}
              step={dpiStepAt(level.y)}
              value={level.y}
              onchange={(e) => (level.y = clampDpi(+e.currentTarget.value))}
            />
          {/if}
          <label class="check">
            <input
              type="checkbox"
              checked={linked}
              onchange={(e) => (level.y = e.currentTarget.checked ? level.x : clampDpi(level.x + dpiStepAt(level.x)))}
            />
            {t("X/Y iguais")}
          </label>
        </div>
      {/each}
    </div>
  {:else if tab === "light"}
    <div class="card">
      <h3>{t("Modo")}</h3>
      <div class="segmented">
        {#each modes as [kind, label]}
          <button class:on={profile.led.mode.type === kind} onclick={() => setMode(kind)}>{t(label)}</button>
        {/each}
      </div>
      {#if "value" in profile.led.mode}
        <div class="row">
          <span>{t("Velocidade")}</span>
          <input
            type="range"
            min="1"
            max="8"
            value={profile.led.mode.value.speed}
            oninput={(e) => setSpeed(+e.currentTarget.value)}
          />
          <span class="value">{profile.led.mode.value.speed}</span>
        </div>
      {/if}
    </div>
    <div class="card">
      <h3>{t("Cor")}</h3>
      <div class="row">
        {#each palette as c}
          <button
            class="swatch"
            class:on={toHex(c) === toHex(profile.led.rgb)}
            style:background={toHex(c)}
            aria-label={toHex(c)}
            onclick={() => (profile.led.rgb = [...c])}
          ></button>
        {/each}
        <label class="check">
          {t("Personalizada")}
          <input
            type="color"
            value={toHex(profile.led.rgb)}
            oninput={(e) => (profile.led.rgb = fromHex(e.currentTarget.value))}
          />
        </label>
      </div>
    </div>
    <div class="card">
      <h3>{t("Brilho")}</h3>
      <div class="segmented">
        {#each [[1, "Baixo"], [2, "Médio"], [3, "Alto"]] as [value, label]}
          <button class:on={profile.led.brightness === value} onclick={() => (profile.led.brightness = +value)}>
            {t(String(label))}
          </button>
        {/each}
      </div>
    </div>
  {:else}
    <ButtonsTab bind:buttons={profile.buttons} />
  {/if}
</div>

<style>
  .tabs {
    display: flex;
    gap: 4px;
    padding: 0 24px 8px;
    border-bottom: 1px solid var(--border);
  }
  .tabs button {
    border: none;
    background: none;
    border-bottom: 2px solid transparent;
    border-radius: 0;
    padding: 6px 12px;
  }
  .tabs button.on {
    border-bottom-color: var(--accent);
    font-weight: 600;
  }
  .body {
    flex: 1;
    overflow: auto;
    padding: 16px 24px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 12px 16px;
  }
  h3 {
    margin: 0 0 10px;
    font-size: 13px;
  }
  .segmented {
    display: inline-flex;
  }
  .segmented button {
    border-radius: 0;
    margin-left: -1px;
  }
  .segmented button:first-child {
    border-radius: 6px 0 0 6px;
  }
  .segmented button:last-child {
    border-radius: 0 6px 6px 0;
  }
  .segmented button.on {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-text);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 6px;
  }
  .value {
    color: var(--muted);
  }
  .hint {
    color: var(--muted);
    font-size: 12px;
    margin: 0 0 8px;
  }
  .dpi-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 4px;
    border-radius: 6px;
  }
  .dpi-row.active {
    background: var(--selected);
  }
  .dpi-row input[type="range"] {
    flex: 1;
  }
  .num {
    width: 80px;
    text-align: right;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
  }
  .swatch {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    padding: 0;
  }
  .swatch.on {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .muted {
    color: var(--muted);
  }
</style>
