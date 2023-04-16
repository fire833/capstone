<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import Modal from "./Modal.svelte";


    $: modal_active = false;

    function openModal() {
        console.log("Calling open modal");
        modal_active = true;
    }

    let hub_url: string;
    let hub_name: string;

    function clearForm(){
        hub_url = undefined;
        hub_name = undefined;
    }

    let error_text: string;

    async function registerHub(){

        error_text = undefined;

        let res = await fetch(`${window.location.origin}/api/hubs`, {
            method: "POST",
            headers: [["Content-Type", "application/json"]],
            body: JSON.stringify({
                url: hub_url,
                name: hub_name
            })
        });


        if(res.ok) {
            modal_active = false;
        } else {
            error_text = `Error ${res.status} ${res.statusText} - ${await res.text()}`;
        }
        
    }
</script>

<div class="add-hub-card" style="--hash-color: var(--foreground-secondary)" on:click={openModal} on:keydown={(e) => e.key === "Enter" ? openModal() : undefined}>
    <h1>+</h1>
    <p style="text-align: center;">Register New</p>
    <span class="corner topleft" />
    <span class="corner topright" />
    <span class="corner botleft" />
    <span class="corner botright" />
</div>

<Modal bind:modal_active on:close_modal={clearForm}>
    <div on:click|stopPropagation class="add-hub-modal-inner" on:keypress={() => {}}>
        <h1 style="line-height: 100%;">Register New Hub</h1>
        <hr style="margin-top: 0.5em;"/>
    
        <form on:submit|preventDefault={registerHub}>
            <span>
                <label for="huburl">Hub URL</label>
                <input bind:value={hub_url} id="huburl" type="text" placeholder="http://myhuburl.com:1234/"/>
            </span>
            <span>
                <label for="hubname">Hub Name</label>
                <input bind:value={hub_name} id="hubname "type="text" placeholder="Example Hub"/>
            </span>
    
            <button style="margin-top: 0.5em;">Register</button>
    
            {#if error_text}
                <p style="color: var(--error);">{error_text}</p>
            {/if}
        </form>
    </div>
    <div class="corner topleft" />
    <div class="corner topright" />
    <div class="corner botleft" />
    <div class="corner botright" />
</Modal>

<style>
    .add-hub-card {
        color: var(--foreground);
        border: 5px solid var(--hash-color);
        padding: 1em;
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 0.25em;
        cursor: pointer;

        transition: background-color 0.2s, box-shadow 0.4s;
    }

    .add-hub-card:hover {
        background-color: rgba(255, 255, 255, 0.1);
        box-shadow: 0 0 10px rgba(255, 255, 255, 0.2);
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

    :global(.svg-override > svg) {
        width: 100%;
        height: 100%;
        vertical-align: middle;
    }

    hr {
        border: 1px solid var(--middle);
    }

    .add-hub-modal-inner {
        padding: 2em;
        position: relative;
    }

    form {
        margin-top: 1em;
        display: flex;
        flex-direction: column;
        gap: 1em
    }

    form input {
        background-color: rgba(0, 0, 0, 0);
        border: 4px solid var(--foreground-secondary-light);
        border-radius: 0;
        padding: 0.5em;
        color: var(--foreground);
    }

    form input::placeholder {
        color: var(--middle);
    }

    form label {
        color: var(--foreground-secondary);
        line-height: 100%;

    }

    form span {
        display: flex;
        flex-direction: column;
        gap: 0.25em;
    }

    form button {
        border: 4px solid var(--foreground-secondary-light);
        border-radius: 0;
    }

    form button:hover {
        background-color: rgba(255, 255, 255, 0.1);
        box-shadow: 0 0 10px rgba(255, 255, 255, 0.1);
    }
</style>
