<script lang="ts">
    import type { Writable } from "svelte/store";
    import { api_data, breadcrumb, type HubInfo } from "../data";
    import NavPanel from "./NavPanel.svelte";


</script>

<div class="navpanel-wrapper">
    {#if $api_data.state === "Loading"}
        <p>Loading navpanel</p>
    {:else if $api_data.state === "Success"}
        {#each $breadcrumb as crumb, idx}
            <NavPanel depth={idx} hubs={$api_data.hubs}></NavPanel>    
        {/each}
        {#if $breadcrumb.length < 3}
            <NavPanel depth={$breadcrumb.length} hubs={$api_data.hubs}></NavPanel>    
        {/if}
    {/if}
</div>

<style lang="css">
    .navpanel-wrapper {
        height: 100%;
        display: flex;
        flex-direction: row;
        /* background-color: red; */
    }
</style>