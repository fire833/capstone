<script lang="ts">
    import type { Writable } from "svelte/store";
    import {
        api_data,
        breadcrumb,
        follow_breadcrumb,
        get_breadcrumb_siblings,
        type HubInfo,
    } from "../data";

    export let hubs: HubInfo[];
    export let depth: number;

    $: object_name = ["Hub", "Node", "Slot"][depth];

    $: nav_objects = follow_breadcrumb($breadcrumb, depth, hubs);
</script>

<div class="nav-panel" style="--theme-depth: {depth}">
    {#each get_breadcrumb_siblings($breadcrumb, depth, hubs) as sibling, idx}
        <button
            class="nav-button"
            class:active={idx === $breadcrumb[depth]}
            on:click={() =>
                breadcrumb.set([...$breadcrumb.slice(0, depth), idx])}>{object_name} {idx}</button
        >
    {/each}
</div>

<style>
    .nav-panel {
        width: 10em;
        height: 100%;
        /* background-color: rgba(9, 9, 20, calc(1 - 0.03 * var(--theme-depth))); */
        background-color: hsl(250, 25%, calc(10% + 2% * var(--theme-depth)));
        display: flex;
        flex-direction: column;
        /* border-right: 1px solid rgba(80, 80, 150, 0.2); */
    }

    .nav-button {
        width: 100%;
        border: none;
        background-color: rgba(0, 0, 0, 0);
        /* border-top: 1px solid rgba(80, 80, 150, 0.2); */
        color: white;
        cursor: pointer;
        font-size: 1.5em;
        font-weight: bold;
    }

    .nav-button:first-of-type {
        border: none;
    }

    .nav-button.active {
        background-color: hsl(250, 25%, calc(10% + 2% * (1 + var(--theme-depth))));
    }
</style>
