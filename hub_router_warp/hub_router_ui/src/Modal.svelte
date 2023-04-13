<script lang="ts">
    import { createEventDispatcher } from "svelte";

    export let modal_active: boolean;

    let dialog: HTMLDialogElement;
    $: if (dialog && modal_active) {
        console.log("Opening because modal is now active");
        dialog.showModal()
    };
    $: if (dialog && !modal_active) {
        console.log("Closing because modal was no longer active");
        dialog.close();
    }


    const dispatch = createEventDispatcher();
</script>


<dialog bind:this={dialog} on:close={() => {modal_active = false; dispatch("close_modal");}} on:click|self={() => dialog.close()} on:keypress={() => {}}>
    <slot></slot>
</dialog>

<style>
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
    
    dialog::backdrop {
		background: rgba(0, 0, 0, 0.3);
	}
</style>