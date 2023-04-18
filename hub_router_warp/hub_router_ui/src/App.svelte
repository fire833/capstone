<script lang="ts">
  import { api_data, sort_hubs, type APIStatusData } from "./lib/data";
  import Hub from "./Hub.svelte";
  import AddHub from "./AddHub.svelte";
  import RemoveHub from "./RemoveHub.svelte";
    import Configure from "./Configure.svelte";
    import Logs from "./Logs.svelte";

  
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
          <Hub {hub_data} />
        {/each}
        <AddHub />
        <RemoveHub />
      </div>
      <h1>Configure</h1>
      <div class="hubs-row">
        <Configure></Configure>
        <Logs></Logs>
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
    overflow-x: auto;
    /* flex-wrap: wrap; */
    padding-bottom: 1em;
  }

  .hubs-row::-webkit-scrollbar {
    background-color: var(--middle);
    height: 0.5em;
  }

  .hubs-row::-webkit-scrollbar-thumb {
    background-color: var(--foreground-secondary);
  }

</style>
