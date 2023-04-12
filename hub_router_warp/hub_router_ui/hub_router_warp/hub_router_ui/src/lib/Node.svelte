<script lang="ts">
    import Session from "./Session.svelte";
    import type { NodesEntity } from "./data";
    import Android from "../assets/android.53b6327754144dbb2eb23411963b28ec.svelte";
    import Firefox from "../assets/firefox.efda58979e042bab7c689eab277b5a5d.svelte";

    export let node: NodesEntity;

    $: supported_browsers = Array.from(new Set(node.slots.map(e => e.stereotype.browserName)));
</script>

<div class="node-wrapper">
    <h2 style="margin: 0; line-height: 100%; display: inline-block;">Node</h2>
    <p class="secondary" style="margin: 0; line-height: 100%; margin-bottom: 0.5em;">
        {node.id} | Selenium {node.version}
    </p>
    <div class="slots-wrapper">
        {#each node.slots as slot}
            <Session slot={slot}></Session>
        {/each}
    </div>

</div>

<style>
    .node-wrapper {
        position: relative;
        width: 100%;
        padding: 1.5em;
        padding-left: 1em;
        padding-bottom: 0;
        /* border-bottom: 1px solid var(--foreground-secondary-trans); */
    }

    p.secondary {
        color: #3e5c76;
    }

    .slots-wrapper {
        margin-top: 1em;
        border: 2px solid var(--foreground-secondary);
        background-color: var(--background-secondary-lightest);
        padding: 1em;
        border-radius: 5px;
    }
</style>
