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
    <div class="inner">
        <slot></slot>
        <div class="corners">
            <div class="corner topleft" />
            <div class="corner topright" />
            <div class="corner botleft" />
            <div class="corner botright" /> 
        </div>
    </div>
</dialog>

<style>
    dialog {
        margin: auto;
        background-color: var(--background-secondary);
        color: var(--foreground);
        border: 5px solid var(--foreground-secondary);
        padding: 0em;
        overflow-x: auto;
        overflow-y: auto;
        position: relative;
        --hash-color: var(--foreground-secondary);
    }
    
    dialog::backdrop {
		background: rgba(0, 0, 0, 0.3);
	}


    .corner {
        content: "";
        width: 8px;
        height: 8px;
        position: absolute;
        top: 0px;
        left: 0px;
        background-color: var(--hash-color);
        clip-path: polygon(100% 0, 0 0, 0 100%);
        /* transform: translate(-1px, -1px); */
    }

    .corner.topright {
        top: 0px;
        bottom: unset;
        left: unset;
        right: 0px;
        transform: rotate(90deg);
    }

    .corner.botleft {
        top: unset;
        bottom: 0px;
        left: 0px;
        right: unset;
        transform: rotate(270deg);
    }

    .corner.botright {
        top: unset;
        bottom: 0px;
        left: unset;
        right: 0px;
        transform: rotate(180deg);
    }

    dialog .inner {
        position: relative;
        top: 0;
        left: 0;
    }
</style>