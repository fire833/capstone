<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import Modal from "./Modal.svelte";
    import { api_data, sort_hubs } from "./lib/data";


    $: modal_active = false;

    function openModal() {
        console.log("Calling open modal");
        modal_active = true;
    }

    let hub_uuid: string;

    function clearForm(){
        hub_uuid = undefined;
    }

    let error_text: string;

    async function registerHub(){

        error_text = undefined;

        let res = await fetch(`${window.location.origin}/api/hubs/${hub_uuid}`, {
            method: "DELETE",
            headers: [["Content-Type", "application/json"]],
        });


        if(res.ok) {
            modal_active = false;
        } else {
            error_text = `Error ${res.status} ${res.statusText} - ${await res.text()}`;
        }
        
    }
</script>

<div class="add-hub-card" style="--hash-color: var(--foreground-secondary)" on:click={openModal} on:keydown={(e) => e.key === "Enter" ? openModal() : undefined}>
    <h1>-</h1>
    <p style="text-align: center;">Deregister</p>
    <span class="corner topleft" />
    <span class="corner topright" />
    <span class="corner botleft" />
    <span class="corner botright" />
</div>

<Modal bind:modal_active on:close_modal={clearForm}>
    {#if  $api_data.state === "Success"}
        <div on:click|stopPropagation class="add-hub-modal-inner" on:keypress={() => {}}>
            <h1 style="line-height: 100%;">Deregister Hub</h1>
            <hr style="margin-top: 0.5em;"/>
        
            <form on:submit|preventDefault={registerHub}>
                <span>
                    <label for="hubuuid">Hub</label>
                    <select bind:value={hub_uuid} id="hubuuid">
                        <option disabled selected>Select a hub</option>
                        {#each sort_hubs($api_data.data) as hub}
                            <option value={hub.router_hub_state.meta.uuid}>{hub.router_hub_state.meta.name} - {hub.router_hub_state.meta.url}</option>
                        {/each}
                    </select>
                </span>

                <button style="margin-top: 0.5em;">Deregister</button>
        
                {#if error_text}
                    <p style="color: var(--error);">{error_text}</p>
                {/if}
            </form>
        </div>
    {/if}
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
