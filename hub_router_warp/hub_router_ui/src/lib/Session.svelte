<script lang="ts">
    import type { SlotsEntity } from "./data";
    export let slot: SlotsEntity;
    import Chrome from "../assets/chrome.191aefd5192c43508fa5f86da6808929.svelte";
    import Edge from "../assets/edge.d2a278165ff8e7dcc4af17246954a0e1.svelte";
    import Firefox from "../assets/firefox.efda58979e042bab7c689eab277b5a5d.svelte";
    import Opera from "../assets/opera.b65596581939839e3bb1bd80706b9e45.svelte";
    import Safari from "../assets/safari.b8bf72ee61f58c60edb2ffa27b172d07.svelte";

    let browser_name_to_icon = {
        chrome: Chrome,
        googlechrome: Chrome,
        firefox: Firefox,
        mozillafirefox: Firefox,
        microsoftedge: Edge,
        safari: Safari,
    };
</script>

<div class="slot-wrapper">
    <div class="slot-row">
        <div class="svg-override">
            <svelte:component
                this={browser_name_to_icon[
                    slot.stereotype.browserName.toLowerCase()
                ]}
                width={69}
            />
        </div>
        {#if slot.session === null}
            <h3 class="slot-h3">Slot</h3>
        {:else}
            <p>Session: {JSON.stringify(slot.session)}</p>
        {/if}
    </div>
</div>

<style>
    .slot-wrapper {
        width: 100%;
    }

    :global(.svg-override > svg) {
        width: 1.5em;
        height: 1.5em;
        vertical-align: middle;
    }

    .svg-override {
        display: inline-block;
        vertical-align: middle;
    }

    .slot-h3 {
        display: inline-block;
        padding-left: 0.5em;
    }
</style>
