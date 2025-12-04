<script>
  import { onMount } from "svelte";
  import {
    activeConnection,
    selectedDatabase,
    selectedTable,
    isSaving,
    saveStatus,
  } from "../../../features/connection/stores/connections";
  import { tabDataStore } from "../../stores/tabData";
  import { themePreference } from "../../../features/settings/stores/theme";
  import { toggleTheme } from "../../../features/settings/services/themeService";
  import { getStorageInfo } from "../../../core/integrations/tauri";

  export let activeTabId = null;

  let storageInfo = null;

  onMount(async () => {
    await loadStorageInfo();
  });

  async function loadStorageInfo() {
    try {
      storageInfo = await getStorageInfo();
    } catch (error) {
      console.error("Failed to load storage info:", error);
    }
  }

  function getThemeIcon(preference) {
    if (preference === "light") return "fa-sun";
    if (preference === "dark") return "fa-moon";
    return "fa-desktop";
  }

  function getThemeLabel(preference) {
    if (preference === "light") return "Light";
    if (preference === "dark") return "Dark";
    return "Auto";
  }

  // Reactive statement: reload storage info when saving completes
  $: if (!$isSaving) {
    loadStorageInfo();
  }

  $: currentTabData = activeTabId ? $tabDataStore[activeTabId] : null;
</script>

<div
  class="status-bar d-flex justify-content-between align-items-center border-top px-2 py-1"
  style="height: 24px; font-size: 10px;"
>
  <div class="d-flex align-items-center gap-2">
    <div class="d-flex align-items-center gap-1">
      {#if $activeConnection}
        <span
          class="badge bg-success rounded-circle p-1"
          style="width: 8px; height: 8px;"
        ></span>
        <div class="d-flex align-items-center gap-1">
          <strong>{$activeConnection.name}</strong>
          <span class="text-muted">·</span>
          <span class="text-secondary">{$activeConnection.db_type}</span>
          <span class="text-muted">·</span>
          <span class="text-secondary"
            >{$activeConnection.host}:{$activeConnection.port}</span
          >
        </div>
      {:else}
        <span
          class="badge bg-secondary rounded-circle p-1"
          style="width: 8px; height: 8px;"
        ></span>
        <span class="text-muted">Not connected</span>
      {/if}
    </div>

    {#if $selectedDatabase}
      <span class="vr"></span>
      <div class="d-flex align-items-center gap-1">
        <i class="fas fa-database text-secondary"></i>
        <span>{$selectedDatabase.name}</span>
      </div>
    {/if}

    {#if $selectedTable}
      <span class="vr"></span>
      <div class="d-flex align-items-center gap-1">
        <i class="fas fa-table text-secondary"></i>
        <span>{$selectedTable.name}</span>
      </div>
    {/if}
  </div>

  <div class="d-flex align-items-center gap-2">
    {#if currentTabData?.queryResult}
      <div class="d-flex align-items-center gap-1">
        <i class="fas fa-list text-secondary"></i>
        <span>
          {currentTabData.queryResult.rows.length} rows
        </span>
      </div>
      <span class="vr"></span>
      <div class="d-flex align-items-center gap-1">
        <i class="fas fa-columns text-secondary"></i>
        <span>
          {currentTabData.queryResult.columns.length} columns
        </span>
      </div>
    {/if}
    {#if currentTabData?.queryResult && storageInfo}
      <span class="vr"></span>
    {/if}
    {#if $saveStatus.message}
      <div class="d-flex align-items-center gap-1">
        {#if $saveStatus.type === "info" || $isSaving}
          <i class="fas fa-spinner fa-spin text-primary"></i>
          <span class="text-primary">{$saveStatus.message}</span>
        {:else if $saveStatus.type === "success"}
          <i class="fas fa-check-circle text-success"></i>
          <span class="text-success">{$saveStatus.message}</span>
        {:else if $saveStatus.type === "error"}
          <i class="fas fa-exclamation-circle text-danger"></i>
          <span class="text-danger">{$saveStatus.message}</span>
        {/if}
      </div>
    {:else if storageInfo}
      <div
        class="d-flex align-items-center gap-1"
        title="{storageInfo.path}\n{storageInfo.exists
          ? `Encrypted (${(storageInfo.size_bytes / 1024).toFixed(1)} KB)`
          : 'Not saved yet'}"
      >
        {#if $isSaving}
          <i class="fas fa-spinner fa-spin text-primary"></i>
          <span>Saving...</span>
        {:else}
          <i class="fas fa-save text-success"></i>
          <span>{storageInfo.exists ? "Saved" : "Not saved"}</span>
        {/if}
      </div>
    {/if}

    <!-- Theme Toggle -->
    <span class="vr"></span>
    <button
      class="theme-toggle d-flex align-items-center gap-1"
      on:click={toggleTheme}
      title="Theme: {getThemeLabel($themePreference)} (click to cycle)"
    >
      <i class="fas {getThemeIcon($themePreference)} pb-1"></i>
      <span>{getThemeLabel($themePreference)}</span>
    </button>
  </div>
</div>

<style>
  .status-bar {
    background: var(--bg-toolbar);
    color: var(--text-primary);
  }

  .theme-toggle {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 11px;
    transition: all 0.15s ease;
  }

  .theme-toggle:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .theme-toggle i {
    font-size: 10px;
  }
</style>
