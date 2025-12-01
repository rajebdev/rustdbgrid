<script>
  import { createEventDispatcher } from "svelte";
  import MenuBar from "./MenuBar.svelte";
  import Toolbar from "./Toolbar.svelte";
  import StatusBar from "./StatusBar.svelte";

  const dispatch = createEventDispatcher();

  export let showToolbar = true;
  export let activeTabId = null;

  function forwardEvent(event) {
    dispatch(event.type, event.detail);
  }
</script>

<div class="d-flex flex-column vh-100 overflow-hidden bg-body">
  <MenuBar
    on:newQuery={forwardEvent}
    on:openFile={forwardEvent}
    on:saveQuery={forwardEvent}
    on:saveAs={forwardEvent}
    on:export={forwardEvent}
    on:import={forwardEvent}
    on:undo={forwardEvent}
    on:redo={forwardEvent}
    on:copy={forwardEvent}
    on:paste={forwardEvent}
    on:toggleSidebar={forwardEvent}
    on:toggleToolbar={forwardEvent}
    on:viewColumns={forwardEvent}
    on:documentation={forwardEvent}
    on:about={forwardEvent}
    on:newConnection={forwardEvent}
    on:connect={forwardEvent}
    on:disconnect={forwardEvent}
  />

  {#if showToolbar}
    <Toolbar
      on:newConnection={forwardEvent}
      on:newQuery={forwardEvent}
      on:saveQuery={forwardEvent}
      on:execute={forwardEvent}
      on:executeScript={forwardEvent}
      on:stop={forwardEvent}
      on:refresh={forwardEvent}
    />
  {/if}

  <div class="d-flex flex-grow-1 overflow-hidden">
    <slot name="sidebar" />
    <slot name="content" />
  </div>

  <StatusBar {activeTabId} />
</div>
