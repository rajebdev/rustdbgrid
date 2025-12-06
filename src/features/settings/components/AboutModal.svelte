<script>
  import { invoke } from "@tauri-apps/api/core";
  import BaseModal from "../../../shared/components/base/BaseModal.svelte";

  export let show = false;

  let appVersion = "loading...";
  let appYear = "2024";

  $: if (show) {
    loadVersion();
    loadYear();
  }

  async function loadVersion() {
    try {
      appVersion = await invoke("get_app_version");
    } catch (error) {
      console.error("Failed to get app version:", error);
      appVersion = "unknown";
    }
  }

  async function loadYear() {
    try {
      appYear = await invoke("get_app_year");
    } catch (error) {
      console.error("Failed to get app year:", error);
      appYear = new Date().getFullYear().toString();
    }
  }
</script>

<BaseModal {show} on:close>
  <div slot="body" class="text-center pt-4">
    <div class="mb-4">
      <i class="fas fa-database text-primary about-icon"></i>
      <h3 class="mt-3 mb-1 fw-bold">RustDBGrid</h3>
      <p class="text-muted mb-0">v{appVersion}</p>
    </div>

    <p class="lead mb-4">Universal Database Management Tool</p>

    <div class="mb-4">
      <h6 class="text-muted text-uppercase small fw-bold mb-3">Built with</h6>
      <div class="d-flex justify-content-center gap-3 flex-wrap">
        <span class="badge bg-secondary bg-opacity-10 text-dark px-3 py-2">
          <i class="fas fa-code me-1"></i> Tauri (Rust)
        </span>
        <span class="badge bg-secondary bg-opacity-10 text-dark px-3 py-2">
          <i class="fab fa-js me-1"></i> Svelte
        </span>
        <span class="badge bg-secondary bg-opacity-10 text-dark px-3 py-2">
          <i class="fas fa-edit me-1"></i> Monaco Editor
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

    <p class="text-muted small mb-0">© {appYear} RustDBGrid</p>
  </div>

  <div slot="footer" class="border-0 pt-0 justify-content-center">
    <button
      type="button"
      class="btn btn-primary px-4"
      on:click={() => (show = false)}
    >
      Close
    </button>
  </div>
</BaseModal>

<style>
  .about-icon {
    font-size: 64px;
  }
</style>
