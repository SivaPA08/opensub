<script lang="ts">
    import { onMount } from "svelte";
    import { videoDuration, videoCurrentTime } from "../store.js";

    type Clip = {
        id: number;
        track: number;
        start: number;
        length: number;
        color: string;
        title: string;
    };

    type DragState = {
        id: number;
        x: number;
        y: number;
        start: number;
        track: number;
    } | null;

    type ResizeSide = "left" | "right";

    type ResizeState = {
        id: number;
        side: ResizeSide;
        x: number;
        start: number;
        length: number;
    } | null;

    let zoom = 3;

    let tracks: number[] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let clips: Clip[] = [
        {
            id: 1,
            track: 0,
            start: 0,
            length: 80,
            color: "#6a8f63",
            title: "Image",
        },
        {
            id: 2,
            track: 1,
            start: 20,
            length: 90,
            color: "#6b63d9",
            title: "Text",
        },
        {
            id: 3,
            track: 2,
            start: 70,
            length: 100,
            color: "#4f7cff",
            title: "Video",
        },
    ];

    let drag: DragState = null;
    let resize: ResizeState = null;

    let tracksRef: HTMLDivElement;
    let rulerRef: HTMLDivElement;

    /* =========================
	   SCROLL SYNC
	========================= */

    onMount(() => {
        if (!tracksRef || !rulerRef) return;

        tracksRef.addEventListener("scroll", () => {
            rulerRef.scrollLeft = tracksRef.scrollLeft;
        });
    });

    /* =========================
	   DRAGGING
	========================= */

    function down(e: MouseEvent, clip: Clip): void {
        e.stopPropagation();

        drag = {
            id: clip.id,
            x: e.clientX,
            y: e.clientY,
            start: clip.start,
            track: clip.track,
        };

        window.addEventListener("mousemove", move);

        window.addEventListener("mouseup", up);
    }

    function move(e: MouseEvent): void {
        if (!drag) return;

        let dx = e.clientX - drag.x;

        let dy = e.clientY - drag.y;

        let clip = clips.find((x) => x.id === drag!.id);

        if (!clip) return;

        clip.start = Math.max(0, drag.start + Math.round(dx / zoom));

        let newTrack = drag.track + Math.round(dy / 60);

        newTrack = Math.max(0, Math.min(tracks.length - 1, newTrack));

        clip.track = newTrack;

        clips = [...clips];
    }

    function up(): void {
        window.removeEventListener("mousemove", move);

        window.removeEventListener("mouseup", up);

        drag = null;
    }

    /* =========================
	   RESIZE
	========================= */

    function resizeStart(e: MouseEvent, clip: Clip, side: ResizeSide): void {
        e.stopPropagation();

        resize = {
            id: clip.id,
            side,
            x: e.clientX,
            start: clip.start,
            length: clip.length,
        };

        window.addEventListener("mousemove", resizeMove);

        window.addEventListener("mouseup", resizeEnd);
    }

    function resizeMove(e: MouseEvent): void {
        if (!resize) return;

        let dx = Math.round((e.clientX - resize.x) / zoom);

        let clip = clips.find((x) => x.id === resize!.id);

        if (!clip) return;

        if (resize.side === "right") {
            clip.length = Math.max(20, resize.length + dx);
        } else {
            let newStart = resize.start + dx;

            let newLength = resize.length - dx;

            if (newLength > 20 && newStart >= 0) {
                clip.start = newStart;
                clip.length = newLength;
            }
        }

        clips = [...clips];
    }

    function resizeEnd(): void {
        window.removeEventListener("mousemove", resizeMove);

        window.removeEventListener("mouseup", resizeEnd);

        resize = null;
    }

    let isDraggingPlayhead = false;

    function startPlayheadDrag(e: MouseEvent): void {
        e.stopPropagation();
        e.preventDefault();
        isDraggingPlayhead = true;

        window.addEventListener("mousemove", movePlayhead);
        window.addEventListener("mouseup", stopPlayheadDrag);
    }

    function movePlayhead(e: MouseEvent): void {
        if (!isDraggingPlayhead || !tracksRef) return;

        const rect = tracksRef.getBoundingClientRect();
        const scrollLeft = tracksRef.scrollLeft;

        const canvasX = e.clientX - rect.left + scrollLeft;
        const timelineX = canvasX - 120; // 120px label offset

        let newTime = timelineX / zoom;
        const duration = $videoDuration || 120;
        newTime = Math.max(0, Math.min(duration, newTime));

        $videoCurrentTime = newTime;
    }

    function stopPlayheadDrag(): void {
        isDraggingPlayhead = false;
        window.removeEventListener("mousemove", movePlayhead);
        window.removeEventListener("mouseup", stopPlayheadDrag);
    }

    function formatTime(seconds: number): string {
        if (isNaN(seconds) || seconds === Infinity) return "00:00.0";
        const m = Math.floor(seconds / 60).toString().padStart(2, "0");
        const s = Math.floor(seconds % 60).toString().padStart(2, "0");
        const ms = Math.floor((seconds % 1) * 10).toString();
        return `${m}:${s}.${ms}`;
    }
</script>

<div class="timeline">
    <div class="zoom-bar">
        <span> Zoom </span>

        <input type="range" min="1" max="20" step="0.5" bind:value={zoom} />

        <span>
            {zoom.toFixed(1)}x
        </span>

        <div class="time-display">
            <span class="current">{formatTime($videoCurrentTime)}</span>
            <span class="separator">/</span>
            <span class="duration">{formatTime($videoDuration)}</span>
        </div>
    </div>

    <div class="ruler" bind:this={rulerRef}>
        <div class="left-space"></div>

        <div class="ruler-content" style="width: {($videoDuration || 120) * zoom}px;">
            {#each Array(Math.ceil(($videoDuration || 120) / 60)) as _, i}
                <div
                    class="tick"
                    style="
						width:{60 * zoom}px
					"
                >
                    {Math.floor((i * 60) / 3600).toString().padStart(2, "0")}:
                    {Math.floor(((i * 60) % 3600) / 60).toString().padStart(2, "0")}:00
                </div>
            {/each}
        </div>
    </div>

    <div class="tracks" bind:this={tracksRef}>
        <!-- Playhead Tracker Line and Drag Handle -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div 
            class="playhead" 
            style="left: {($videoCurrentTime * zoom) + 120}px;"
        >
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
                class="playhead-handle" 
                on:mousedown={startPlayheadDrag}
            ></div>
        </div>

        {#each tracks as track}
            <div class="track">
                <div class="label">
                    Track {track + 1}
                </div>

                <div class="content" style="width: {($videoDuration || 120) * zoom}px;">
                    {#each clips.filter((c) => c.track === track) as clip}
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div
                            class="clip"
                            role="button"
                            tabindex="0"
                            style="
								left:{clip.start * zoom}px;
								width:{clip.length * zoom}px;
								background:{clip.color};
							"
                            on:mousedown={(e) => down(e, clip)}
                            on:keydown={(e) => {
                                if (e.key === "Enter" || e.key === " ") {
                                    e.preventDefault();
                                }
                            }}
                        >
                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <div
                                class="resize left"
                                role="button"
                                tabindex="-1"
                                aria-label="Resize left"
                                on:mousedown={(e) =>
                                    resizeStart(e, clip, "left")}
                            ></div>

                            <div class="title">
                                {clip.title}
                            </div>

                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <div
                                class="resize right"
                                role="button"
                                tabindex="-1"
                                aria-label="Resize right"
                                on:mousedown={(e) =>
                                    resizeStart(e, clip, "right")}
                            ></div>
                        </div>
                    {/each}
                </div>
            </div>
        {/each}
    </div>

</div>

<style>
    .timeline {
        width: 100%;
        height: 100%;

        display: flex;
        flex-direction: column;

        background: #131722;

        overflow: hidden;

        isolation: isolate;

        transform: translateZ(0);
    }

    /* =========================
	   RULER
	========================= */

    .ruler {
        height: 36px;

        display: flex;

        flex-shrink: 0;

        background: #181d28;

        border-bottom: 1px solid #262c38;

        overflow: hidden;
    }

    .left-space {
        width: 120px;

        flex-shrink: 0;

        background: #181d28;

        border-right: 1px solid #262c38;
    }

    .ruler-content {
        display: flex;

        flex-shrink: 0;
    }

    .tick {
        flex-shrink: 0;

        border-left: 1px solid #2a3140;

        box-sizing: border-box;

        padding-left: 6px;
        padding-top: 8px;

        font-size: 11px;

        color: #8f96a8;
    }

    /* =========================
	   TRACKS
	========================= */

    .tracks {
        position: relative;

        flex: 1;

        overflow-x: auto;
        overflow-y: auto;

        background: #171b26;

        scrollbar-gutter: stable;

        will-change: scroll-position;

        contain: strict;
    }

    .track {
        height: 60px;

        display: flex;

        border-bottom: 1px solid #262c38;
    }

    /* =========================
	   LABELS
	========================= */

    .label {
        width: 120px;

        flex-shrink: 0;

        display: flex;
        align-items: center;

        padding-left: 14px;

        box-sizing: border-box;

        background: #171c27;

        border-right: 1px solid #262c38;

        font-size: 14px;

        color: #c7ccda;
    }

    /* =========================
	   CONTENT
	========================= */

    .content {
        position: relative;

        height: 100%;

        flex-shrink: 0;

        background: repeating-linear-gradient(
            to right,
            #171b26 0px,
            #171b26 179px,
            #232937 180px
        );
    }

    /* =========================
	   CLIPS
	========================= */

    .clip {
        position: absolute;

        top: 8px;

        height: 42px;

        display: flex;
        align-items: center;
        justify-content: center;

        box-sizing: border-box;

        border-radius: 6px;

        font-size: 13px;
        font-weight: 600;

        color: white;

        cursor: grab;

        user-select: none;

        white-space: nowrap;

        box-shadow:
            0 2px 8px rgba(0, 0, 0, 0.35),
            inset 0 0 0 1px rgba(255, 255, 255, 0.08);

        transition:
            filter 0.08s ease,
            box-shadow 0.08s ease;
    }

    .clip:hover {
        filter: brightness(1.08);
    }

    .title {
        width: 100%;

        text-align: center;

        pointer-events: none;
    }

    /* =========================
	   RESIZE HANDLES
	========================= */

    .resize {
        position: absolute;

        top: 0;
        bottom: 0;

        width: 8px;

        z-index: 20;

        cursor: ew-resize;
    }

    .resize.left {
        left: 0;
    }

    .resize.right {
        right: 0;
    }

    /* =========================
	   ZOOM BAR
	========================= */

    .zoom-bar {
        height: 42px;

        display: flex;
        align-items: center;
        gap: 12px;

        padding: 0 14px;

        background: #151925;

        border-bottom: 1px solid #262c38;

        color: #c7ccda;

        flex-shrink: 0;
    }

    .zoom-bar input {
        width: 180px;
    }

    /* =========================
	   SCROLLBAR
	========================= */

    .tracks::-webkit-scrollbar {
        height: 10px;
        width: 10px;
    }

    .tracks::-webkit-scrollbar-track {
        background: #11151d;
    }

    .tracks::-webkit-scrollbar-thumb {
        background: #2e3647;

        border-radius: 999px;
    }

    .tracks::-webkit-scrollbar-thumb:hover {
        background: #3b465c;
    }

    /* =========================
	   PLAYHEAD TRACKER
	========================= */
    .playhead {
        position: absolute;
        top: 0;
        bottom: 0;
        width: 2px;
        background: #ff4a4a;
        z-index: 100;
        pointer-events: none;
    }

    .playhead-handle {
        position: absolute;
        top: 0;
        left: -8px;
        width: 16px;
        height: 16px;
        background: #ff4a4a;
        border-radius: 50% 50% 50% 0;
        transform: rotate(-45deg);
        cursor: grab;
        pointer-events: auto;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: center;
        justify-content: center;
        transition: transform 0.1s ease, background-color 0.1s ease;
    }

    .playhead-handle:hover {
        background: #ff6b6b;
        transform: rotate(-45deg) scale(1.1);
    }

    .playhead-handle:active {
        cursor: grabbing;
        background: #e63939;
        transform: rotate(-45deg) scale(1.15);
    }

    /* =========================
	   TIME DISPLAY
	========================= */
    .time-display {
        margin-left: auto;
        display: flex;
        align-items: center;
        gap: 6px;
        font-family: 'JetBrains Mono', 'Courier New', Courier, monospace;
        font-size: 13px;
        color: #8f96a8;
        background: #1c2130;
        padding: 4px 10px;
        border-radius: 6px;
        border: 1px solid #2a3145;
    }

    .time-display .current {
        color: #ffffff;
        font-weight: 600;
    }

    .time-display .separator {
        color: #4a5268;
    }

    .time-display .duration {
        color: #8f96a8;
    }
</style>
