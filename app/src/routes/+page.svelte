<script lang="ts">
  import { onMount } from "svelte";
  import { app } from "$lib/state.svelte";
  import { onDeviceEvent, onServiceError } from "$lib/api";
  import { i18n, t } from "$lib/i18n.svelte";
  import ProfileEditor from "$lib/ProfileEditor.svelte";
  import MacrosView from "$lib/MacrosView.svelte";
  import { installNativeFeel } from "$lib/nativeFeel";
  import "../app.css";

  onMount(() => {
    installNativeFeel();
    app.connect();
    const unlisten = [onDeviceEvent((e) => app.onDeviceEvent(e)), onServiceError((m) => (app.error = m))];
    return () => unlisten.forEach((u) => u.then((f) => f()));
  });

  import { getVersion } from "@tauri-apps/api/app";
  let version = $state("");
  getVersion().then((v) => (version = v));

  const rgb = (c: [number, number, number]) => `rgb(${c[0]}, ${c[1]}, ${c[2]})`;
</script>

<div class="shell">
  <nav class="sidebar">
    <div class="brand"><img src="/logo.svg" alt="" width="24" height="24" />Salamandra</div>
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
    {#if app.connected && app.model && !app.model.tested}
      <div class="untested" role="alert">
        <strong>{t("Modelo não testado no mouse físico.")}</strong>
        {t("Os valores vêm do software oficial e de documentação pública. Faça um backup antes de gravar e relate o resultado.")}
        <a href="https://github.com/polotto/salamandra/issues/new?template=new-mouse.yml" target="_blank" rel="noreferrer">{t("Relatar teste")}</a>
      </div>
    {/if}
    {#if !app.connected}
      <div class="empty">
        <h2>{t("Mouse não encontrado")}</h2>
        <p>{t("Conecte um mouse Redragon suportado por USB.")}</p>
        {#if app.error}<p class="error selectable">{app.error}</p>{/if}
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
        <span class="grow"></span>
        <button disabled={app.busy} onclick={() => app.importProfile(p)}>{t("Carregar arquivo")}</button>
        <button disabled={app.busy} onclick={() => app.exportProfile(p)}>{t("Salvar em arquivo")}</button>
      </header>
      <ProfileEditor bind:profile={app.edited[p]} />
      <footer class="bar">
        <span class="status selectable">{app.error ?? app.message ?? ""}</span>
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
      <header class="page-header"><h1>{t("Sobre")}</h1></header>
      <div class="about">
        <div class="card">
          <h3>Salamandra {version}</h3>
          <p>{t("Configurador aberto de mouses gamer para Windows, Linux e macOS.")}</p>
          {#if app.model}
            <p class="muted">
              {t("Mouse")}: {app.model.name} (USB {app.model.vendor_id.toString(16).padStart(4, "0")}:{app.model.product_id.toString(16).padStart(4, "0")}).
            </p>
          {/if}
          <p class="muted">{t("Projeto independente, sem vínculo com a Redragon. Licença MIT.")}</p>
        </div>
        <div class="card">
          <h3>{t("Backup do mouse")}</h3>
          <p class="muted">{t("Salva tudo o que está no mouse: perfis, botões e macros.")}</p>
          <div class="actions">
            <button disabled={app.busy} onclick={() => app.backup()}>{t("Fazer backup")}</button>
            <button disabled={app.busy} onclick={() => app.restore()}>{t("Restaurar backup")}</button>
          </div>
        </div>
        <div class="card">
          <h3>{t("Padrão de fábrica")}</h3>
          <p class="muted">{t("Volta os 5 perfis para o padrão de fábrica. As macros não mudam.")}</p>
          <button
            disabled={app.busy}
            onclick={() => app.resetAll(t("Voltar todos os perfis para o padrão de fábrica?"))}>{t("Resetar todos os perfis")}</button
          >
        </div>
        <p class="status selectable">{app.error ?? app.message ?? ""}</p>
      </div>
    {/if}
  </main>
</div>
