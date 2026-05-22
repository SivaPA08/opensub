<script>
    import { goto } from "$app/navigation";
    import VideoPlayer from "$lib/player/VideoPlayer.svelte";

    async function home() {
        await goto("/");
    }
</script>

<main>
    <div class="left">
        <!-- Section for setting -->
        <div class="top">
            <h2>Setting</h2>
            <button onclick={home}>Go Back</button>
        </div>
        <!-- section for timeline -->
        <div class="bottom">
            <h2>Timeline</h2>
        </div>
    </div>
    <div class="right">
        <VideoPlayer></VideoPlayer>
    </div>
</main>

<style>
    :global(body) {
        margin: 0;
    }
    main {
        display: grid;
        grid-template-columns: 3fr 1fr;

        height: 100vh;

        overflow: hidden;

        background: #111;
        color: white;
    }
    .left {
        display: grid;

        grid-template-rows: 3fr 2fr;

        height: 100%;

        border-right: 1px solid #2a2a2a;
    }

    .top {
        padding: 1rem;
        background: #1a1a1a;

        border-bottom: 1px solid #2a2a2a;
    }

    .bottom {
        padding: 1rem;
        background: #151515;
    }

    .right {
        display: flex;
        justify-content: center;
        align-items: center;

        background: black;

        overflow: hidden;

        padding: 1rem;
    }
</style>
