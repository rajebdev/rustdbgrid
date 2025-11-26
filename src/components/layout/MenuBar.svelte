<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  let activeMenu = null;

  function toggleMenu(menu) {
    activeMenu = activeMenu === menu ? null : menu;
  }

  function closeMenu() {
    activeMenu = null;
  }

  function handleAction(action) {
    dispatch(action);
    closeMenu();
  }
</script>

<svelte:window on:click={closeMenu} />

<div class="menubar">
  <div class="menu-section">
    <div class="menu-item">
      <button
        class="menu-button"
        on:click|stopPropagation={() => toggleMenu("file")}
      >
        File
      </button>
      {#if activeMenu === "file"}
        <div class="dropdown-menu show">
          <button
            class="dropdown-item"
            on:click={() => handleAction("newQuery")}
          >
            <i class="fas fa-file"></i> New SQL Script
            <span class="shortcut">Ctrl+N</span>
          </button>
          <button
            class="dropdown-item"
            on:click={() => handleAction("openFile")}
          >
            <i class="fas fa-folder-open"></i> Open File...
            <span class="shortcut">Ctrl+O</span>
          </button>
          <div class="dropdown-divider"></div>
          <button
            class="dropdown-item"
            on:click={() => handleAction("saveQuery")}
          >
            <i class="fas fa-save"></i> Save
            <span class="shortcut">Ctrl+S</span>
          </button>
          <button class="dropdown-item" on:click={() => handleAction("saveAs")}>
            <i class="fas fa-save"></i> Save As...
            <span class="shortcut">Ctrl+Shift+S</span>
          </button>
          <div class="dropdown-divider"></div>
          <button class="dropdown-item" on:click={() => handleAction("export")}>
            <i class="fas fa-download"></i> Export Data...
          </button>
          <button class="dropdown-item" on:click={() => handleAction("import")}>
            <i class="fas fa-upload"></i> Import Data...
          </button>
        </div>
      {/if}
    </div>

    <div class="menu-item">
      <button
        class="menu-button"
        on:click|stopPropagation={() => toggleMenu("edit")}
      >
        Edit
      </button>
      {#if activeMenu === "edit"}
        <div class="dropdown-menu show">
          <button class="dropdown-item" on:click={() => handleAction("undo")}>
            <i class="fas fa-undo"></i> Undo
            <span class="shortcut">Ctrl+Z</span>
          </button>
          <button class="dropdown-item" on:click={() => handleAction("redo")}>
            <i class="fas fa-redo"></i> Redo
            <span class="shortcut">Ctrl+Y</span>
          </button>
          <div class="dropdown-divider"></div>
          <button class="dropdown-item" on:click={() => handleAction("copy")}>
            <i class="fas fa-copy"></i> Copy
            <span class="shortcut">Ctrl+C</span>
          </button>
          <button class="dropdown-item" on:click={() => handleAction("paste")}>
            <i class="fas fa-paste"></i> Paste
            <span class="shortcut">Ctrl+V</span>
          </button>
        </div>
      {/if}
    </div>

    <div class="menu-item">
      <button
        class="menu-button"
        on:click|stopPropagation={() => toggleMenu("view")}
      >
        View
      </button>
      {#if activeMenu === "view"}
        <div class="dropdown-menu show">
          <button
            class="dropdown-item"
            on:click={() => handleAction("toggleSidebar")}
          >
            <i class="fas fa-sidebar"></i> Toggle Sidebar
            <span class="shortcut">Ctrl+B</span>
          </button>
          <button
            class="dropdown-item"
            on:click={() => handleAction("toggleToolbar")}
          >
            <i class="fas fa-tools"></i> Toggle Toolbar
          </button>
          <div class="dropdown-divider"></div>
          <button
            class="dropdown-item"
            on:click={() => handleAction("viewColumns")}
          >
            <i class="fas fa-columns"></i> View Columns
          </button>
        </div>
      {/if}
    </div>

    <div class="menu-item">
      <button
        class="menu-button"
        on:click|stopPropagation={() => toggleMenu("database")}
      >
        Database
      </button>
      {#if activeMenu === "database"}
        <div class="dropdown-menu show">
          <button
            class="dropdown-item"
            on:click={() => handleAction("newConnection")}
          >
            <i class="fas fa-plus-circle"></i> New Connection
            <span class="shortcut">Ctrl+Shift+C</span>
          </button>
          <button
            class="dropdown-item"
            on:click={() => handleAction("connect")}
          >
            <i class="fas fa-plug"></i> Connect
          </button>
          <button
            class="dropdown-item"
            on:click={() => handleAction("disconnect")}
          >
            <i class="fas fa-times-circle"></i> Disconnect
          </button>
        </div>
      {/if}
    </div>

    <div class="menu-item">
      <button
        class="menu-button"
        on:click|stopPropagation={() => toggleMenu("help")}
      >
        Help
      </button>
      {#if activeMenu === "help"}
        <div class="dropdown-menu show">
          <button
            class="dropdown-item"
            on:click={() => handleAction("documentation")}
          >
            <i class="fas fa-book"></i> Documentation
          </button>
          <button class="dropdown-item" on:click={() => handleAction("about")}>
            <i class="fas fa-info-circle"></i> About
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .menubar {
    display: flex;
    align-items: center;
    background: var(--bg-toolbar);
    border-bottom: 1px solid var(--border-color);
    height: 32px;
    padding: 0;
    user-select: none;
    position: relative;
    z-index: 1000;
  }

  .menu-section {
    display: flex;
    gap: 0;
    flex: 1;
    padding: 0 4px;
  }

  .menu-item {
    position: relative;
  }

  .menu-button {
    background: transparent;
    border: none;
    color: var(--text-primary);
    padding: 6px 12px;
    font-size: 13px;
    cursor: pointer;
    transition: background-color 0.15s;
    font-weight: 400;
  }

  .menu-button:hover {
    background: var(--hover-bg);
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    min-width: 220px;
    padding: 4px 0;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    z-index: 1001;
  }

  .dropdown-item {
    width: 100%;
    background: transparent;
    border: none;
    color: var(--text-primary);
    padding: 6px 16px;
    text-align: left;
    cursor: pointer;
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .dropdown-item:hover {
    background: var(--selected-bg);
  }

  .dropdown-item i {
    width: 16px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .dropdown-divider {
    height: 1px;
    background: var(--border-light);
    margin: 4px 0;
  }

  .shortcut {
    margin-left: auto;
    padding-left: 20px;
    font-size: 11px;
    color: var(--text-secondary);
    font-family: "Consolas", "Monaco", monospace;
  }
</style>
