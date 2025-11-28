<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;

  function close() {
    dispatch("close");
  }
</script>

{#if show}
  <div
    class="modal fade show d-block"
    tabindex="-1"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="modal-dialog modal-lg modal-dialog-centered modal-dialog-scrollable"
      role="document"
    >
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">
            <i class="fas fa-keyboard me-2"></i>
            Keyboard Shortcuts
          </h5>
        </div>
        <div class="modal-body">
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
        <div class="modal-footer">
          <button type="button" class="btn btn-primary" on:click={close}
            >Close</button
          >
        </div>
      </div>
    </div>
  </div>
  <div class="modal-backdrop fade show"></div>
{/if}

<style>
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
