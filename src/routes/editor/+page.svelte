<script>
    import { goto } from "$app/navigation";
    import VideoPlayer from "$lib/player/VideoPlayer.svelte";
    import TimeLine from "$lib/timeline/TimeLine.svelte";

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
            <!-- <h2>Timeline</h2> -->
            <TimeLine></TimeLine>
        </div>
    </div>
    <div class="right">
        <VideoPlayer></VideoPlayer>
    </div>
</main>

<style>
    main {
        display: grid;

        grid-template-columns: 3fr 1fr;

        height: 100vh;

        background: #111;

        color: white;

        overflow: hidden;
    }

    .left {
        display: grid;

        grid-template-rows: 400px 1fr;

        min-height: 0;

        border-right: 1px solid #2a2a2a;
    }

    .top {
        padding: 1rem;

        background: #1a1a1a;

        border-bottom: 1px solid #2a2a2a;

        overflow: auto;
    }

    .bottom {
        background: #151515;

        min-height: 0;

        overflow: hidden;

        display: flex;
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
