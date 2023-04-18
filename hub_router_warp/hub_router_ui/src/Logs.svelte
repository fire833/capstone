<script lang="ts">
    import Modal from "./Modal.svelte";

    type ApiLog = {
      "log": string,
      "time": {
        "secs_since_epoch": number,
        "nanos_since_epoch": number
      }
    }

    $: logs_data = get_logs_api_call();

    function get_logs_api_call() {
        return fetch(`${window.location.origin}/api/logs`).then(async (e) => {
            if (e.ok) {
                return await e.json();
            } else {
                throw await `Error ${e.status} ${e.statusText}: ${await e.text()}`;
            }
        });
    }

    $: modal_active = false;

    function openModal() {
        console.log("Calling open modal");
        logs_data = get_logs_api_call();
        modal_active = true;
    }

    function sort_logs(logs: ApiLog[]) {
        let copy = [...logs];
        copy.sort((a, b) => a.time.secs_since_epoch - b.time.secs_since_epoch);
        return copy;
    }
</script>

<div
    class="add-hub-card"
    style="--hash-color: var(--foreground-secondary)"
    on:click={openModal}
    on:keydown={(e) => (e.key === "Enter" ? openModal() : undefined)}
>
    <h1>📄</h1>
    <p style="text-align: center;">View Logs</p>
    <span class="corner topleft" />
    <span class="corner topright" />
    <span class="corner botleft" />
    <span class="corner botright" />
</div>

<Modal bind:modal_active>
    <div class="inner">
        {#await logs_data}
            <p>Loading...</p>
        {:then data}
            <h1 style="line-height: 100%;">Recent logs</h1>
            <hr style="margin-top: 0.5em; margin-bottom: 1em" />

            {#each sort_logs(data.logs) as log}
                <p style="color: var(--error); margin-top: 1em;">
                    <span style="color: var(--foreground-secondary);">{new Date(log.time.secs_since_epoch * 1000).toString()}</span> 
                    <br/>
                    {log.log}
                </p>
            {/each}

        {:catch err}
            <p>Error loading config:</p>
            <p>{err}</p>
        {/await}
    </div>
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
        gap: 1em;
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

    .inner {
        padding: 2em;
        width: 100%;
        height: 100%;
        overflow-y: auto;
    }
</style>
