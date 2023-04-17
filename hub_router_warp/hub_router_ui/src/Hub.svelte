<script lang="ts">
    import type { APIStatusData, HubInfo } from "./lib/data";

    /**
     * Returns a hash code from a string
     * @param  {String} str The string to hash.
     * @return {Number}    A 32bit integer
     * @see http://werxltd.com/wp/2010/05/13/javascript-implementation-of-javas-string-hashcode-method/
     */
    // https://stackoverflow.com/questions/6122571/simple-non-secure-hash-function-for-javascript
    function hashCode(str): number {
        let hash = 0;
        for (let i = 0, len = str.length; i < len; i++) {
            let chr = str.charCodeAt(i);
            hash = (hash << 5) - hash + chr;
            hash |= 0; // Convert to 32bit integer
        }
        return hash;
    }

    function compute_slots(hub: HubInfo): [string, number, number][] {
        let nodes = hub.nodes;
        let slots = nodes.flatMap((e) => e.slots);
        let browser_util: { [browserName: string]: [number, number] } = {};
        for (let s of slots) {
            let browser = s.stereotype.browserName.toLowerCase();
            let session = s.session;
            if (browser_util[browser] === undefined)
                browser_util[browser] = [0, 0];
            browser_util[browser][1]++;
            browser_util[browser][0] += !!session ? 1 : 0;
        }
        let flat: [string, number, number][] = Object.keys(browser_util).map(
            (key) => [key, ...browser_util[key]]
        );
        // flat.sort((a, b) => b[0].localeCompare(a[0]));
        flat.sort(
            (a, b) => b[1] - a[1] || b[2] - a[2] || b[0].localeCompare(a[0])
        );
        return flat;
    }

    import Chrome from "./assets/chrome.191aefd5192c43508fa5f86da6808929.svelte";
    import Edge from "./assets/edge.d2a278165ff8e7dcc4af17246954a0e1.svelte";
    import Firefox from "./assets/firefox.efda58979e042bab7c689eab277b5a5d.svelte";
    import Opera from "./assets/opera.b65596581939839e3bb1bd80706b9e45.svelte";
    import Safari from "./assets/safari.b8bf72ee61f58c60edb2ffa27b172d07.svelte";

    let browser_name_to_icon = {
        chrome: Chrome,
        googlechrome: Chrome,
        firefox: Firefox,
        mozillafirefox: Firefox,
        microsoftedge: Edge,
        safari: Safari,
        opera: Opera,
    };

    export let hub_data: APIStatusData;
</script>

<div
    class="hub-row"
    style="--hash-color: hsl({Math.abs(
        hashCode(hub_data.router_hub_state.meta.name)
    ) % 360}, {(Math.abs(hashCode(hub_data.router_hub_state.meta.name)) % 20) +
        50}%, 72%)"
>
    <div style="display: flex; flex-direction: column; align-items: center;">
        <h1 class="notranslate">{hub_data.router_hub_state.meta.name}</h1>
        <p style="color: var(--foreground-secondary)">
            {hub_data.router_hub_state.meta.url}
        </p>
    </div>

    {#if hub_data.hub_status_response}
        <div
            style="display: flex; flex-direction: row; justify-content: space-evenly; margin-top: 0.5em;"
        >
            {#each compute_slots(hub_data.hub_status_response) as slot}
                <div
                    style="display: flex; flex-direction: column; align-items: center; text-align: center;"
                >
                    <div class="svg-override" style="width: 2em; height: 2em;">
                        <svelte:component
                            this={browser_name_to_icon[slot[0]]}
                        />
                    </div>
                    <p
                        style="color: var(--foreground-secondary); line-height: 100%; text-align: center; margin-top: 0.1em;"
                    >
                        {slot[1]}/{slot[2]}
                    </p>
                </div>
            {/each}
        </div>
    {:else}
        <div class="unhealthy-wrapper">
            <h3>Unhealthy</h3>
            <p>{hub_data.err}</p>
        </div>
    {/if}

    <span class="corner topleft" />
    <span class="corner topright" />
    <span class="corner botleft" />
    <span class="corner botright" />
    <!-- <h2>{hub_data.hub_status_response.value.nodes?.length} nodes</h2> -->
</div>

<style>
    .hub-row {
        color: var(--foreground);
        border: 5px solid var(--hash-color);
        padding: 1em;
        position: relative;
        display: flex;
        flex-direction: column;
        gap: 0.25em;
    }

    .corner {
        content: "";
        width: 8px;
        height: 8px;
        position: absolute;
        top: -1px;
        left: -1px;
        background-color: var(--hash-color);
        clip-path: polygon(100% 0, 0 0, 0 100%);
    }

    .corner.topright {
        top: -1px;
        bottom: unset;
        left: unset;
        right: -1px;
        transform: rotate(90deg);
    }

    .corner.botleft {
        top: unset;
        bottom: -1px;
        left: -1px;
        right: unset;
        transform: rotate(270deg);
    }

    .corner.botright {
        top: unset;
        bottom: -1px;
        left: unset;
        right: -1px;
        transform: rotate(180deg);
    }

    .hub-row h1 {
        line-height: 100%;
        margin: 0;
        text-transform: capitalize;
    }

    :global(.svg-override > svg) {
        width: 100%;
        height: 100%;
        vertical-align: middle;
    }

    .unhealthy-wrapper {
        color: var(--error);
        text-align: center;
    }
</style>
