<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  function handleNewQuery() {
    dispatch("newQuery");
  }

  function handleNewConnection() {
    dispatch("newConnection");
  }

  function handleToggleSidebar() {
    dispatch("toggleSidebar");
  }

  function handleKeyboardShortcuts() {
    dispatch("keyboardShortcuts");
  }
</script>

<div
  class="welcome-screen d-flex flex-column align-items-center justify-content-center h-100 p-4"
  style="gap: 2rem;"
>
  <!-- Logo besar -->
  <div class="text-center">
    <i
      class="fas fa-database welcome-icon"
      style="font-size: 120px; opacity: 0.9;"
    ></i>
    <h2 class="mt-3 fw-bold welcome-title">RustDBGrid</h2>
    <p class="welcome-subtitle">Universal Database Management Tool</p>
  </div>

  <!-- Start section -->
  <div class="welcome-section">
    <h5 class="section-title mb-3">
      <i class="fas fa-play-circle me-2"></i>Start
    </h5>
    <div class="d-flex flex-column gap-2">
      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
        on:click={handleNewQuery}
      >
        <i class="fas fa-file-code icon-primary" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="button-title fw-medium">New Query</div>
          <small class="button-subtitle">Create a new SQL query tab</small>
        </div>
        <kbd class="kbd-shortcut">Ctrl+N</kbd>
      </button>

      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
        on:click={handleNewConnection}
      >
        <i class="fas fa-plug icon-success" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="button-title fw-medium">New Connection</div>
          <small class="button-subtitle">Connect to a database</small>
        </div>
        <kbd class="kbd-shortcut">Ctrl+Shift+C</kbd>
      </button>

      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
        on:click={handleToggleSidebar}
      >
        <i class="fas fa-columns icon-info" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="button-title fw-medium">Toggle Sidebar</div>
          <small class="button-subtitle">Show/hide database explorer</small>
        </div>
        <kbd class="kbd-shortcut">Ctrl+B</kbd>
      </button>
    </div>
  </div>

  <!-- Help section -->
  <div class="welcome-section">
    <h5 class="section-title mb-3">
      <i class="fas fa-question-circle me-2"></i>Help
    </h5>
    <div class="d-flex flex-column gap-2">
      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
      >
        <i class="fas fa-book icon-warning" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="button-title fw-medium">Documentation</div>
          <small class="button-subtitle">Learn how to use RustDBGrid</small>
        </div>
      </button>

      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
        on:click={handleKeyboardShortcuts}
      >
        <i class="fas fa-keyboard icon-secondary" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="button-title fw-medium">Keyboard Shortcuts</div>
          <small class="button-subtitle">View all available shortcuts</small>
        </div>
        <kbd class="kbd-shortcut">Ctrl+K Ctrl+S</kbd>
      </button>
    </div>
  </div>
</div>

<style>
  .welcome-screen {
    background: var(--bg-primary);
  }

  .welcome-icon {
    color: var(--accent-blue);
  }

  .welcome-title {
    color: var(--text-primary);
  }

  .welcome-subtitle {
    color: var(--text-muted);
  }

  .section-title {
    color: var(--text-muted);
  }

  .button-title {
    color: var(--text-primary);
  }

  .button-subtitle {
    color: var(--text-muted);
  }

  .icon-primary {
    color: var(--accent-blue);
  }

  .icon-success {
    color: var(--accent-green);
  }

  .icon-info {
    color: var(--accent-blue);
  }

  .icon-warning {
    color: var(--accent-yellow);
  }

  .icon-secondary {
    color: var(--text-secondary);
  }

  .welcome-section {
    width: 100%;
    max-width: 600px;
  }

  .welcome-button {
    border-radius: 8px;
    transition: all 0.2s ease;
    background-color: transparent;
    border: 1px solid transparent;
  }

  .welcome-button:hover {
    background-color: var(--hover-bg);
    border-color: var(--border-light);
    transform: translateX(4px);
  }

  .kbd-shortcut {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 11px;
    font-family: "Consolas", "Monaco", monospace;
    color: var(--text-secondary);
    box-shadow: var(--shadow-sm);
  }
</style>
