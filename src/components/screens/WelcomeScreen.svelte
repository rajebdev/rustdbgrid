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
  class="d-flex flex-column align-items-center justify-content-center h-100 bg-body-tertiary p-4"
  style="gap: 2rem;"
>
  <!-- Logo besar -->
  <div class="text-center">
    <i
      class="fas fa-database text-primary"
      style="font-size: 120px; opacity: 0.9;"
    ></i>
    <h2 class="mt-3 fw-bold text-dark">RustDBGrid</h2>
    <p class="text-muted">Universal Database Management Tool</p>
  </div>

  <!-- Start section -->
  <div class="welcome-section">
    <h5 class="text-muted mb-3">
      <i class="fas fa-play-circle me-2"></i>Start
    </h5>
    <div class="d-flex flex-column gap-2">
      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
        on:click={handleNewQuery}
      >
        <i class="fas fa-file-code text-primary" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="text-dark fw-medium">New Query</div>
          <small class="text-muted">Create a new SQL query tab</small>
        </div>
        <kbd class="kbd-shortcut">Ctrl+N</kbd>
      </button>

      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
        on:click={handleNewConnection}
      >
        <i class="fas fa-plug text-success" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="text-dark fw-medium">New Connection</div>
          <small class="text-muted">Connect to a database</small>
        </div>
        <kbd class="kbd-shortcut">Ctrl+Shift+C</kbd>
      </button>

      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
        on:click={handleToggleSidebar}
      >
        <i class="fas fa-sidebar text-info" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="text-dark fw-medium">Toggle Sidebar</div>
          <small class="text-muted">Show/hide database explorer</small>
        </div>
        <kbd class="kbd-shortcut">Ctrl+B</kbd>
      </button>
    </div>
  </div>

  <!-- Help section -->
  <div class="welcome-section">
    <h5 class="text-muted mb-3">
      <i class="fas fa-question-circle me-2"></i>Help
    </h5>
    <div class="d-flex flex-column gap-2">
      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
      >
        <i class="fas fa-book text-warning" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="text-dark fw-medium">Documentation</div>
          <small class="text-muted">Learn how to use RustDBGrid</small>
        </div>
      </button>

      <button
        class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
        on:click={handleKeyboardShortcuts}
      >
        <i class="fas fa-keyboard text-secondary" style="font-size: 20px;"></i>
        <div class="flex-grow-1">
          <div class="text-dark fw-medium">Keyboard Shortcuts</div>
          <small class="text-muted">View all available shortcuts</small>
        </div>
        <kbd class="kbd-shortcut">Ctrl+K Ctrl+S</kbd>
      </button>
    </div>
  </div>
</div>

<style>
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
    background-color: rgba(0, 0, 0, 0.03);
    border-color: rgba(0, 0, 0, 0.1);
    transform: translateX(4px);
  }

  .kbd-shortcut {
    background-color: #f0f0f0;
    border: 1px solid #ccc;
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 11px;
    font-family: "Consolas", "Monaco", monospace;
    color: #666;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }
</style>
