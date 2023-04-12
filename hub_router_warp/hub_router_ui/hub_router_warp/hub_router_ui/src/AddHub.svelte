<script lang="ts">
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    function openModal() {
        dispatch("open-register-hub-modal");
        modal_active = true;
    }

    let modal_active: boolean = false;

    let dialog: HTMLDialogElement;
	$: if (dialog && modal_active) dialog.showModal();

    let hub_ip: string;
    let hub_port: string;
    let hub_name: string;

    function clearForm(){
        hub_ip = undefined;
        hub_port = undefined;
        hub_name = undefined;
    }

    function registerHub(){
        alert("Registering a new hub");
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

<dialog bind:this={dialog} on:close={() => {modal_active = false; clearForm();}} on:click|self={() => dialog.close()} on:keypress={() => {}}>
    <div on:click|stopPropagation class="add-hub-modal-inner" on:keypress={() => {}}>
        <h1 style="line-height: 100%;">Register New Hub</h1>
        <hr style="margin-top: 0.5em;"/>

        <form on:submit|preventDefault={registerHub}>
            <span>
                <label for="hubip">Hub IP</label>
                <input bind:value={hub_ip} id="hubip" type="text" placeholder="1.2.3.4"/>
            </span>
            <span>
                <label for="hubport">Hub Port</label>
                <input bind:value={hub_port} id="hubport" type="text" placeholder="9994"/>
            </span>
            <span>
                <label for="hubname">Hub Name</label>
                <input bind:value={hub_name} id="hubname "type="text" placeholder="Example Hub"/>
            </span>

            <button style="margin-top: 0.5em;">Register</button>
        </form>
    </div>
    <div class="corner topleft" />
    <div class="corner topright" />
    <div class="corner botleft" />
    <div class="corner botright" />
</dialog>

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

    dialog {
        margin: auto;
        background-color: var(--background-secondary);
        color: var(--foreground);
        border: 5px solid var(--foreground-secondary);
        padding: 0;
        overflow: hidden;
        position: relative;
        --hash-color: var(--foreground-secondary);
    }

    dialog hr {
        border: 1px solid var(--middle);
    }

    dialog::backdrop {
		background: rgba(0, 0, 0, 0.3);
	}

    .add-hub-modal-inner {
        padding: 2em;
        /* padding-left: 2em; */
        /* padding-right: 2em; */
        position: relative;
    }

    dialog form {
        margin-top: 1em;
        display: flex;
        flex-direction: column;
        gap: 1em
    }

    dialog form input {
        background-color: rgba(0, 0, 0, 0);
        border: 4px solid var(--foreground-secondary-light);
        border-radius: 0;
        padding: 0.5em;
        color: var(--foreground);
    }

    dialog form input::placeholder {
        color: var(--middle);
    }

    dialog form label {
        color: var(--foreground-secondary);
        line-height: 100%;

    }

    dialog form span {
        display: flex;
        flex-direction: column;
        gap: 0.25em;
    }

    dialog form button {
        border: 4px solid var(--foreground-secondary-light);
        border-radius: 0;
    }

    dialog form button:hover {
        background-color: rgba(255, 255, 255, 0.1);
        box-shadow: 0 0 10px rgba(255, 255, 255, 0.1);
    }
</style>
