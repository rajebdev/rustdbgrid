<script>
  import SubTabBar from "./table-subtabs/SubTabBar.svelte";
  import PropertiesTab from "./table-subtabs/PropertiesTab.svelte";
  import DataTab from "./table-subtabs/DataTab.svelte";
  import DiagramTab from "./table-subtabs/DiagramTab.svelte";
  import { activeConnection } from "../../connection/stores/connections";
  import { tabDataStore } from "../../../shared/stores/tabData";

  export let tabId;
  export let currentTabData;
  export let tableInfo;

  // Get activeSubTab from store for this specific tab
  $: activeSubTab = currentTabData?.activeSubTab || "data";

  function handleSubTabChange(newSubTab) {
    tabDataStore.setActiveSubTab(tabId, newSubTab);
  }
</script>

<div class="table-tab-container">
  <SubTabBar {activeSubTab} onTabChange={handleSubTabChange} />

  <div class="subtab-content">
    {#if activeSubTab === "properties"}
      <PropertiesTab
        {tabId}
        {tableInfo}
        connection={tableInfo?.connection || $activeConnection}
      />
    {:else if activeSubTab === "data"}
      <DataTab
        {tabId}
        {currentTabData}
        {tableInfo}
        connection={tableInfo?.connection || $activeConnection}
      />
    {:else if activeSubTab === "diagram"}
      <DiagramTab
        {tableInfo}
        connection={tableInfo?.connection || $activeConnection}
      />
    {/if}
  </div>
</div>

<style>
  .table-tab-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .subtab-content {
    flex: 1;
    overflow: hidden;
  }
</style>
