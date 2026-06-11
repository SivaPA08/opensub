<script lang="ts">
    import { onMount } from "svelte";
    import { videoDuration, videoCurrentTime, subtitle, selectedSubtitleIndices, updateSubtitleProperties, pushUndoSnapshot, setSubtitles } from "../store.js";

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

    let zoom = 10;
    let prevDuration = 0;
    $: if ($videoDuration && $videoDuration !== prevDuration) {
        prevDuration = $videoDuration;
        // Optimized auto-zoom defaults so that small/word-per-frame clips are beautifully visible initially
        if ($videoDuration < 15) {
            zoom = 300;
        } else if ($videoDuration < 60) {
            zoom = 150;
        } else if ($videoDuration < 300) {
            zoom = 60;
        } else {
            zoom = 20;
        }
    }

    $: timelineDuration = Math.max(($videoDuration || 120) + 30, 120);

    // Calculate dynamic tick intervals based on zoom depth
    $: tickInterval = zoom >= 350 ? 1 : zoom >= 150 ? 2 : zoom >= 80 ? 5 : zoom >= 30 ? 10 : zoom >= 10 ? 30 : 60;

    function formatTickTime(seconds: number): string {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        
        const mStr = m.toString().padStart(2, "0");
        const sStr = s.toString().padStart(2, "0");
        
        if (h > 0) {
            return `${h}:${mStr}:${sStr}`;
        }
        return `${mStr}:${sStr}`;
    }

    let lastClampedDuration = 0;
    $: if ($videoDuration && $videoDuration !== lastClampedDuration) {
        lastClampedDuration = $videoDuration;
        clips = clips.map(clip => {
            let start = Math.min(clip.start, $videoDuration);
            let length = Math.min(clip.length, $videoDuration - start);
            if (length < 0.1) {
                length = 0.1;
            }
            if (start < 0) {
                start = 0;
            }
            return { ...clip, start, length };
        });
    }

    let tracks: number[] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let clips: Clip[] = [];

    // Sync from $subtitle store to local clips
    let lastSubtitlesKey = "";
    $: {
        const currentKey = $subtitle.map(s => `${s.start}-${s.end}-${s.content}`).join('|');
        if (currentKey !== lastSubtitlesKey && !drag && !resize) {
            lastSubtitlesKey = currentKey;
            clips = $subtitle.map((sub, index) => ({
                id: index + 1,
                track: 0, // Subtitles on track 1 (index 0)
                start: sub.start,
                length: sub.end - sub.start,
                color: "#6b63d9", // Beautiful deep purple for subtitles
                title: sub.content,
            }));
        }
    }

    // Sync from local clips back to $subtitle store in real-time
    $: if (clips && (drag || resize)) {
        const updated = clips.map((clip, index) => {
            // Keep original properties and update positioning
            const original = $subtitle[index] || {};
            return {
                ...original,
                start: clip.start,
                end: clip.start + clip.length,
                content: clip.title
            };
        });
        setSubtitles(updated, { recordUndo: false });
        // Sync our serialization key so we avoid triggering the store-to-clips reactive block
        lastSubtitlesKey = updated.map(s => `${s.start}-${s.end}-${s.content}`).join('|');
    }

    let drag: DragState = null;
    let resize: ResizeState = null;

    type MultiDragState = {
        id: number;
        start: number;
        track: number;
    };
    let multiDragStates: MultiDragState[] = [];

    type MultiResizeState = {
        id: number;
        start: number;
        length: number;
    };
    let multiResizeStates: MultiResizeState[] = [];

    let isBoxSelecting = false;
    let boxStart = { x: 0, y: 0 };
    let boxCurrent = { x: 0, y: 0 };

    let tracksRef: HTMLDivElement;
    let rulerRef: HTMLDivElement;

    /* =========================
	   SCROLL SYNC AND KEYBOARD
	========================= */

    function handleKeyDown(e: KeyboardEvent): void {
        if (e.key === "Escape") {
            selectedSubtitleIndices.set([]);
        }
    }

    onMount(() => {
        if (!tracksRef || !rulerRef) return;

        tracksRef.addEventListener("scroll", () => {
            rulerRef.scrollLeft = tracksRef.scrollLeft;
        });

        window.addEventListener("keydown", handleKeyDown);
        return () => {
            window.removeEventListener("keydown", handleKeyDown);
        };
    });

    /* =========================
	   DRAGGING (MULTI-DRAG)
	========================= */

    function down(e: MouseEvent, clip: Clip): void {
        e.stopPropagation();
        pushUndoSnapshot(true);

        const index = clip.id - 1;

        selectedSubtitleIndices.update(existing => {
            if (e.shiftKey) {
                if (existing.includes(index)) {
                    return existing.filter(i => i !== index);
                } else {
                    return [...existing, index];
                }
            } else {
                if (existing.includes(index)) {
                    return existing;
                }
                return [index];
            }
        });

        const selectedClips = clips.filter((_, idx) => $selectedSubtitleIndices.includes(idx));

        drag = {
            id: clip.id,
            x: e.clientX,
            y: e.clientY,
            start: clip.start,
            track: clip.track,
        };

        multiDragStates = selectedClips.map(c => ({
            id: c.id,
            start: c.start,
            track: c.track
        }));

        window.addEventListener("mousemove", move);
        window.addEventListener("mouseup", up);
    }

    function move(e: MouseEvent): void {
        if (!drag) return;

        let dx = e.clientX - drag.x;
        let dy = e.clientY - drag.y;

        const duration = $videoDuration || 120;
        const deltaStart = dx / zoom;
        const deltaTrack = Math.round(dy / 60);

        clips = clips.map(clip => {
            const dragInfo = multiDragStates.find(x => x.id === clip.id);
            if (dragInfo) {
                let newStart = dragInfo.start + deltaStart;
                newStart = Math.max(0, Math.min(duration - clip.length, newStart));
                
                let newTrack = dragInfo.track + deltaTrack;
                newTrack = Math.max(0, Math.min(tracks.length - 1, newTrack));
                
                return {
                    ...clip,
                    start: parseFloat(newStart.toFixed(3)),
                    track: newTrack
                };
            }
            return clip;
        });
    }

    function up(): void {
        window.removeEventListener("mousemove", move);
        window.removeEventListener("mouseup", up);
        drag = null;
        multiDragStates = [];
    }

    /* =========================
	   RESIZE (MULTI-RESIZE)
	========================= */

    function resizeStart(e: MouseEvent, clip: Clip, side: ResizeSide): void {
        e.stopPropagation();
        pushUndoSnapshot(true);

        const index = clip.id - 1;
        selectedSubtitleIndices.update(existing => {
            if (existing.includes(index)) {
                return existing;
            }
            return [index];
        });

        const selectedClips = clips.filter((_, idx) => $selectedSubtitleIndices.includes(idx));

        resize = {
            id: clip.id,
            side,
            x: e.clientX,
            start: clip.start,
            length: clip.length,
        };

        multiResizeStates = selectedClips.map(c => ({
            id: c.id,
            start: c.start,
            length: c.length
        }));

        window.addEventListener("mousemove", resizeMove);
        window.addEventListener("mouseup", resizeEnd);
    }

    function resizeMove(e: MouseEvent): void {
        if (!resize) return;

        let dx = (e.clientX - resize.x) / zoom;
        const duration = $videoDuration || 120;

        clips = clips.map(clip => {
            const resizeInfo = multiResizeStates.find(x => x.id === clip.id);
            if (resizeInfo) {
                if (resize!.side === "right") {
                    let newLength = resizeInfo.length + dx;
                    const maxLength = duration - clip.start;
                    return {
                        ...clip,
                        length: Math.max(0.1, Math.min(maxLength, parseFloat(newLength.toFixed(3))))
                    };
                } else {
                    let newStart = resizeInfo.start + dx;
                    let newLength = resizeInfo.length - dx;
                    if (newLength > 0.1 && newStart >= 0) {
                        return {
                            ...clip,
                            start: parseFloat(newStart.toFixed(3)),
                            length: parseFloat(newLength.toFixed(3))
                        };
                    }
                }
            }
            return clip;
        });
    }

    function resizeEnd(): void {
        window.removeEventListener("mousemove", resizeMove);
        window.removeEventListener("mouseup", resizeEnd);
        resize = null;
        multiResizeStates = [];
    }

    /* =========================
	   BOX SELECTION (RUBBER BAND)
	========================= */

    function startBoxSelection(e: MouseEvent): void {
        // Only left click starts selection
        if (e.button !== 0 || !tracksRef) return;

        const rect = tracksRef.getBoundingClientRect();
        const scrollLeft = tracksRef.scrollLeft;
        const scrollTop = tracksRef.scrollTop;

        const clickX = e.clientX - rect.left + scrollLeft;
        const clickY = e.clientY - rect.top + scrollTop;

        // Only start if click is in tracks content area (right of 120px label)
        if (clickX < 120) return;

        isBoxSelecting = true;
        boxStart = { x: clickX, y: clickY };
        boxCurrent = { x: clickX, y: clickY };

        if (!e.shiftKey) {
            selectedSubtitleIndices.set([]);
        }

        window.addEventListener("mousemove", handleBoxSelectionMove);
        window.addEventListener("mouseup", handleBoxSelectionEnd);
    }

    function handleBoxSelectionMove(e: MouseEvent): void {
        if (!isBoxSelecting || !tracksRef) return;

        const rect = tracksRef.getBoundingClientRect();
        const scrollLeft = tracksRef.scrollLeft;
        const scrollTop = tracksRef.scrollTop;

        boxCurrent = {
            x: Math.max(120, e.clientX - rect.left + scrollLeft),
            y: e.clientY - rect.top + scrollTop
        };

        const selectMinX = Math.min(boxStart.x, boxCurrent.x) - 120;
        const selectMaxX = Math.max(boxStart.x, boxCurrent.x) - 120;
        const selectMinY = Math.min(boxStart.y, boxCurrent.y);
        const selectMaxY = Math.max(boxStart.y, boxCurrent.y);

        const overlappingIndices: number[] = [];
        clips.forEach((clip, index) => {
            const clipLeft = clip.start * zoom;
            const clipRight = (clip.start + clip.length) * zoom;
            const clipTop = clip.track * 61 + 8;
            const clipBottom = clip.track * 61 + 8 + 42;

            const overlapsX = clipLeft < selectMaxX && clipRight > selectMinX;
            const overlapsY = clipTop < selectMaxY && clipBottom > selectMinY;

            if (overlapsX && overlapsY) {
                overlappingIndices.push(index);
            }
        });

        if (e.shiftKey) {
            selectedSubtitleIndices.update(existing => {
                const unique = new Set([...existing, ...overlappingIndices]);
                return Array.from(unique);
            });
        } else {
            selectedSubtitleIndices.set(overlappingIndices);
        }
    }

    function handleBoxSelectionEnd(): void {
        isBoxSelecting = false;
        window.removeEventListener("mousemove", handleBoxSelectionMove);
        window.removeEventListener("mouseup", handleBoxSelectionEnd);
    }

    function selectAll(): void {
        selectedSubtitleIndices.set(clips.map((_, index) => index));
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

        <input type="range" min="2" max="1200" step="1" bind:value={zoom} />

        <span>
            {zoom.toFixed(1)}x
        </span>

        <button class="select-all-btn" on:click={selectAll}>
            Select All
        </button>

        <div class="time-display">
            <span class="current">{formatTime($videoCurrentTime)}</span>
            <span class="separator">/</span>
            <span class="duration">{formatTime($videoDuration)}</span>
        </div>
    </div>

    <div class="ruler" bind:this={rulerRef}>
        <div class="left-space"></div>

        <div class="ruler-content" style="width: {timelineDuration * zoom}px;">
            {#each Array(Math.ceil(timelineDuration / tickInterval)) as _, i}
                <div
                    class="tick"
                    style="
						width:{tickInterval * zoom}px
					"
                >
                    {formatTickTime(i * tickInterval)}
                </div>
            {/each}
        </div>
    </div>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="tracks" bind:this={tracksRef} on:mousedown={startBoxSelection}>
        <div class="tracks-inner" style="width: {timelineDuration * zoom + 120}px;">
            <!-- Selection Box Overlay -->
            {#if isBoxSelecting}
                <div
                    class="selection-box"
                    style="
                        left: {Math.min(boxStart.x, boxCurrent.x)}px;
                        top: {Math.min(boxStart.y, boxCurrent.y)}px;
                        width: {Math.abs(boxStart.x - boxCurrent.x)}px;
                        height: {Math.abs(boxStart.y - boxCurrent.y)}px;
                    "
                ></div>
            {/if}

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

            <!-- End of Video Marker Line -->
            {#if $videoDuration}
                <div 
                    class="video-end-line" 
                    style="left: {($videoDuration * zoom) + 120}px;"
                >
                    <div class="video-end-label">End of Video</div>
                </div>
            {/if}

            {#each tracks as track}
                <div class="track">
                    <div class="label">
                        Track {track + 1}
                    </div>

                    <div class="content" style="width: {timelineDuration * zoom}px; background-size: {tickInterval * zoom}px 100%;">
                        {#each clips.filter((c) => c.track === track) as clip}
                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <div
                                class="clip"
                                class:selected={$selectedSubtitleIndices.includes(clip.id - 1)}
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
                                    [{formatTime(clip.start)} - {formatTime(clip.start + clip.length)}] {clip.title}
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

    .tracks-inner {
        position: relative;
        min-height: 100%;
        display: flex;
        flex-direction: column;
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

        background-color: #171b26;
        background-image: linear-gradient(to right, #232937 1px, transparent 1px);
        background-repeat: repeat-x;
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

    .clip.selected {
        outline: 2.5px solid #00bcd4;
        outline-offset: 1px;
        box-shadow: 
            0 0 16px rgba(0, 188, 212, 0.65),
            0 2px 8px rgba(0, 0, 0, 0.45);
    }

    .title {
        width: 100%;
        text-align: center;
        pointer-events: none;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        padding: 0 8px;
        box-sizing: border-box;
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

    /* =========================
	   VIDEO END MARKER
	========================= */
    .video-end-line {
        position: absolute;
        top: 0;
        bottom: 0;
        width: 0;
        border-left: 2px dashed #ff4a4a;
        opacity: 0.55;
        z-index: 50;
        pointer-events: none;
    }

    .video-end-label {
        position: absolute;
        top: 6px;
        left: 6px;
        font-size: 10px;
        color: #ff4a4a;
        background: rgba(21, 25, 37, 0.85);
        padding: 2px 6px;
        border-radius: 4px;
        border: 1px solid rgba(255, 74, 74, 0.35);
        white-space: nowrap;
        font-weight: 700;
        letter-spacing: 0.05em;
        text-transform: uppercase;
        pointer-events: none;
    }

    /* =========================
	   SELECTION BOX & SELECT ALL
	========================= */
    .selection-box {
        position: absolute;
        background: rgba(0, 188, 212, 0.15);
        border: 1.5px solid #00bcd4;
        border-radius: 4px;
        pointer-events: none;
        z-index: 95;
        box-shadow: 0 0 8px rgba(0, 188, 212, 0.2);
    }

    .select-all-btn {
        background: rgba(0, 188, 212, 0.12);
        border: 1px solid rgba(0, 188, 212, 0.35);
        color: #00bcd4;
        font-size: 11px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        padding: 5px 12px;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .select-all-btn:hover {
        background: #00bcd4;
        color: #111;
        border-color: #00bcd4;
        box-shadow: 0 0 12px rgba(0, 188, 212, 0.45);
    }

    .select-all-btn:active {
        transform: scale(0.95);
    }
</style>
