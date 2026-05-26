<script lang="ts">
    import { goto } from "$app/navigation";
    import VideoPlayer from "$lib/player/VideoPlayer.svelte";
    import TimeLine from "$lib/timeline/TimeLine.svelte";

    async function home(): Promise<void> {
        await goto("/");
    }

    let mainRef: HTMLElement | undefined = undefined;
    let leftWidthPercent: number = 75; // Left panel default width in percent
    let topHeightPx: number = 400;     // Top panel default height in pixels

    let isResizingHorizontal: boolean = false;
    let isResizingVertical: boolean = false;

    function startHorizontalResize(e: MouseEvent): void {
        e.preventDefault();
        isResizingHorizontal = true;
        window.addEventListener("mousemove", handleHorizontalResize as EventListener);
        window.addEventListener("mouseup", stopResize);
        document.body.style.cursor = "col-resize";
        document.body.style.userSelect = "none";
    }

    function handleHorizontalResize(e: MouseEvent): void {
        if (!isResizingHorizontal || !mainRef) return;
        const rect = mainRef.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        let percent = (mouseX / rect.width) * 100;
        // Restrict sizing boundaries to keep both panels visible
        leftWidthPercent = Math.max(30, Math.min(85, percent));
    }

    function startVerticalResize(e: MouseEvent): void {
        e.preventDefault();
        isResizingVertical = true;
        window.addEventListener("mousemove", handleVerticalResize as EventListener);
        window.addEventListener("mouseup", stopResize);
        document.body.style.cursor = "row-resize";
        document.body.style.userSelect = "none";
    }

    function handleVerticalResize(e: MouseEvent): void {
        if (!isResizingVertical || !mainRef) return;
        const rect = mainRef.getBoundingClientRect();
        const mouseY = e.clientY - rect.top;
        // Restrict sizing boundaries to keep both panels visible
        topHeightPx = Math.max(150, Math.min(rect.height - 150, mouseY));
    }

    function stopResize(): void {
        isResizingHorizontal = false;
        isResizingVertical = false;
        window.removeEventListener("mousemove", handleHorizontalResize as EventListener);
        window.removeEventListener("mousemove", handleVerticalResize as EventListener);
        window.removeEventListener("mouseup", stopResize);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
    }
</script>

<main bind:this={mainRef} style="grid-template-columns: {leftWidthPercent}% 4px 1fr;">
    <div class="left" style="grid-template-rows: {topHeightPx}px 4px 1fr;">
        <!-- Section for setting -->
        <div class="top">
            <h2>Setting</h2>
            <button onclick={home}>Go Back</button>
        </div>

        <!-- Horizontal Resizer Divider -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="resizer horizontal" onmousedown={startVerticalResize}></div>

        <!-- section for timeline -->
        <div class="bottom">
            <TimeLine></TimeLine>
        </div>
    </div>

    <!-- Vertical Resizer Divider -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="resizer vertical" onmousedown={startHorizontalResize}></div>

    <div class="right">
        <VideoPlayer></VideoPlayer>
    </div>
</main>

<style>
    main {
        display: grid;

        height: 100vh;

        background: #111;

        color: white;

        overflow: hidden;
    }

    .left {
        display: grid;

        min-height: 0;
    }

    .top {
        padding: 1rem;

        background: #1a1a1a;

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

    /* =========================
	   RESIZERS
	========================= */
    .resizer {
        background: #232833;
        transition: background-color 0.1s ease;
        z-index: 10;
        flex-shrink: 0;
    }

    .resizer:hover, .resizer:active {
        background: #00bcd4; /* Sleek cyan color on focus */
    }

    .resizer.vertical {
        cursor: col-resize;
        width: 4px;
        height: 100%;
    }

    .resizer.horizontal {
        cursor: row-resize;
        height: 4px;
        width: 100%;
    }
</style>
