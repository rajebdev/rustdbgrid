<script>
  import { createEventDispatcher } from "svelte";
  import { activeConnection } from "../stores/connections";

  const dispatch = createEventDispatcher();

  function handleAction(action) {
    dispatch(action);
  }
</script>

<div
  class="d-flex align-items-center justify-content-between bg-light border-bottom px-2"
  style="height: 32px; gap: 4px;"
>
  <div class="d-flex align-items-center gap-2">
    <div class="btn-group btn-group-sm">
      <button
        class="btn btn-outline-secondary"
        title="Search"
        on:click={() => handleAction("search")}
      >
        <i class="fas fa-search"></i>
        <span class="d-none d-xl-inline ms-1">Search</span>
      </button>

      <button
        class="btn btn-primary"
        title="Add connection"
        on:click={() => handleAction("addConnection")}
      >
        <i class="fas fa-plus"></i>
        <span class="d-none d-xl-inline ms-1">Add connection</span>
      </button>
    </div>

    <div class="vr"></div>

    <div class="btn-group btn-group-sm">
      <button
        class="btn btn-outline-secondary"
        title="New query"
        on:click={() => handleAction("newQuery")}
      >
        <i class="fas fa-file-code"></i>
        <span class="d-none d-xl-inline ms-1">New query</span>
      </button>

      <button
        class="btn btn-outline-secondary"
        title="New table"
        on:click={() => handleAction("newTable")}
      >
        <i class="fas fa-table"></i>
        <span class="d-none d-xl-inline ms-1">New table</span>
      </button>

      <button
        class="btn btn-outline-secondary"
        title="Compare DB"
        on:click={() => handleAction("compareDB")}
      >
        <i class="fas fa-exchange-alt"></i>
        <span class="d-none d-xl-inline ms-1">Compare DB</span>
      </button>

      <button
        class="btn btn-outline-secondary"
        title="Import data"
        on:click={() => handleAction("importData")}
      >
        <i class="fas fa-upload"></i>
        <span class="d-none d-xl-inline ms-1">Import data</span>
      </button>

      <button
        class="btn btn-outline-secondary"
        title="SQL Generator"
        on:click={() => handleAction("sqlGenerator")}
      >
        <i class="fas fa-magic"></i>
        <span class="d-none d-xl-inline ms-1">SQL Generator</span>
      </button>
    </div>
  </div>

  <div class="d-flex align-items-center gap-2">
    {#if $activeConnection}
      <button
        class="badge bg-success-subtle text-success border border-success d-flex align-items-center gap-1 btn btn-sm p-1"
        style="font-size: 10px; font-weight: normal;"
        title="Current connection: {$activeConnection.name}"
        on:click={() => handleAction("showConnectionInfo")}
      >
        <i class="fas fa-plug"></i>
        <span class="d-none d-lg-inline">{$activeConnection.name}</span>
      </button>
    {/if}

    <button
      class="btn btn-sm btn-outline-secondary"
      title="Refresh"
      on:click={() => handleAction("refresh")}
    >
      <i class="fas fa-sync-alt"></i>
      <span class="d-none d-xl-inline ms-1">Refresh</span>
    </button>
  </div>
</div>

<style>
  /* Bootstrap overrides for smaller buttons */
  .btn-group-sm > .btn {
    font-size: 11px;
    padding: 3px 8px;
  }
</style>
