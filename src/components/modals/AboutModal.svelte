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
    <div class="modal-dialog modal-dialog-centered" role="document">
      <div class="modal-content">
        <div class="modal-header border-0 pb-0">
          <button
            type="button"
            class="btn-close"
            on:click={close}
            aria-label="Close"
          ></button>
        </div>
        <div class="modal-body text-center pt-0">
          <div class="mb-4">
            <i class="fas fa-database text-primary" style="font-size: 64px;"
            ></i>
            <h3 class="mt-3 mb-1 fw-bold">RustDBGrid</h3>
            <p class="text-muted mb-0">v1.0.0</p>
          </div>

          <p class="lead mb-4">Universal Database Management Tool</p>

          <div class="mb-4">
            <h6 class="text-muted text-uppercase small fw-bold mb-3">
              Built with
            </h6>
            <div class="d-flex justify-content-center gap-3 flex-wrap">
              <span
                class="badge bg-secondary bg-opacity-10 text-dark px-3 py-2"
              >
                <i class="fas fa-code me-1"></i> Tauri (Rust)
              </span>
              <span
                class="badge bg-secondary bg-opacity-10 text-dark px-3 py-2"
              >
                <i class="fab fa-js me-1"></i> Svelte
              </span>
              <span
                class="badge bg-secondary bg-opacity-10 text-dark px-3 py-2"
              >
                <i class="fas fa-edit me-1"></i> CodeMirror
              </span>
            </div>
          </div>

          <div class="mb-4">
            <h6 class="text-muted text-uppercase small fw-bold mb-3">
              Supported Databases
            </h6>
            <div class="d-flex justify-content-center gap-2 flex-wrap">
              <span class="badge bg-primary">MySQL</span>
              <span class="badge bg-info">PostgreSQL</span>
              <span class="badge bg-success">MongoDB</span>
              <span class="badge bg-danger">Redis</span>
              <span class="badge bg-warning text-dark">Apache Ignite</span>
            </div>
          </div>

          <p class="text-muted small mb-0">© 2024 RustDBGrid</p>
        </div>
        <div class="modal-footer border-0 pt-0 justify-content-center">
          <button type="button" class="btn btn-primary px-4" on:click={close}
            >Close</button
          >
        </div>
      </div>
    </div>
  </div>
  <div class="modal-backdrop fade show"></div>
{/if}
