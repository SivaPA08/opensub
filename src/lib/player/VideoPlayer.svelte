<script lang="ts">
    import { onDestroy } from "svelte";
import { subtitleAnimation } from "../store.js";
    import { videoPath, videoDuration, videoCurrentTime, subtitle, subtitleFontSize, type Subtitle } from "../store.js";
    import { invoke } from "@tauri-apps/api/core";

    let videoElement: HTMLVideoElement;
    let videoSrc = "";

    onDestroy(() => {
        if (videoElement) {
            videoElement.pause();
            videoElement.src = "";
            videoElement.load();
        }
    });
    let errorMessage = "";
    let loading = false;
    let loaded = false;
    let prevPath = "";

    let videoWidth = 0;
    let videoHeight = 0;
    $: containerAspectRatio = videoWidth && videoHeight ? `${videoWidth} / ${videoHeight}` : '16 / 9';

    $: activeSubtitle = $subtitle.find(
        (sub) => $videoCurrentTime >= sub.start && $videoCurrentTime <= sub.end
    );

    let showSizeControls = false;
    let isEditingText = false;

    // Subtitle Custom Position & Size States (percentage values)
    let subX = 50; // Horizontal center percentage
    let subY = 85; // Vertical offset percentage
    let subWidth = 70; // Width percentage of the video wrapper

    let isDraggingSubtitle = false;
    let dragStartX = 0;
    let dragStartY = 0;
    let initialSubX = 50;
    let initialSubY = 85;

    let isResizingSubtitle = false;
    let resizeStartX = 0;
    let initialSubWidth = 70;

    let containerRef: HTMLDivElement;

    // Autofocus action for inline text editor
    function autofocus(node: HTMLTextAreaElement) {
        node.focus();
        node.select();
    }

    // Drag-to-Reposition Logic
    function startSubtitleDrag(e: MouseEvent) {
        const target = e.target as HTMLElement;
        if (target.classList.contains('resize-handle') || isEditingText) return;

        e.preventDefault();
        e.stopPropagation();
        isDraggingSubtitle = true;
        dragStartX = e.clientX;
        dragStartY = e.clientY;
        initialSubX = subX;
        initialSubY = subY;

        window.addEventListener('mousemove', handleSubtitleDrag);
        window.addEventListener('mouseup', stopSubtitleDrag);
    }

    function handleSubtitleDrag(e: MouseEvent) {
        if (!isDraggingSubtitle || !containerRef) return;

        const rect = containerRef.getBoundingClientRect();
        const dx = e.clientX - dragStartX;
        const dy = e.clientY - dragStartY;

        const pctDx = (dx / rect.width) * 100;
        const pctDy = (dy / rect.height) * 100;

        subX = Math.max(5, Math.min(95, initialSubX + pctDx));
        subY = Math.max(5, Math.min(95, initialSubY + pctDy));
    }

    function stopSubtitleDrag() {
        isDraggingSubtitle = false;
        window.removeEventListener('mousemove', handleSubtitleDrag);
        window.removeEventListener('mouseup', stopSubtitleDrag);
    }

    // Drag-to-Resize Logic
    function startSubtitleResize(e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();
        isResizingSubtitle = true;
        resizeStartX = e.clientX;
        initialSubWidth = subWidth;

        window.addEventListener('mousemove', handleSubtitleResize);
        window.addEventListener('mouseup', stopSubtitleResize);
    }

    function handleSubtitleResize(e: MouseEvent) {
        if (!isResizingSubtitle || !containerRef) return;

        const rect = containerRef.getBoundingClientRect();
        const dx = e.clientX - resizeStartX;
        const pctDx = (dx / rect.width) * 100 * 2; // Resize from center

        subWidth = Math.max(20, Math.min(95, initialSubWidth + pctDx));
    }

    function stopSubtitleResize() {
        isResizingSubtitle = false;
        window.removeEventListener('mousemove', handleSubtitleResize);
        window.removeEventListener('mouseup', stopSubtitleResize);
    }

    function addSubtitleAtPlayhead() {
        const currentTime = $videoCurrentTime;
        const duration = $videoDuration || 120;
        const end = Math.min(duration, currentTime + 2.0);

        const newSub: Subtitle = {
            start: parseFloat(currentTime.toFixed(3)),
            end: parseFloat(end.toFixed(3)),
            content: "New Subtitle segment"
        };

        subtitle.update(items => {
            const updated = [...items, newSub];
            return updated.sort((a, b) => a.start - b.start);
        });

        // Instantly focus and open editing interface
        isEditingText = true;
    }

    function handleSubtitleClick(e: MouseEvent) {
        showSizeControls = !showSizeControls;
    }

    function changeSize(delta: number) {
        subtitleAnimation.update(style => ({
            ...style,
            fontSize: Math.max(12, Math.min(80, style.fontSize + delta))
        }));
    }

    function updateSubtitleText(sub: Subtitle, newText: string) {
        subtitle.update(items => {
            return items.map(item => {
                if (item.start === sub.start && item.end === sub.end) {
                    return { ...item, content: newText };
                }
                return item;
            });
        });
    }

    function hexOrRgbToRgba(color: string, opacity: number): string {
        if (!color) return `rgba(0,0,0,${opacity})`;
        
        // If it's already rgba, replace the alpha
        if (color.startsWith('rgba')) {
            return color.replace(/[\d\.]+\)$/, `${opacity})`);
        }
        
        // If it's rgb, convert to rgba
        if (color.startsWith('rgb')) {
            return color.replace('rgb', 'rgba').replace(')', `, ${opacity})`);
        }
        
        // If it's hex (#fff or #ffffff)
        if (color.startsWith('#')) {
            let hex = color.slice(1);
            if (hex.length === 3) {
                hex = hex.split('').map(c => c + c).join('');
            }
            const r = parseInt(hex.slice(0, 2), 16) || 0;
            const g = parseInt(hex.slice(2, 4), 16) || 0;
            const b = parseInt(hex.slice(4, 6), 16) || 0;
            return `rgba(${r}, ${g}, ${b}, ${opacity})`;
        }
        
        return color; // Fallback
    }

    $: if ($videoPath && $videoPath !== prevPath) {
        prevPath = $videoPath;
        loading = true;
        errorMessage = "";
        loaded = false;
        videoSrc = "";
        showSizeControls = false;
        invoke<string>("get_streaming_url", { path: $videoPath })
            .then((url) => {
                videoSrc = url;
            })
            .catch((err) => {
                console.error("Failed to start video stream:", err);
                errorMessage = "Failed to initialize local HTTP streaming server.";
            })
            .finally(() => {
                loading = false;
            });
    } else if (!$videoPath) {
        videoSrc = "";
        prevPath = "";
    }
</script>

{#if loading}
    <div class="placeholder animate-fade-in">
        <div class="spinner"></div>
        <span class="loading-text">Preparing seamless local stream...</span>
    </div>
{:else if errorMessage}
    <div class="placeholder error animate-fade-in">
        <span class="error-icon">⚠️</span>
        <span class="error-text">{errorMessage}</span>
    </div>
{:else if videoSrc}
    <div class="video-wrapper animate-fade-in">
        <div class="video-container" bind:this={containerRef} style="aspect-ratio: {containerAspectRatio};">
            <!-- svelte-ignore a11y_media_has_caption -->
            <video
                src={videoSrc}
                controls
                autoplay
                preload="metadata"
                class="video"
                bind:this={videoElement}
                bind:duration={$videoDuration}
                bind:currentTime={$videoCurrentTime}
                bind:videoWidth={videoWidth}
                bind:videoHeight={videoHeight}
                on:loadedmetadata={() => {
                    loaded = true;
                    console.log("video metadata loaded");
                }}
                on:canplay={() => {
                    loaded = true;
                    console.log("video can play");
                }}
                on:canplaythrough={() => {
                    loaded = true;
                    console.log("video can play through");
                }}
                on:error={(event) => {
                    const mediaError = (event.currentTarget as HTMLVideoElement).error;
                    console.error("Video element error:", mediaError);
                    errorMessage = `Playback failed: ${mediaError ? mediaError.message : "file may be unsupported or corrupted."}`;
                    loaded = false;
                    videoSrc = "";
                }}
            >
                <track kind="captions" />
            </video>

            <!-- Subtitle Overlay -->
            <div class="subtitle-overlay">
                {#if activeSubtitle}
                    <!-- Floating Popover Controls -->
                    {#if showSizeControls}
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div class="size-controls-popover animate-scale-in" on:mousedown|stopPropagation>
                            <div class="popover-header">
                                <span class="popover-title">Subtitle Properties</span>
                                <button class="close-btn" on:click={() => showSizeControls = false}>×</button>
                            </div>
                            
                            <div class="popover-section">
                                <span class="section-label">Font Size: <span class="val-highlight">{$subtitleAnimation.fontSize}px</span></span>
                                <div class="slider-row">
                                    <button class="adjust-btn" on:click={() => changeSize(-2)}>A-</button>
                                    <input 
                                        type="range" 
                                        min="12" 
                                        max="80" 
                                        bind:value={$subtitleAnimation.fontSize} 
                                        class="size-slider"
                                    />
                                    <button class="adjust-btn" on:click={() => changeSize(2)}>A+</button>
                                </div>
                            </div>

                            <div class="popover-section">
                                <span class="section-label">Edit text content:</span>
                                <textarea 
                                    value={activeSubtitle.content}
                                    on:input={(e) => updateSubtitleText(activeSubtitle, e.currentTarget.value)}
                                    class="edit-textarea"
                                    rows="2"
                                ></textarea>
                            </div>
                        </div>
                    {/if}

                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div 
                        class="subtitle-block animate-scale-in"
                        style="
                            left: {subX}%; 
                            top: {subY}%; 
                            width: {subWidth}%; 
                            transform: translate(-50%, -50%);
                            background: {hexOrRgbToRgba($subtitleAnimation.backgroundColor, $subtitleAnimation.backgroundOpacity)};
                            font-family: {$subtitleAnimation.customFont};
                            font-size: {$subtitleAnimation.fontSize}px;
                            color: {hexOrRgbToRgba($subtitleAnimation.fontColor, $subtitleAnimation.fontOpacity)};
                            -webkit-backdrop-filter: blur({8 * $subtitleAnimation.backgroundOpacity}px);
                            backdrop-filter: blur({8 * $subtitleAnimation.backgroundOpacity}px);
                            border: 1px solid rgba(255, 255, 255, {0.1 * $subtitleAnimation.backgroundOpacity});
                            box-shadow: 0 8px 32px rgba(0, 0, 0, {0.6 * $subtitleAnimation.backgroundOpacity}), inset 0 0 0 1px rgba(255, 255, 255, {0.15 * $subtitleAnimation.backgroundOpacity});
                        "
                        on:mousedown={startSubtitleDrag}
                        on:click|stopPropagation={handleSubtitleClick}
                    >
                        {#if isEditingText}
                            <textarea
                                class="subtitle-textarea"
                                style="font-size: {$subtitleAnimation.fontSize}px; color: {hexOrRgbToRgba($subtitleAnimation.fontColor, $subtitleAnimation.fontOpacity)}; font-family: {$subtitleAnimation.customFont};"
                                value={activeSubtitle.content}
                                on:input={(e) => updateSubtitleText(activeSubtitle, e.currentTarget.value)}
                                on:blur={() => isEditingText = false}
                                on:click|stopPropagation
                                on:keydown={(e) => {
                                    if (e.key === 'Enter' && !e.shiftKey) {
                                        e.preventDefault();
                                        isEditingText = false;
                                    }
                                }}
                                use:autofocus
                            ></textarea>
                        {:else}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <div 
                                class="subtitle-text-render"
                                style="font-size: {$subtitleAnimation.fontSize}px; color: {hexOrRgbToRgba($subtitleAnimation.fontColor, $subtitleAnimation.fontOpacity)}; font-family: {$subtitleAnimation.customFont};"
                                on:dblclick={() => isEditingText = true}
                            >
                                {activeSubtitle.content}
                                <span class="edit-hint">Double-click to edit text</span>
                            </div>
                        {/if}

                        <!-- Resize Handle in Bottom-Right Corner -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div class="resize-handle" on:mousedown={startSubtitleResize}></div>
                    </div>
                {:else}
                    <!-- Insert new subtitle at playhead if no segment is active -->
                    <button class="add-subtitle-btn animate-fade-in" on:click|stopPropagation={addSubtitleAtPlayhead}>
                        <span class="plus-icon">+</span> Add Subtitle
                    </button>
                {/if}
            </div>
        </div>
        <div class="status">
            {#if loaded}
                <span class="success-dot"></span> Streaming high-quality local media
            {:else}
                <div class="mini-spinner"></div> Buffering video stream...
            {/if}
        </div>
    </div>
{:else}
    <div class="placeholder animate-fade-in">
        <span class="placeholder-icon">🎬</span>
        <span>Select a video file to begin</span>
    </div>
{/if}

<style>
    .video-wrapper {
        width: 100%;
        border-radius: 12px;
        overflow: hidden;
        border: 1px solid #2a2a2a;
        background: #000000;
        box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7);
    }

    .video-container {
        position: relative;
        width: 100%;
        max-height: 80vh;
        background: #000000;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: visible; /* Let popover float nicely */
    }
    
    .video {
        width: 100%;
        height: 100%;
        object-fit: contain;
        display: block;
    }

    .subtitle-overlay {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        pointer-events: none; /* Let video play/pause controls be clickable */
        z-index: 10;
        box-sizing: border-box;
    }

    .subtitle-block {
        position: absolute;
        pointer-events: auto; /* Subtitle itself is interactive! */
        cursor: move; /* Reposition grab cursor */
        color: #ffffff;
        padding: 10px 24px;
        border-radius: 8px;
        text-align: center;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        user-select: none;
        min-height: 50px;
    }

    .subtitle-block:hover {
        border-color: #00bcd4;
        box-shadow: 0 8px 36px rgba(0, 188, 212, 0.3);
    }

    .subtitle-text-render {
        width: 100%;
        word-wrap: break-word;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        font-weight: 600;
        line-height: 1.4;
        position: relative;
    }

    .edit-hint {
        display: block;
        font-size: 10px;
        color: #00bcd4;
        margin-top: 4px;
        opacity: 0;
        transition: opacity 0.2s ease;
        font-weight: 400;
    }

    .subtitle-block:hover .edit-hint {
        opacity: 0.85;
    }

    .subtitle-textarea {
        width: 100%;
        background: transparent;
        border: none;
        color: #ffffff;
        text-align: center;
        font-family: inherit;
        font-weight: 600;
        line-height: 1.4;
        resize: none;
        outline: none;
        padding: 0;
        margin: 0;
        box-sizing: border-box;
    }

    .resize-handle {
        position: absolute;
        bottom: 0;
        right: 0;
        width: 14px;
        height: 14px;
        cursor: se-resize; /* Resizing diagonal cursor */
        background: radial-gradient(circle, #00bcd4 1.5px, transparent 2px);
        background-size: 4px 4px;
        opacity: 0.4;
        transition: opacity 0.2s;
    }

    .subtitle-block:hover .resize-handle {
        opacity: 1;
    }

    /* Insert new subtitle floating CTA button */
    .add-subtitle-btn {
        position: absolute;
        bottom: 12%;
        left: 50%;
        transform: translateX(-50%);
        pointer-events: auto;
        background: rgba(0, 188, 212, 0.85);
        backdrop-filter: blur(4px);
        border: 1px solid #00bcd4;
        color: white;
        padding: 8px 18px;
        border-radius: 20px;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 6px;
        box-shadow: 0 4px 16px rgba(0, 188, 212, 0.4);
        transition: background 0.15s, transform 0.15s, box-shadow 0.15s;
    }

    .add-subtitle-btn:hover {
        background: #00bcd4;
        transform: translateX(-50%) translateY(-1px);
        box-shadow: 0 6px 20px rgba(0, 188, 212, 0.5);
    }

    .add-subtitle-btn:active {
        transform: translateX(-50%) translateY(0);
    }

    .plus-icon {
        font-size: 14px;
        font-weight: 700;
    }

    /* Popover Controls */
    .size-controls-popover {
        position: absolute;
        bottom: 120%; /* Float right above the subtitle block */
        background: rgba(20, 24, 35, 0.96);
        backdrop-filter: blur(16px);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 12px;
        padding: 16px;
        width: 290px;
        box-shadow: 0 12px 40px rgba(0, 0, 0, 0.7);
        pointer-events: auto; /* interactive */
        display: flex;
        flex-direction: column;
        gap: 14px;
        z-index: 100;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        box-sizing: border-box;
        text-align: left;
    }

    .popover-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        padding-bottom: 8px;
    }

    .popover-title {
        font-size: 13px;
        font-weight: 700;
        color: #e2e8f0;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .close-btn {
        background: transparent;
        border: none;
        color: #94a3b8;
        font-size: 20px;
        cursor: pointer;
        line-height: 1;
        padding: 0 4px;
        transition: color 0.1s;
    }

    .close-btn:hover {
        color: #ffffff;
    }

    .popover-section {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .section-label {
        font-size: 12px;
        color: #94a3b8;
        font-weight: 500;
    }

    .val-highlight {
        color: #00bcd4;
        font-weight: 700;
    }

    .slider-row {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .adjust-btn {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
        border-radius: 6px;
        width: 32px;
        height: 28px;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background 0.15s, border-color 0.15s, color 0.15s;
    }

    .adjust-btn:hover {
        background: rgba(255, 255, 255, 0.15);
        border-color: #00bcd4;
        color: #00bcd4;
    }

    .size-slider {
        flex: 1;
        accent-color: #00bcd4;
        cursor: pointer;
        height: 6px;
        border-radius: 3px;
        background: rgba(255, 255, 255, 0.15);
        outline: none;
    }

    .edit-textarea {
        background: rgba(0, 0, 0, 0.3);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 6px;
        color: white;
        padding: 8px;
        font-size: 13px;
        font-family: inherit;
        resize: none;
        outline: none;
        transition: border-color 0.2s, box-shadow 0.2s;
        width: 100%;
        box-sizing: border-box;
    }

    .edit-textarea:focus {
        border-color: #00bcd4;
        box-shadow: 0 0 0 2px rgba(0, 188, 212, 0.2);
    }
    
    .placeholder {
        width: 100%;
        aspect-ratio: 16/9;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 1.25rem;
        color: #888888;
        font-size: 14px;
        font-weight: 500;
        border: 1px dashed #333333;
        border-radius: 12px;
        background: #111111;
        transition: border-color 0.25s, background-color 0.25s;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }
    
    .placeholder:hover {
        border-color: #555555;
        background: #141414;
    }
    
    .placeholder.error {
        color: #ff6b6b;
        border-color: #551a1a;
        background: #1a0808;
    }
    
    .placeholder-icon {
        font-size: 2.25rem;
        opacity: 0.8;
    }
    
    .error-icon {
        font-size: 2.25rem;
    }

    .loading-text {
        color: #a8a8a8;
        letter-spacing: 0.02em;
    }
    
    .spinner {
        width: 32px;
        height: 32px;
        border: 3px solid #222222;
        border-top-color: #ffffff;
        border-radius: 50%;
        animation: spin 0.8s cubic-bezier(0.5, 0.1, 0.4, 0.9) infinite;
    }

    .mini-spinner {
        width: 12px;
        height: 12px;
        border: 2px solid #222222;
        border-top-color: #888888;
        border-radius: 50%;
        display: inline-block;
        margin-right: 6px;
        vertical-align: middle;
        animation: spin 0.8s linear infinite;
    }
    
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
    
    .status {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.75rem 1rem;
        background: #111111;
        border-top: 1px solid #1f1f1f;
        color: #999999;
        font-size: 0.8rem;
        font-weight: 500;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }

    .success-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: #00e676;
        display: inline-block;
        margin-right: 8px;
        box-shadow: 0 0 10px rgba(0, 230, 118, 0.6);
    }
    
    .animate-fade-in {
        animation: fadeIn 0.5s cubic-bezier(0.16, 1, 0.3, 1) both;
    }

    .animate-scale-in {
        animation: scaleIn 0.2s cubic-bezier(0.34, 1.56, 0.64, 1) both;
    }
    
    @keyframes fadeIn {
        from { 
            opacity: 0; 
            transform: scale(0.98) translateY(4px); 
        }
        to { 
            opacity: 1; 
            transform: scale(1) translateY(0); 
        }
    }

    @keyframes scaleIn {
        from {
            opacity: 0;
            transform: scale(0.92) translateY(8px);
        }
        to {
            opacity: 1;
            transform: scale(1) translateY(0);
        }
    }
</style>
