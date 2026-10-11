<script lang="ts">
  import { onMount } from "svelte";
  import { app } from "$lib/state.svelte";
  import { onDeviceEvent, onServiceError } from "$lib/api";
  import { i18n, t } from "$lib/i18n.svelte";
  import ProfileEditor from "$lib/ProfileEditor.svelte";
  import MacrosView from "$lib/MacrosView.svelte";
  import "../app.css";

  onMount(() => {
    app.connect();
    const unlisten = [onDeviceEvent((e) => app.onDeviceEvent(e)), onServiceError((m) => (app.error = m))];
    return () => unlisten.forEach((u) => u.then((f) => f()));
  });

  const rgb = (c: [number, number, number]) => `rgb(${c[0]}, ${c[1]}, ${c[2]})`;
</script>

<div class="shell">
  <nav class="sidebar">
    <div class="brand">Salamandra</div>
    <div class="section">{t("Perfis")}</div>
    {#each app.edited as profile, p}
      <button
        class="item"
        class:selected={app.page.kind === "profile" && app.page.index === p}
        onclick={() => (app.page = { kind: "profile", index: p })}
      >
        <span class="dot" style:background={rgb(profile.led.rgb)}></span>
        {t("Perfil")} {p + 1}
        {#if app.activeProfile === p}<span class="tag">{t("ativo")}</span>{/if}
        {#if app.isDirty(p)}<span class="dirty" title={t("Alterações não gravadas")}></span>{/if}
      </button>
    {/each}
    <div class="section"></div>
    <button class="item" class:selected={app.page.kind === "macros"} onclick={() => (app.page = { kind: "macros" })}>
      {t("Macros")}
    </button>
    <button class="item" class:selected={app.page.kind === "about"} onclick={() => (app.page = { kind: "about" })}>
      {t("Sobre")}
    </button>
    <div class="spacer"></div>
    <div class="lang">
      <button class:on={i18n.lang === "pt"} onclick={() => (i18n.lang = "pt")}>PT</button>
      <button class:on={i18n.lang === "en"} onclick={() => (i18n.lang = "en")}>EN</button>
    </div>
  </nav>

  <main class="content">
    {#if !app.connected}
      <div class="empty">
        <h2>{t("Mouse não encontrado")}</h2>
        <p>{t("Conecte o Redragon M901-1 Perdition 2 por USB.")}</p>
        {#if app.error}<p class="error">{app.error}</p>{/if}
        <button class="primary" disabled={app.busy} onclick={() => app.connect()}>{t("Tentar de novo")}</button>
      </div>
    {:else if app.page.kind === "profile"}
      {@const p = app.page.index}
      <header class="page-header">
        <h1>{t("Perfil")} {p + 1}</h1>
        {#if app.activeProfile === p}
          <span class="ok">{t("Perfil ativo no mouse")}</span>
        {:else}
          <span class="muted">{t("Este perfil não está ativo.")}</span>
          <button disabled={app.busy} onclick={() => app.activate(p)}>{t("Ativar")}</button>
        {/if}
      </header>
      <ProfileEditor bind:profile={app.edited[p]} />
      <footer class="bar">
        <span class="status">{app.error ?? app.message ?? ""}</span>
        <button disabled={app.busy} onclick={() => app.loadFactory(p)}>{t("Padrão de fábrica")}</button>
        <button disabled={!app.isDirty(p) || app.busy} onclick={() => app.discard(p)}>{t("Descartar")}</button>
        <button class="primary" disabled={!app.isDirty(p) || app.busy} onclick={() => app.save(p)}>
          {t("Gravar no mouse")}
        </button>
      </footer>
    {:else if app.page.kind === "macros"}
      <header class="page-header"><h1>{t("Macros")}</h1></header>
      <MacrosView />
    {:else}
      <div class="empty"><p class="muted">{app.page.kind}</p></div>
    {/if}
  </main>
</div>
