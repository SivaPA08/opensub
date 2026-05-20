<script>
    import { goto } from "$app/navigation";
    import VideoPlayer from "$lib/player/VideoPlayer.svelte";

    async function home() {
        await goto("/");
    }
</script>

<main>
    <h1>Main Editor</h1>
    <button onclick={home}>Goback</button>
    <VideoPlayer />
</main>
