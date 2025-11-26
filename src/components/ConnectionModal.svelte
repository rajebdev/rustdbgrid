<script>
  import { createEventDispatcher } from "svelte";
  import { testConnection, saveConnection } from "../utils/tauri";
  import { isSaving } from "../stores/connections";

  export let connection = null;

  const dispatch = createEventDispatcher();

  let formData = {
    id: crypto.randomUUID(),
    name: "",
    db_type: "MySQL",
    host: "localhost",
    port: 3306,
    username: "root",
    password: "",
    database: "",
    ssl: false,
  };

  let testing = false;
  let saving = false;
  let testResult = null;

  if (connection) {
    formData = { ...connection };
  }

  $: {
    if (formData.db_type === "MySQL") formData.port = 3306;
    else if (formData.db_type === "PostgreSQL") formData.port = 5432;
    else if (formData.db_type === "MongoDB") formData.port = 27017;
    else if (formData.db_type === "Redis") formData.port = 6379;
    else if (formData.db_type === "Ignite") formData.port = 10800;
  }

  async function handleTest() {
    testing = true;
    testResult = null;
    try {
      const result = await testConnection(formData);
      testResult = result;
    } catch (error) {
      testResult = { success: false, message: error.toString() };
    }
    testing = false;
  }

  async function handleSave() {
    saving = true;
    isSaving.set(true);
    try {
      await saveConnection(formData);
      // Auto-saved to encrypted JSON file
      dispatch("save");
    } catch (error) {
      alert("Failed to save connection: " + error);
    } finally {
      saving = false;
      isSaving.set(false);
    }
  }

  function handleClose() {
    dispatch("close");
  }
</script>

<div class="modal show d-block" tabindex="-1">
  <div class="modal-dialog modal-lg">
    <div class="modal-content">
      <div class="modal-header">
        <h5 class="modal-title">
          <i class="fas fa-plug"></i>
          {connection ? "Edit Connection" : "New Connection"}
        </h5>
        <button type="button" title="" class="btn-close" on:click={handleClose}
        ></button>
      </div>

      <div class="modal-body">
        <form on:submit|preventDefault={handleSave}>
          <div class="mb-3">
            <label class="form-label" for="connectionName"
              >Connection Name</label
            >
            <input
              type="text"
              class="form-control"
              id="connectionName"
              bind:value={formData.name}
              required
            />
          </div>

          <div class="mb-3">
            <label class="form-label" for="dbType">Database Type</label>
            <select
              class="form-select"
              id="dbType"
              bind:value={formData.db_type}
            >
              <option value="MySQL">MySQL</option>
              <option value="PostgreSQL">PostgreSQL</option>
              <option value="MongoDB">MongoDB</option>
              <option value="Redis">Redis</option>
              <option value="Ignite">Apache Ignite</option>
            </select>
          </div>

          <div class="row">
            <div class="col-md-8 mb-3">
              <label class="form-label" for="host">Host</label>
              <input
                type="text"
                class="form-control"
                id="host"
                bind:value={formData.host}
                required
              />
            </div>
            <div class="col-md-4 mb-3">
              <label class="form-label" for="port">Port</label>
              <input
                type="number"
                class="form-control"
                id="port"
                bind:value={formData.port}
                required
              />
            </div>
          </div>

          <div class="row">
            <div class="col-md-6 mb-3">
              <label class="form-label" for="username">Username</label>
              <input
                type="text"
                class="form-control"
                id="username"
                bind:value={formData.username}
              />
            </div>
            <div class="col-md-6 mb-3">
              <label class="form-label" for="password">Password</label>
              <input
                type="password"
                class="form-control"
                id="password"
                bind:value={formData.password}
              />
            </div>
          </div>

          <div class="mb-3">
            <label class="form-label" for="database">Database (optional)</label>
            <input
              type="text"
              class="form-control"
              id="database"
              bind:value={formData.database}
            />
          </div>

          <div class="mb-3 form-check">
            <input
              type="checkbox"
              class="form-check-input"
              id="sslCheck"
              bind:checked={formData.ssl}
            />
            <label class="form-check-label" for="sslCheck">Use SSL</label>
          </div>

          {#if testResult}
            <div
              class="alert {testResult.success
                ? 'alert-success'
                : 'alert-danger'}"
            >
              <i
                class="fas {testResult.success
                  ? 'fa-check-circle'
                  : 'fa-times-circle'}"
              ></i>
              {testResult.message}
            </div>
          {/if}
        </form>
      </div>

      <div class="modal-footer">
        <div class="auto-save-notice">
          <i class="fas fa-shield-alt"></i>
          <small>Connection auto-saved to encrypted file</small>
        </div>
        <button
          type="button"
          class="btn btn-secondary"
          on:click={handleTest}
          disabled={testing}
        >
          <i class="fas fa-vial"></i>
          {testing ? "Testing..." : "Test Connection"}
        </button>
        <button
          type="button"
          class="btn btn-primary"
          on:click={handleSave}
          disabled={saving}
        >
          <i class="fas fa-save"></i>
          {saving ? "Saving..." : "Save"}
        </button>
        <button
          type="button"
          class="btn btn-outline-secondary"
          on:click={handleClose}
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
</div>
<div class="modal-backdrop show"></div>

<style>
  .auto-save-notice {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #6c757d;
    margin-right: auto;
  }

  .auto-save-notice i {
    color: #28a745;
  }
</style>
