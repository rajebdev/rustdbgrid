<script>
  import { createEventDispatcher } from "svelte";
  import TabBar from "../navigation/TabBar.svelte";
  import QueryTabContent from "../../tab-content/QueryTabContent.svelte";
  import TableTabContent from "../../tab-content/TableTabContent.svelte";
  import ProcedureTabContent from "../../tab-content/ProcedureTabContent.svelte";
  import WelcomeScreen from "../../screens/WelcomeScreen.svelte";

  const dispatch = createEventDispatcher();

  export let tabs = [];
  export let activeTab = null;
  export let currentTabData = null;
  export let editorHeight = 300;
  export let isResizingEditor = false;

  function handleTabSelect(event) {
    dispatch("tabSelect", event.detail);
  }

  function handleTabClose(event) {
    dispatch("tabClose", event.detail);
  }

  function handleNewTab() {
    dispatch("newTab");
  }

  function forwardEvent(event) {
    dispatch(event.type, event.detail);
  }
</script>

<div
  class="d-flex flex-column flex-grow-1"
  style="min-height: 0; overflow: hidden;"
>
  <div style="flex-shrink: 0; position: sticky; top: 0; z-index: 100;">
    <TabBar
      {tabs}
      {activeTab}
      on:select={handleTabSelect}
      on:close={handleTabClose}
      on:new={handleNewTab}
    />
  </div>

  <div
    class="flex-grow-1 d-flex flex-column main-content-area"
    style="overflow: hidden; min-height: 0;"
  >
    {#if activeTab}
      {#if activeTab.type === "query"}
        <QueryTabContent
          tabId={activeTab.id}
          {currentTabData}
          {editorHeight}
          {isResizingEditor}
        />
      {:else if activeTab.type === "table"}
        <TableTabContent
          tabId={activeTab.id}
          {currentTabData}
          tableInfo={activeTab.tableInfo}
        />
      {:else if activeTab.type === "procedure"}
        {#key activeTab.id}
          <ProcedureTabContent
            procedure={{
              name: activeTab.procedureInfo.name,
              procedure_type: activeTab.procedureInfo.procedure_type,
            }}
            database={{ name: activeTab.procedureInfo.database }}
            connection={activeTab.procedureInfo.connection}
          />
        {/key}
      {/if}
    {:else}
      <WelcomeScreen
        on:newQuery={forwardEvent}
        on:newConnection={forwardEvent}
        on:toggleSidebar={forwardEvent}
        on:keyboardShortcuts={forwardEvent}
      />
    {/if}
  </div>
</div>
