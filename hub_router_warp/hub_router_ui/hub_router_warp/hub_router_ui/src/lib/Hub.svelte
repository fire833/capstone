<script lang="ts">
    import SeleniumGridLogo from "../assets/selenium-grid-logo.86cdb73d38521c0e49691666db832e49.svelte";
    import Node from "./Node.svelte";
    import type { HubInfo } from "./data";

    export let hub: HubInfo;
</script>

<div class="hub">
    <div class="hub-info">
        <div style="display: inline-block; vertical-align: middle;">
            <div class="hub-header-row">
                <!-- <SeleniumGridLogo /> -->
                <h1 style="margin: 0; line-height: 100%;">Hub</h1>
            </div>
        </div>
        <p class="secondary" style="margin-top: 0em; line-height: 100%;">Healthy</p>
    </div>
    <!-- <hr class="spacer"/> -->
    <div class="nodes-wrapper">
        {#each hub.value.nodes as node}
            <Node {node} />
        {/each}
    </div>
</div>

<style>
    .hub {
        width: 80%;
        background-color: var(--background-secondary);
        color: var(--foreground);
        box-shadow: 0px 0px 16px var(--foreground-secondary-trans);
        border-radius: 10px;
        padding-top: 2em;
        margin-top: 2em;
        border: 2px solid var(--foreground-secondary);

        display: flex;
        flex-direction: column;
        justify-content: space-between;
    }

    .nodes-wrapper {
        margin-top: 1em;
        /* margin-bottom: 1em; */
        width: 100%;
        /* padding: 1em; */
        border-radius: 7px;
        border: 2px solid var(--foreground-secondary);
        background-color: var(--background-secondary-lighter);
        padding-left: 1.5em;
        padding-bottom: 1.5em;
        /* border-bottom-left-radius: 0; */
        /* border-bottom-right-radius: 0; */
        /* border-bottom: none; */
    }

    .spacer {
        margin-top: 0.5em;
        margin-bottom: 1em;
        border: 1px solid var(--foreground-secondary-trans);
    }

    p.secondary {
        color: #3e5c76;
    }

    .hub-info {
        padding-left: 2em;
        padding-right: 2em;
    }

    .hub-header-row {
        display: flex;
        flex-direction: row;
        align-items: center;
    }

    :global(.hub-header-row svg) {
        width: 2em;
        height: 2em;
    }

    .hub-header-row h1 {
        /* padding-left: 0.5em; */
        vertical-align: middle;
    }
</style>
