<script lang="ts">
    import Modal from "./Modal.svelte";

    $: config_data = get_config_api_call();

    function get_config_api_call() {
        return fetch(`${window.location.origin}/api/config`).then(async (e) => {
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
        config_data = get_config_api_call();
        modal_active = true;
    }

    let form: HTMLFormElement;

    function clearForm() {}

    let error_text: string;

    async function submit() {
        error_text = "";
        let form_data = Object.fromEntries(
            Array.from(form.getElementsByTagName("input")).map((e) => [
                e.name,
                e.type === "number" ? e.valueAsNumber : e.value,
            ])
        );

        let res = await fetch(`${window.location.origin}/api/config`, {
            method: "POST",
            headers: [["Content-Type", "application/json"]],
            body: JSON.stringify(form_data),
        });

        if (res.ok) {
            modal_active = false;
        } else {
            error_text = `Error submitting config: ${res.status} ${
                res.statusText
            } - ${await res.text()}`;

            if(res.status === 404){
                error_text = `Invalid configuration supplied - ensure all inputs have correct values`
            }
        }
    }

    function js_type_to_input(type: string) {
        if (type === "string") return "text";
        return type;
    }
</script>

<div
    class="add-hub-card"
    style="--hash-color: var(--foreground-secondary)"
    on:click={openModal}
    on:keydown={(e) => (e.key === "Enter" ? openModal() : undefined)}
>
    <h1>⚙</h1>
    <p style="text-align: center;">Settings</p>
    <span class="corner topleft" />
    <span class="corner topright" />
    <span class="corner botleft" />
    <span class="corner botright" />
</div>

<Modal bind:modal_active on:close_modal={clearForm}>
    <div class="inner">
        {#await config_data}
            <p>Loading...</p>
        {:then data}
            <h1 style="line-height: 100%;">Edit Settings</h1>
            <p>Settings will be applied on next restart</p>
            <hr style="margin-top: 0.5em;" />

            <form bind:this={form} on:submit|preventDefault={submit}>
                {#each Object.entries(data) as [key, value]}
                    <span>
                        <label for={`form_${key}`}>{key}</label>
                        <input
                            id={`form_${key}`}
                            name={`${key}`}
                            type={js_type_to_input(typeof value)}
                            {value}
                            placeholder={value.toString()}
                        />
                    </span>
                {/each}
                <button> Set </button>

                {#if error_text}
                    <p style="color: var(--error);">{error_text}</p>
                {/if}
            </form>
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
