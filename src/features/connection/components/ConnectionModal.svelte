<script>
  import BaseModal from "./base/BaseModal.svelte";
  import { testConnection, saveConnection } from "../../utils/tauri";
  import { isSaving } from "../../stores/connections";
  import {
    parseConnectionString as parseConnStr,
    getDefaultPort,
    getConnectionStringFormats,
  } from "../../utils/connectionStringParser";
  import { DatabaseType } from "../../utils/databaseTypes";

  export let connection = null;
  export let show = true;

  let formData = {
    id: crypto.randomUUID(),
    name: "",
    db_type: DatabaseType.MYSQL,
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
  let connectionString = "";
  let showConnectionString = false;

  if (connection) {
    formData = { ...connection };
  }

  function parseConnectionString() {
    const result = parseConnStr(connectionString, formData.db_type);

    if (result.success) {
      Object.assign(formData, result.data);
      connectionString = "";
      showConnectionString = false;
      testResult = {
        success: true,
        message: "Connection string parsed successfully!",
      };
    } else {
      testResult = {
        success: false,
        message: result.error,
      };
    }
  }

  // Update port when database type changes
  $: {
    if (formData.db_type) {
      formData.port = getDefaultPort(formData.db_type);
    }
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
    // Validate connection name is not empty
    if (!formData.name || !formData.name.trim()) {
      testResult = { success: false, message: "Connection name is required" };
      return;
    }

    saving = true;
    isSaving.set(true);
    try {
      await saveConnection(formData);
      show = false;
    } catch (error) {
      alert("Failed to save connection: " + error);
    } finally {
      saving = false;
      isSaving.set(false);
    }
  }
</script>

<BaseModal {show} size="lg" backdrop="static" on:close>
  <div slot="header">
    <h5 class="modal-title">
      <i class="fas fa-plug"></i>
      {connection ? "Edit Connection" : "New Connection"}
    </h5>
  </div>

  <div slot="body">
    <form on:submit|preventDefault={handleSave}>
      <div class="mb-3">
        <label class="form-label" for="connectionName">Connection Name</label>
        <input
          type="text"
          class="form-control"
          id="connectionName"
          bind:value={formData.name}
          required
        />
      </div>

      <div class="mb-3">
        <button
          type="button"
          class="btn btn-sm btn-outline-primary w-100"
          on:click={() => (showConnectionString = !showConnectionString)}
        >
          <i class="fas fa-link"></i>
          {showConnectionString ? "Hide" : "Paste"} Connection String
        </button>

        {#if showConnectionString}
          <div class="mt-2">
            <textarea
              class="form-control"
              rows="3"
              placeholder="Paste your connection string here (e.g., mongodb://user:pass@host:port/db)"
              bind:value={connectionString}
            ></textarea>
            <button
              type="button"
              class="btn btn-sm btn-primary mt-2"
              on:click={parseConnectionString}
              disabled={!connectionString.trim()}
            >
              <i class="fas fa-check"></i>
              Parse Connection String
            </button>
            <small class="form-text text-muted d-block mt-1">
              Supported formats: {getConnectionStringFormats(formData.db_type)}
            </small>
          </div>
        {/if}
      </div>

      <div class="mb-3">
        <label class="form-label" for="dbType">Database Type</label>
        <select
          class="form-select db-type-select"
          id="dbType"
          bind:value={formData.db_type}
        >
          <option value={DatabaseType.MYSQL}>🐬 MySQL</option>
          <option value={DatabaseType.POSTGRESQL}>🐘 PostgreSQL</option>
          <option value={DatabaseType.MONGODB}>🍃 MongoDB</option>
          <option value={DatabaseType.REDIS}>📕 Redis</option>
          <option value={DatabaseType.IGNITE}>🔥 Apache Ignite</option>
          <option value={DatabaseType.MSSQL}>🗄️ Microsoft SQL Server</option>
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
          <label class="form-label" for="username"
            >Username {formData.db_type === DatabaseType.REDIS ||
            formData.db_type === DatabaseType.IGNITE
              ? "(optional)"
              : ""}</label
          >
          <input
            type="text"
            class="form-control"
            id="username"
            bind:value={formData.username}
          />
        </div>
        <div class="col-md-6 mb-3">
          <label class="form-label" for="password"
            >Password {formData.db_type === DatabaseType.REDIS ||
            formData.db_type === DatabaseType.IGNITE
              ? "(optional)"
              : ""}</label
          >
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
          class="alert {testResult.success ? 'alert-success' : 'alert-danger'}"
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

  <div slot="footer">
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
      on:click={() => (show = false)}
    >
      Cancel
    </button>
  </div>
</BaseModal>

<style>
  .auto-save-notice {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-muted);
    margin-right: auto;
  }

  .auto-save-notice i {
    color: var(--accent-green);
  }

  .db-type-select {
    font-size: 15px;
  }

  .db-type-select option {
    padding: 8px;
  }
</style>
