<script>
    import { goto } from "$app/navigation";

    async function home() {
        await goto("/");
    }
</script>

<main>
    <h1>Main Editor</h1>
    <button onclick={home}>Goback</button>
</main>
