<script lang="ts">
  import {
    api_data,
    type ApiStatusData,
  } from "./lib/data";
  import Hub from "./Hub.svelte";


  function sort_hubs(hubs: ApiStatusData[]): ApiStatusData[] {
    let copy = [...hubs];
    copy.sort((a, b) =>
      a.router_hub_state.name.localeCompare(b.router_hub_state.name)
    );
    return copy;
  }

</script>

{#if $api_data.state === "Error"}
  <p>Got APIFetchError: {JSON.stringify($api_data)}</p>
{:else if $api_data.state === "Success"}
  <div class="app-wrapper">
    <div class="header-row">
      <h1>Hub Router</h1>
    </div>
    <div class="hubs-wrapper">
      <h1>Hubs ({$api_data.data.length})</h1>
      <div class="hubs-row">
        {#each sort_hubs($api_data.data) as hub_data}
          <Hub {hub_data}></Hub>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style lang="css">
  .app-wrapper {
    display: flex;
    flex-direction: column;
    /* align-items: center; */
    height: 100%;
    width: 80%;
    margin-left: 10%;
    background-color: var(--background);
    color: var(--foreground);
    padding-top: 2em;
  }

  .header-row {
    width: 100%;
    border-bottom: 2px solid var(--middle);
  }

  .hubs-wrapper {
    margin-top: 1.5em;
  }

  .hubs-row {
    display: flex;
    flex-direction: row;
    gap: 1.5em;
    margin-top: 0.5em;
  }

</style>
