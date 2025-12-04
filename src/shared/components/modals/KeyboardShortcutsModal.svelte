<script>
  import BaseModal from "../base/BaseModal.svelte";

  export let show = false;
</script>

<BaseModal {show} size="lg" scrollable on:close>
  <div slot="header">
    <h5 class="modal-title">
      <i class="fas fa-keyboard me-2"></i>
      Keyboard Shortcuts
    </h5>
  </div>

  <div slot="body">
    <div class="row g-4">
      <!-- File Operations -->
      <div class="col-md-6">
        <h6 class="text-muted text-uppercase small fw-bold mb-3">
          <i class="fas fa-file me-2"></i>File Operations
        </h6>
        <div class="list-group list-group-flush">
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>New Query</span>
            <kbd class="kbd-shortcut">Ctrl+N</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Open File</span>
            <kbd class="kbd-shortcut">Ctrl+O</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Save Query</span>
            <kbd class="kbd-shortcut">Ctrl+S</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Save As</span>
            <kbd class="kbd-shortcut">Ctrl+Shift+S</kbd>
          </div>
        </div>
      </div>

      <!-- Query Execution -->
      <div class="col-md-6">
        <h6 class="text-muted text-uppercase small fw-bold mb-3">
          <i class="fas fa-play me-2"></i>Query Execution
        </h6>
        <div class="list-group list-group-flush">
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Execute Query</span>
            <kbd class="kbd-shortcut">F5</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Execute Query</span>
            <kbd class="kbd-shortcut">Ctrl+Enter</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Execute Script</span>
            <kbd class="kbd-shortcut">Ctrl+Shift+Enter</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Refresh</span>
            <kbd class="kbd-shortcut">Shift+F5</kbd>
          </div>
        </div>
      </div>

      <!-- View & Navigation -->
      <div class="col-md-6">
        <h6 class="text-muted text-uppercase small fw-bold mb-3">
          <i class="fas fa-eye me-2"></i>View & Navigation
        </h6>
        <div class="list-group list-group-flush">
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Toggle Sidebar</span>
            <kbd class="kbd-shortcut">Ctrl+B</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Next Tab</span>
            <kbd class="kbd-shortcut">Ctrl+Tab</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Previous Tab</span>
            <kbd class="kbd-shortcut">Ctrl+Shift+Tab</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Close Tab</span>
            <kbd class="kbd-shortcut">Ctrl+W</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Toggle Theme</span>
            <kbd class="kbd-shortcut">Ctrl+Shift+T</kbd>
          </div>
        </div>
      </div>

      <!-- Database Operations -->
      <div class="col-md-6">
        <h6 class="text-muted text-uppercase small fw-bold mb-3">
          <i class="fas fa-database me-2"></i>Database Operations
        </h6>
        <div class="list-group list-group-flush">
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>New Connection</span>
            <kbd class="kbd-shortcut">Ctrl+Shift+C</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Commit Transaction</span>
            <kbd class="kbd-shortcut">Alt+C</kbd>
          </div>
          <div
            class="list-group-item d-flex justify-content-between align-items-center border-0 px-0"
          >
            <span>Rollback Transaction</span>
            <kbd class="kbd-shortcut">Alt+R</kbd>
          </div>
        </div>
      </div>
    </div>

    <div class="alert alert-info mt-4 mb-0">
      <i class="fas fa-info-circle me-2"></i>
      <strong>Tip:</strong> Most shortcuts use
      <kbd class="kbd-shortcut">Ctrl</kbd>
      on Windows/Linux and <kbd class="kbd-shortcut">Cmd</kbd> on macOS.
    </div>
  </div>

  <div slot="footer">
    <button
      type="button"
      class="btn btn-primary"
      on:click={() => (show = false)}
    >
      Close
    </button>
  </div>
</BaseModal>
