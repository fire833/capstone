<script lang="ts">

    let response: string;


    fetch("/api/hubs").then(e => e.json()).then(e => {
        response = JSON.stringify(e);
    }).catch(exp => {
        console.error(exp);
    })
</script>


<h1>Welcome to SvelteKit 3!</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>

<p>Response: {response}</p>