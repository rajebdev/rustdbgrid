<script>
  import { createEventDispatcher } from "svelte";
  import { testConnection, saveConnection } from "../../utils/tauri";
  import { isSaving } from "../../stores/connections";

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
  let connectionString = "";
  let showConnectionString = false;

  if (connection) {
    formData = { ...connection };
  }

  function parseConnectionString() {
    try {
      const str = connectionString.trim();

      if (formData.db_type === "MySQL") {
        // JDBC: jdbc:mysql://host:port/database or mysql://user:password@host:port/database
        let match = str.match(
          /^jdbc:mysql:\/\/([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
        );
        if (match) {
          formData.host = match[1];
          formData.port = match[2] ? parseInt(match[2]) : 3306;
          if (match[3]) formData.database = match[3];
        } else {
          match = str.match(
            /^mysql:\/\/(?:([^:]+):([^@]+)@)?([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
          );
          if (match) {
            if (match[1]) formData.username = decodeURIComponent(match[1]);
            if (match[2]) formData.password = decodeURIComponent(match[2]);
            formData.host = match[3];
            formData.port = match[4] ? parseInt(match[4]) : 3306;
            if (match[5]) formData.database = match[5];
          }
        }
      } else if (formData.db_type === "PostgreSQL") {
        // JDBC: jdbc:postgresql://host:port/database or postgresql://user:password@host:port/database
        let match = str.match(
          /^jdbc:postgresql:\/\/([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
        );
        if (match) {
          formData.host = match[1];
          formData.port = match[2] ? parseInt(match[2]) : 5432;
          if (match[3]) formData.database = match[3];
        } else {
          match = str.match(
            /^postgres(?:ql)?:\/\/(?:([^:]+):([^@]+)@)?([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
          );
          if (match) {
            if (match[1]) formData.username = decodeURIComponent(match[1]);
            if (match[2]) formData.password = decodeURIComponent(match[2]);
            formData.host = match[3];
            formData.port = match[4] ? parseInt(match[4]) : 5432;
            if (match[5]) formData.database = match[5];
          }
        }
      } else if (formData.db_type === "MongoDB") {
        // mongodb://user:password@host:port/database or mongodb+srv://...
        const match = str.match(
          /^mongodb(?:\+srv)?:\/\/(?:([^:]+):([^@]+)@)?([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
        );
        if (match) {
          if (match[1]) formData.username = decodeURIComponent(match[1]);
          if (match[2]) formData.password = decodeURIComponent(match[2]);
          formData.host = match[3];
          formData.port = match[4] ? parseInt(match[4]) : 27017;
          if (match[5]) formData.database = match[5];
        }
      } else if (formData.db_type === "Redis") {
        // redis://[:password@]host:port[/database]
        const match = str.match(
          /^redis:\/\/(?::([^@]+)@)?([^:\/]+)(?::(\d+))?(?:\/(\d+))?/
        );
        if (match) {
          if (match[1]) formData.password = decodeURIComponent(match[1]);
          formData.host = match[2];
          formData.port = match[3] ? parseInt(match[3]) : 6379;
          if (match[4]) formData.database = match[4];
        }
      }

      connectionString = "";
      showConnectionString = false;
      testResult = {
        success: true,
        message: "Connection string parsed successfully!",
      };
    } catch (error) {
      testResult = {
        success: false,
        message: "Failed to parse connection string: " + error,
      };
    }
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
                  Supported formats:
                  {#if formData.db_type === "MySQL"}
                    mysql://user:password@host:port/database or
                    jdbc:mysql://host:port/database
                  {:else if formData.db_type === "PostgreSQL"}
                    postgresql://user:password@host:port/database or
                    jdbc:postgresql://host:port/database
                  {:else if formData.db_type === "MongoDB"}
                    mongodb://user:password@host:port/database
                  {:else if formData.db_type === "Redis"}
                    redis://:password@host:port/database
                  {:else}
                    Connection string format varies by database type
                  {/if}
                </small>
              </div>
            {/if}
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
