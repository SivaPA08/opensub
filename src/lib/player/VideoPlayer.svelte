<script lang="ts">
    import { onDestroy, onMount } from "svelte";
    import {
        videoPath,
        videoDuration,
        videoCurrentTime,
        subtitle,
        subtitleFontSize,
        subtitleAnimation,
        selectedSubtitleIndices,
        updateSubtitleProperties,
        pushUndoSnapshot,
        updateSubtitles,
        videoPreviewMetrics,
        type Subtitle,
    } from "../store.js";
    import {
        toVideoFontSize,
    } from "../subtitleScale.js";
    import { invoke } from "@tauri-apps/api/core";
    import { save } from "@tauri-apps/plugin-dialog";
    import { revealItemInDir } from "@tauri-apps/plugin-opener";
    import { listen } from "@tauri-apps/api/event";
    import GlitchText from "../animations/GlitchText.svelte";
    import SplitText from "../animations/SplitText.svelte";
    import TypingText from "../animations/TypingText.svelte";
    import DecriptText from "../animations/DecriptText.svelte";
    import PopUp from "../animations/PopUp.svelte";
    import BottomToTop from "../animations/BottomToTop.svelte";
    import Wave from "../animations/Wave.svelte";

    let videoElement: HTMLVideoElement;
    let videoSrc = "";

    // Rendering State
    let showRenderOverlay = false;
    let isRendering = false;
    let renderProgress = 0;
    let renderError = "";
    let renderSuccess = false;
    let outputPath = "";

    let unlistenRender: (() => void) | null = null;

    onMount(async () => {
        unlistenRender = await listen<number>("render-progress", (event) => {
            renderProgress = Math.round(event.payload);
        });

        if (typeof ResizeObserver !== "undefined") {
            containerResizeObserver = new ResizeObserver(() => updatePreviewMetrics());
        }
    });

    onDestroy(() => {
        containerResizeObserver?.disconnect();
        if (unlistenRender) unlistenRender();
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
    $: containerAspectRatio =
        videoWidth && videoHeight ? `${videoWidth} / ${videoHeight}` : "16 / 9";

    $: activeSubtitles = $subtitle.filter(
        (sub) => $videoCurrentTime >= sub.start && $videoCurrentTime <= sub.end,
    );

    $: activeSubtitle = (() => {
        for (const sub of activeSubtitles) {
            const idx = $subtitle.indexOf(sub);
            if ($selectedSubtitleIndices.includes(idx)) {
                return sub;
            }
        }
        return activeSubtitles[0];
    })();

    let showSizeControls = false;
    let isEditingText = false;

    // Subtitle Custom Position & Size States (percentage values)
    let subX = 50; // Horizontal center percentage
    let subY = 85; // Vertical offset percentage
    let subWidth = 70; // Width percentage of the video wrapper

    let lastActiveSub: Subtitle | undefined = undefined;

    $: if (activeSubtitle && activeSubtitle !== lastActiveSub) {
        lastActiveSub = activeSubtitle;
        subX = activeSubtitle.subX !== undefined ? activeSubtitle.subX : 50;
        subY = activeSubtitle.subY !== undefined ? activeSubtitle.subY : 85;
        subWidth = activeSubtitle.subWidth !== undefined ? activeSubtitle.subWidth : 70;
    }

    let draggingSubtitle: Subtitle | null = null;
    let resizingSubtitle: Subtitle | null = null;

    function getSubStyle(sub: Subtitle) {
        return {
            x: sub.subX !== undefined ? sub.subX : 50,
            y: sub.subY !== undefined ? sub.subY : 85,
            width: sub.subWidth !== undefined ? sub.subWidth : 70,
            fontSize: sub.fontSize !== undefined ? sub.fontSize : $subtitleAnimation.fontSize,
            fontColor: sub.fontColor !== undefined ? sub.fontColor : $subtitleAnimation.fontColor,
            backgroundColor: sub.backgroundColor !== undefined ? sub.backgroundColor : $subtitleAnimation.backgroundColor,
            customFont: sub.customFont !== undefined ? sub.customFont : $subtitleAnimation.customFont,
            fontOpacity: sub.fontOpacity !== undefined ? sub.fontOpacity : ($subtitleAnimation.fontOpacity ?? 1.0),
            backgroundOpacity: sub.backgroundOpacity !== undefined ? sub.backgroundOpacity : ($subtitleAnimation.backgroundOpacity ?? 0.85),
            animationType: sub.animationType !== undefined ? sub.animationType : ($subtitleAnimation.animationType ?? 'none'),
            animationSpeed: sub.animationSpeed !== undefined ? sub.animationSpeed : ($subtitleAnimation.animationSpeed ?? 200),
        };
    }

    // Resolved styles for active subtitle falling back to global settings
    $: activeFontSize = (activeSubtitle && activeSubtitle.fontSize !== undefined) ? activeSubtitle.fontSize : $subtitleAnimation.fontSize;
    $: activeFontColor = (activeSubtitle && activeSubtitle.fontColor !== undefined) ? activeSubtitle.fontColor : $subtitleAnimation.fontColor;
    $: activeBackgroundColor = (activeSubtitle && activeSubtitle.backgroundColor !== undefined) ? activeSubtitle.backgroundColor : $subtitleAnimation.backgroundColor;
    $: activeCustomFont = (activeSubtitle && activeSubtitle.customFont !== undefined) ? activeSubtitle.customFont : $subtitleAnimation.customFont;
    $: activeFontOpacity = (activeSubtitle && activeSubtitle.fontOpacity !== undefined) ? activeSubtitle.fontOpacity : ($subtitleAnimation.fontOpacity ?? 1.0);
    $: activeBackgroundOpacity = (activeSubtitle && activeSubtitle.backgroundOpacity !== undefined) ? activeSubtitle.backgroundOpacity : ($subtitleAnimation.backgroundOpacity ?? 0.85);
    $: activeAnimationType = (activeSubtitle && activeSubtitle.animationType !== undefined) ? activeSubtitle.animationType : ($subtitleAnimation.animationType ?? 'none');
    $: activeAnimationSpeed = (activeSubtitle && activeSubtitle.animationSpeed !== undefined) ? activeSubtitle.animationSpeed : ($subtitleAnimation.animationSpeed ?? 200);

    function saveSubtitlePosition(targetSub: Subtitle, x?: number, y?: number, width?: number) {
        const targetIndex = $subtitle.findIndex(
            (s) => s.start === targetSub.start && s.end === targetSub.end && s.content === targetSub.content
        );
        if (targetIndex === -1) return;

        const isSelected = $selectedSubtitleIndices.includes(targetIndex);
        const indicesToUpdate = isSelected ? $selectedSubtitleIndices : [targetIndex];

        const updates: Partial<Subtitle> = {};
        if (x !== undefined) {
            updates.subX = x;
            if (targetSub === activeSubtitle) subX = x;
        }
        if (y !== undefined) {
            updates.subY = y;
            if (targetSub === activeSubtitle) subY = y;
        }
        if (width !== undefined) {
            updates.subWidth = width;
            if (targetSub === activeSubtitle) subWidth = width;
        }

        updateSubtitleProperties(indicesToUpdate, updates, { recordUndo: false });
    }

    let isDraggingSubtitle = false;
    let dragStartX = 0;
    let dragStartY = 0;
    let initialSubX = 50;
    let initialSubY = 85;

    let isResizingSubtitle = false;
    let resizeStartX = 0;
    let initialSubWidth = 70;

    let containerRef: HTMLDivElement;
    let containerResizeObserver: ResizeObserver | undefined;

    function updatePreviewMetrics() {
        if (!containerRef || videoWidth <= 0 || videoHeight <= 0) return;
        const rect = containerRef.getBoundingClientRect();
        videoPreviewMetrics.set({
            videoWidth,
            videoHeight,
            containerWidth: rect.width,
            containerHeight: rect.height,
        });
    }

    $: if (containerRef && videoWidth > 0 && videoHeight > 0) {
        updatePreviewMetrics();
    }

    $: if (containerRef && containerResizeObserver) {
        containerResizeObserver.disconnect();
        containerResizeObserver.observe(containerRef);
    }

    // Autofocus action for inline text editor
    function autofocus(node: HTMLTextAreaElement) {
        node.focus();
        node.select();
    }

    // Select the active subtitle in the global selection store
    function selectSubtitle(sub: Subtitle) {
        if (!sub) return;
        const targetIndex = $subtitle.findIndex(
            (s) => s.start === sub.start && s.end === sub.end && s.content === sub.content
        );
        if (targetIndex !== -1 && !$selectedSubtitleIndices.includes(targetIndex)) {
            selectedSubtitleIndices.set([targetIndex]);
        }
    }

    // Drag-to-Reposition Logic
    function startSubtitleDrag(e: MouseEvent, sub: Subtitle) {
        const target = e.target as HTMLElement;
        if (target.classList.contains("resize-handle") || isEditingText) return;

        e.preventDefault();
        e.stopPropagation();
        selectSubtitle(sub);
        pushUndoSnapshot(true);
        draggingSubtitle = sub;
        isDraggingSubtitle = true;
        dragStartX = e.clientX;
        dragStartY = e.clientY;
        initialSubX = sub.subX !== undefined ? sub.subX : 50;
        initialSubY = sub.subY !== undefined ? sub.subY : 85;

        window.addEventListener("mousemove", handleSubtitleDrag);
        window.addEventListener("mouseup", stopSubtitleDrag);
    }

    function handleSubtitleDrag(e: MouseEvent) {
        if (!isDraggingSubtitle || !containerRef || !draggingSubtitle) return;

        const rect = containerRef.getBoundingClientRect();
        const dx = e.clientX - dragStartX;
        const dy = e.clientY - dragStartY;

        const pctDx = (dx / rect.width) * 100;
        const pctDy = (dy / rect.height) * 100;

        const newSubX = Math.max(5, Math.min(95, initialSubX + pctDx));
        const newSubY = Math.max(5, Math.min(95, initialSubY + pctDy));

        saveSubtitlePosition(draggingSubtitle, newSubX, newSubY, undefined);
    }

    function stopSubtitleDrag() {
        isDraggingSubtitle = false;
        draggingSubtitle = null;
        window.removeEventListener("mousemove", handleSubtitleDrag);
        window.removeEventListener("mouseup", stopSubtitleDrag);
    }

    // Drag-to-Resize Logic
    function startSubtitleResize(e: MouseEvent, sub: Subtitle) {
        e.preventDefault();
        e.stopPropagation();
        selectSubtitle(sub);
        pushUndoSnapshot(true);
        resizingSubtitle = sub;
        isResizingSubtitle = true;
        resizeStartX = e.clientX;
        initialSubWidth = sub.subWidth !== undefined ? sub.subWidth : 70;

        window.addEventListener("mousemove", handleSubtitleResize);
        window.addEventListener("mouseup", stopSubtitleResize);
    }

    function handleSubtitleResize(e: MouseEvent) {
        if (!isResizingSubtitle || !containerRef || !resizingSubtitle) return;

        const rect = containerRef.getBoundingClientRect();
        const dx = e.clientX - resizeStartX;
        const pctDx = (dx / rect.width) * 100 * 2; // Resize from center

        const newSubWidth = Math.max(20, Math.min(95, initialSubWidth + pctDx));

        saveSubtitlePosition(resizingSubtitle, undefined, undefined, newSubWidth);
    }

    function stopSubtitleResize() {
        isResizingSubtitle = false;
        resizingSubtitle = null;
        window.removeEventListener("mousemove", handleSubtitleResize);
        window.removeEventListener("mouseup", stopSubtitleResize);
    }


    function handleSubtitleClick(e: MouseEvent, sub: Subtitle) {
        if (activeSubtitle === sub) {
            showSizeControls = !showSizeControls;
        } else {
            selectSubtitle(sub);
            showSizeControls = true;
        }
    }

    function changeSize(delta: number) {
        const newSize = Math.max(12, Math.min(80, activeFontSize + delta));
        updateFontSize(newSize);
    }

    function updateFontSize(newSize: number) {
        subtitleAnimation.update((style) => ({
            ...style,
            fontSize: newSize,
        }));

        if (activeSubtitle) {
            const targetIndex = $subtitle.findIndex(
                (s) => s.start === activeSubtitle.start && s.end === activeSubtitle.end && s.content === activeSubtitle.content
            );
            if (targetIndex !== -1) {
                const isSelected = $selectedSubtitleIndices.includes(targetIndex);
                const indicesToUpdate = isSelected ? $selectedSubtitleIndices : [targetIndex];
                updateSubtitleProperties(indicesToUpdate, { fontSize: newSize });
            }
        }
    }

    let textEditUndoRecorded = false;

    function updateSubtitleText(sub: Subtitle, newText: string) {
        if (!textEditUndoRecorded) {
            pushUndoSnapshot(true);
            textEditUndoRecorded = true;
        }
        updateSubtitles((items) => {
            return items.map((item) => {
                if (item.start === sub.start && item.end === sub.end) {
                    return { ...item, content: newText };
                }
                return item;
            });
        }, { recordUndo: false });
    }

    function hexOrRgbToRgba(color: string, opacity: number): string {
        if (!color) return `rgba(0,0,0,${opacity})`;

        // If it's already rgba, replace the alpha
        if (color.startsWith("rgba")) {
            return color.replace(/[\d\.]+\)$/, `${opacity})`);
        }

        // If it's rgb, convert to rgba
        if (color.startsWith("rgb")) {
            return color.replace("rgb", "rgba").replace(")", `, ${opacity})`);
        }

        // If it's hex (#fff or #ffffff)
        if (color.startsWith("#")) {
            let hex = color.slice(1);
            if (hex.length === 3) {
                hex = hex
                    .split("")
                    .map((c) => c + c)
                    .join("");
            }
            const r = parseInt(hex.slice(0, 2), 16) || 0;
            const g = parseInt(hex.slice(2, 4), 16) || 0;
            const b = parseInt(hex.slice(4, 6), 16) || 0;
            return `rgba(${r}, ${g}, ${b}, ${opacity})`;
        }

        return color; // Fallback
    }

    async function chooseSaveLocation() {
        try {
            const inputName = $videoPath
                ? $videoPath.split("/").pop()
                : "output.mp4";
            const defaultName = inputName
                ? inputName.replace(/\.[^/.]+$/, "") + "_subbed.mp4"
                : "output.mp4";

            const path = await save({
                title: "Save Rendered Video",
                defaultPath: defaultName,
                filters: [
                    {
                        name: "Video",
                        extensions: ["mp4"],
                    },
                ],
            });
            if (path && typeof path === "string") {
                outputPath = path;
            }
        } catch (err) {
            console.error("Error choosing save location:", err);
            renderError = "Failed to choose save location: " + err;
        }
    }

    async function openOutputFolder() {
        if (!outputPath) return;
        try {
            await revealItemInDir(outputPath);
        } catch (err) {
            console.error("Error opening output folder:", err);
        }
    }

    async function startRendering() {
        if (!outputPath) return;

        isRendering = true;
        renderProgress = 0;
        renderError = "";
        renderSuccess = false;

        try {
            updatePreviewMetrics();
            const metrics = $videoPreviewMetrics;
            const containerRect = containerRef?.getBoundingClientRect();
            const containerWidth = containerRect?.width ?? metrics.containerWidth;
            const containerHeight = containerRect?.height ?? metrics.containerHeight;

            const resolveVideoFontSize = (previewFontSize: number | undefined) => {
                if (previewFontSize === undefined) return undefined;
                return toVideoFontSize(previewFontSize, {
                    videoWidth,
                    videoHeight,
                    containerWidth,
                    containerHeight,
                });
            };

            const config = {
                input_path: $videoPath,
                output_path: outputPath,
                subtitles: $subtitle.map((s) => ({
                    start: s.start,
                    end: s.end,
                    content: s.content,
                    subX: s.subX,
                    subY: s.subY,
                    subWidth: s.subWidth,
                    fontSize: resolveVideoFontSize(s.fontSize),
                    fontColor: s.fontColor,
                    backgroundColor: s.backgroundColor,
                    customFont: s.customFont,
                    customFontFile: s.customFontFile,
                    fontOpacity: s.fontOpacity,
                    backgroundOpacity: s.backgroundOpacity,
                    animationType: s.animationType,
                    animationSpeed: s.animationSpeed,
                })),
                style: {
                    fontSize: toVideoFontSize($subtitleAnimation.fontSize, {
                        videoWidth,
                        videoHeight,
                        containerWidth,
                        containerHeight,
                    }),
                    fontColor: $subtitleAnimation.fontColor,
                    backgroundColor: $subtitleAnimation.backgroundColor,
                    customFont: $subtitleAnimation.customFont,
                    customFontFile: $subtitleAnimation.customFontFile,
                    fontOpacity: $subtitleAnimation.fontOpacity,
                    backgroundOpacity: $subtitleAnimation.backgroundOpacity,
                    animationType: $subtitleAnimation.animationType ?? 'none',
                    animationSpeed: $subtitleAnimation.animationSpeed ?? 200,
                },
                position: {
                    subX: subX,
                    subY: subY,
                    subWidth: subWidth,
                },
                video_info: {
                    width: videoWidth,
                    height: videoHeight,
                    container_width: containerWidth,
                    container_height: containerHeight,
                },
            };

            const configJson = JSON.stringify(config);

            const result = await invoke<{ status: string; message: string }>(
                "run_render",
                {
                    configJson: configJson,
                },
            );

            if (result.status === "ok") {
                renderSuccess = true;
            } else {
                renderError = result.message;
            }
        } catch (err) {
            console.error("Render failed:", err);
            renderError = String(err);
        } finally {
            isRendering = false;
        }
    }

    function closeRenderOverlay() {
        showRenderOverlay = false;
        renderSuccess = false;
        renderError = "";
    }

    function handleVideoEnded() {
        showRenderOverlay = true;
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
                errorMessage =
                    "Failed to initialize local HTTP streaming server.";
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
        <div
            class="video-container"
            bind:this={containerRef}
            style="aspect-ratio: {containerAspectRatio};"
        >
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
                bind:videoWidth
                bind:videoHeight
                on:ended={handleVideoEnded}
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
                    const mediaError = (event.currentTarget as HTMLVideoElement)
                        .error;
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
                {#if activeSubtitles.length > 0}
                    {#each activeSubtitles as sub (sub.start + '_' + sub.end)}
                        {@const style = getSubStyle(sub)}
                        <!-- Floating Popover Controls -->
                        {#if showSizeControls && activeSubtitle === sub}
                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <div
                                class="size-controls-popover animate-scale-in"
                                on:mousedown|stopPropagation
                            >
                                <div class="popover-header">
                                    <span class="popover-title"
                                        >Subtitle Properties</span
                                    >
                                    <button
                                        class="close-btn"
                                        on:click={() => (showSizeControls = false)}
                                        >×</button
                                    >
                                </div>

                                <div class="popover-section">
                                    <span class="section-label"
                                        >Font Size: <span class="val-highlight"
                                            >{style.fontSize}px</span
                                        ></span
                                    >
                                    <div class="slider-row">
                                        <button
                                            class="adjust-btn"
                                            on:click={() => changeSize(-2)}
                                            >A-</button
                                        >
                                        <input
                                            type="range"
                                            min="12"
                                            max="80"
                                            value={style.fontSize}
                                            on:input={(e) => updateFontSize(parseInt(e.currentTarget.value))}
                                            class="size-slider"
                                        />
                                        <button
                                            class="adjust-btn"
                                            on:click={() => changeSize(2)}
                                            >A+</button
                                        >
                                    </div>
                                </div>

                                <div class="popover-section">
                                    <span class="section-label"
                                        >Edit text content:</span
                                    >
                                    <textarea
                                        value={sub.content}
                                        on:input={(e) =>
                                            updateSubtitleText(
                                                sub,
                                                e.currentTarget.value,
                                            )}
                                        class="edit-textarea"
                                        rows="2"
                                    ></textarea>
                                </div>
                            </div>
                        {/if}

                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <!-- svelte-ignore a11y_no_static_element_interactions -->
                        <div
                            class="subtitle-block {style.animationType === 'scale-in' ? 'animate-scale-in' : ''}"
                            class:selected={activeSubtitle === sub}
                            style="
                                left: {style.x}%; 
                                top: {style.y}%; 
                                width: {style.width}%; 
                                transform: translate(-50%, -50%);
                                --anim-speed: {style.animationSpeed}ms;
                                background: {hexOrRgbToRgba(
                                    style.backgroundColor,
                                    style.backgroundOpacity,
                                )};
                                font-family: {style.customFont};
                                font-size: {style.fontSize}px;
                                color: {hexOrRgbToRgba(
                                    style.fontColor,
                                    style.fontOpacity,
                                )};
                                -webkit-backdrop-filter: blur({8 *
                                    style.backgroundOpacity}px);
                                backdrop-filter: blur({8 *
                                    style.backgroundOpacity}px);
                                border: 1px solid rgba(255, 255, 255, {0.1 *
                                    style.backgroundOpacity});
                                box-shadow: 0 8px 32px rgba(0, 0, 0, {0.6 *
                                    style.backgroundOpacity}), inset 0 0 0 1px rgba(255, 255, 255, {0.15 *
                                    style.backgroundOpacity});
                            "
                            on:mousedown={(e) => startSubtitleDrag(e, sub)}
                            on:click|stopPropagation={(e) => handleSubtitleClick(e, sub)}
                        >
                            {#if isEditingText && activeSubtitle === sub}
                                <textarea
                                    class="subtitle-textarea"
                                    style="font-size: {style.fontSize}px; color: {hexOrRgbToRgba(
                                        style.fontColor,
                                        style.fontOpacity,
                                    )}; font-family: {style.customFont};"
                                    value={sub.content}
                                    on:input={(e) =>
                                        updateSubtitleText(
                                            sub,
                                            e.currentTarget.value,
                                        )}
                                    on:blur={() => {
                                        isEditingText = false;
                                        textEditUndoRecorded = false;
                                    }}
                                    on:click|stopPropagation
                                    on:keydown={(e) => {
                                        if (e.key === "Enter" && !e.shiftKey) {
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
                                    style="font-size: {style.fontSize}px; color: {hexOrRgbToRgba(
                                        style.fontColor,
                                        style.fontOpacity,
                                    )}; font-family: {style.customFont};"
                                    on:dblclick={() => {
                                        textEditUndoRecorded = false;
                                        isEditingText = true;
                                        selectSubtitle(sub);
                                    }}
                                >
                                    {#if style.animationType === 'glitch'}
                                        <GlitchText
                                            text={sub.content}
                                            speed={style.animationSpeed}
                                        />
                                    {:else if style.animationType === 'split-text'}
                                        <SplitText
                                            text={sub.content}
                                            duration={style.animationSpeed / 1000}
                                            delay={(style.animationSpeed / 10) || 10}
                                        />
                                    {:else if style.animationType === 'typing'}
                                        <TypingText
                                            text={sub.content}
                                            typingSpeed={style.animationSpeed}
                                            loop={false}
                                            showCursor={true}
                                        />
                                    {:else if style.animationType === 'decrypt'}
                                        <DecriptText
                                            text={sub.content}
                                            speed={style.animationSpeed}
                                            animateOn="view"
                                            sequential={true}
                                        />
                                    {:else if style.animationType === 'pop-up'}
                                        <PopUp
                                            text={sub.content}
                                            speed={style.animationSpeed}
                                        />
                                    {:else if style.animationType === 'bottom-to-top'}
                                        <BottomToTop
                                            text={sub.content}
                                            speed={style.animationSpeed}
                                        />
                                    {:else if style.animationType === 'wave'}
                                        <Wave
                                            text={sub.content}
                                            speed={style.animationSpeed}
                                        />
                                    {:else}
                                        {sub.content}
                                    {/if}
                                    <span class="edit-hint"
                                        >Double-click to edit text</span
                                    >
                                </div>
                            {/if}

                            <!-- Resize Handle in Bottom-Right Corner -->
                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <div
                                class="resize-handle"
                                on:mousedown={(e) => startSubtitleResize(e, sub)}
                            ></div>
                        </div>
                    {/each}
                {/if}
            </div>

            <!-- Render Overlay -->
            {#if showRenderOverlay}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                    class="render-overlay animate-fade-in"
                    on:mousedown|stopPropagation
                >
                    <div class="render-card">
                        <button
                            class="close-overlay-btn"
                            on:click={closeRenderOverlay}>×</button
                        >

                        {#if isRendering}
                            <h3 class="render-title">Rendering Video</h3>
                            <p class="render-subtitle">
                                Burning custom styled subtitles into your video
                                file...
                            </p>

                            <div class="progress-container">
                                <div class="progress-bar-outer">
                                    <div
                                        class="progress-bar-inner"
                                        style="width: {renderProgress}%"
                                    ></div>
                                </div>
                                <span class="progress-text"
                                    >{renderProgress}%</span
                                >
                            </div>
                        {:else if renderSuccess}
                            <div class="success-badge">✓</div>
                            <h3 class="render-title">Render Complete!</h3>
                            <p class="render-subtitle">
                                Your video has been saved successfully with
                                burned-in subtitles.
                            </p>

                            <div class="success-path">{outputPath}</div>

                            <div class="actions-row">
                                <button
                                    class="action-btn primary"
                                    on:click={openOutputFolder}
                                    >Open Folder</button
                                >
                                <button
                                    class="action-btn secondary"
                                    on:click={closeRenderOverlay}
                                    >Dismiss</button
                                >
                            </div>
                        {:else}
                            <h3 class="render-title">Export & Render Video</h3>
                            <p class="render-subtitle">
                                Save your video with all the custom subtitle
                                adjustments and styles.
                            </p>

                            {#if renderError}
                                <div class="render-error-box">
                                    <strong>Error:</strong>
                                    {renderError}
                                </div>
                            {/if}

                            <div class="location-picker">
                                <span class="picker-label"
                                    >Destination File:</span
                                >
                                <div class="picker-row">
                                    <span
                                        class="chosen-path"
                                        class:placeholder={!outputPath}
                                    >
                                        {outputPath
                                            ? outputPath
                                            : "No output location selected"}
                                    </span>
                                    <button
                                        class="picker-btn"
                                        on:click={chooseSaveLocation}
                                    >
                                        Browse...
                                    </button>
                                </div>
                            </div>

                            <div class="actions-row">
                                <button
                                    class="action-btn primary"
                                    disabled={!outputPath}
                                    on:click={startRendering}
                                >
                                    Start Render
                                </button>
                                <button
                                    class="action-btn secondary"
                                    on:click={closeRenderOverlay}>Cancel</button
                                >
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}
        </div>
        <div class="status">
            <div class="status-left">
                {#if loaded}
                    <span class="success-dot"></span> Streaming high-quality local
                    media
                {:else}
                    <div class="mini-spinner"></div>
                     Buffering video stream...
                {/if}
            </div>
            <button
                class="export-btn"
                on:click={() => (showRenderOverlay = true)}
            >
                Export Video
            </button>
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
        max-width: 100%;
        max-height: 75vh;
        background: #000000;
        display: block;
        margin: 0 auto;
        overflow: visible; /* Let popover float nicely */
    }

    .video {
        width: 100%;
        height: 100%;
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

    .subtitle-block.selected {
        border: 1px solid #00bcd4 !important;
        box-shadow: 0 0 0 2px rgba(0, 188, 212, 0.4), 0 8px 36px rgba(0, 188, 212, 0.2) !important;
    }

    .subtitle-text-render {
        width: 100%;
        word-wrap: break-word;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
            sans-serif;
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


    /* Popover Controls */
    .size-controls-popover {
        position: absolute;
        bottom: 120%; /* Float right above the subtitle block */
        left: 50%;
        transform: translateX(-50%);
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
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
            sans-serif;
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
        transition:
            background 0.15s,
            border-color 0.15s,
            color 0.15s;
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
        transition:
            border-color 0.2s,
            box-shadow 0.2s;
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
        transition:
            border-color 0.25s,
            background-color 0.25s;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
            sans-serif;
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
        to {
            transform: rotate(360deg);
        }
    }

    .status {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.75rem 1.25rem;
        background: #111111;
        border-top: 1px solid #1f1f1f;
        color: #999999;
        font-size: 0.8rem;
        font-weight: 500;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
            sans-serif;
    }

    .status-left {
        display: flex;
        align-items: center;
    }

    .export-btn {
        background: #00bcd4;
        border: none;
        color: black;
        padding: 6px 14px;
        border-radius: 6px;
        font-size: 0.78rem;
        font-weight: 700;
        cursor: pointer;
        transition:
            background-color 0.2s,
            transform 0.2s;
        box-shadow: 0 4px 12px rgba(0, 188, 212, 0.3);
        font-family: inherit;
    }

    .export-btn:hover {
        background: #00acc1;
        transform: translateY(-1px);
    }

    .export-btn:active {
        transform: translateY(0);
    }

    /* =========================
	   RENDER OVERLAY & CARD
	========================= */
    .render-overlay {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.75);
        backdrop-filter: blur(12px);
        z-index: 1000;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 20px;
        box-sizing: border-box;
    }

    .render-card {
        background: #141822;
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 16px;
        padding: 30px;
        width: 100%;
        max-width: 440px;
        box-shadow:
            0 20px 50px rgba(0, 0, 0, 0.6),
            inset 0 0 0 1px rgba(255, 255, 255, 0.05);
        position: relative;
        text-align: center;
        color: #ffffff;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
            sans-serif;
    }

    .close-overlay-btn {
        position: absolute;
        top: 16px;
        right: 16px;
        background: transparent;
        border: none;
        color: #8f96a8;
        font-size: 24px;
        cursor: pointer;
        line-height: 1;
        transition: color 0.15s ease;
    }

    .close-overlay-btn:hover {
        color: #ffffff;
    }

    .render-title {
        font-size: 1.25rem;
        font-weight: 700;
        margin: 0 0 8px 0;
        letter-spacing: -0.02em;
    }

    .render-subtitle {
        font-size: 0.88rem;
        color: #8f96a8;
        margin: 0 0 24px 0;
        line-height: 1.5;
    }

    .location-picker {
        background: rgba(0, 0, 0, 0.2);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        padding: 14px;
        margin-bottom: 24px;
        text-align: left;
    }

    .picker-label {
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: #8f96a8;
        font-weight: 700;
        display: block;
        margin-bottom: 8px;
    }

    .picker-row {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .chosen-path {
        flex: 1;
        font-size: 0.82rem;
        color: #ffffff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        background: rgba(0, 0, 0, 0.3);
        padding: 8px 12px;
        border-radius: 6px;
        border: 1px solid rgba(255, 255, 255, 0.05);
    }

    .chosen-path.placeholder {
        color: #4a5268;
    }

    .picker-btn {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
        padding: 8px 16px;
        border-radius: 6px;
        font-size: 0.82rem;
        font-weight: 600;
        cursor: pointer;
        transition:
            background 0.15s,
            border-color 0.15s;
    }

    .picker-btn:hover {
        background: rgba(255, 255, 255, 0.15);
        border-color: #00bcd4;
    }

    .actions-row {
        display: flex;
        gap: 12px;
        justify-content: center;
    }

    .action-btn {
        padding: 10px 24px;
        border-radius: 8px;
        font-size: 0.88rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s ease;
        border: none;
    }

    .action-btn.primary {
        background: #00bcd4;
        color: black;
        box-shadow: 0 4px 14px rgba(0, 188, 212, 0.3);
    }

    .action-btn.primary:hover:not(:disabled) {
        background: #00acc1;
        transform: translateY(-1px);
    }

    .action-btn.primary:disabled {
        background: #2a3140;
        color: #4a5268;
        cursor: not-allowed;
        box-shadow: none;
    }

    .action-btn.secondary {
        background: rgba(255, 255, 255, 0.08);
        color: white;
        border: 1px solid rgba(255, 255, 255, 0.05);
    }

    .action-btn.secondary:hover {
        background: rgba(255, 255, 255, 0.15);
    }

    .render-error-box {
        background: rgba(255, 74, 74, 0.1);
        border: 1px solid rgba(255, 74, 74, 0.2);
        color: #ff6b6b;
        padding: 12px;
        border-radius: 8px;
        font-size: 0.82rem;
        margin-bottom: 20px;
        text-align: left;
        line-height: 1.4;
    }

    /* Progress and Success States */
    .progress-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        margin: 20px 0;
    }

    .progress-bar-outer {
        width: 100%;
        height: 6px;
        background: rgba(255, 255, 255, 0.08);
        border-radius: 3px;
        overflow: hidden;
    }

    .progress-bar-inner {
        height: 100%;
        background: #00bcd4;
        border-radius: 3px;
        transition: width 0.3s ease;
        box-shadow: 0 0 10px rgba(0, 188, 212, 0.5);
    }

    .progress-text {
        font-size: 1.25rem;
        font-weight: 700;
        color: #00bcd4;
        font-family: "JetBrains Mono", monospace;
    }

    .success-badge {
        width: 60px;
        height: 60px;
        border-radius: 50%;
        background: rgba(0, 230, 118, 0.1);
        border: 2px solid #00e676;
        color: #00e676;
        font-size: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        margin: 0 auto 20px auto;
        box-shadow: 0 0 20px rgba(0, 230, 118, 0.2);
    }

    .success-path {
        background: rgba(0, 0, 0, 0.2);
        padding: 10px 14px;
        border-radius: 8px;
        font-size: 0.78rem;
        color: #8f96a8;
        font-family: monospace;
        margin-bottom: 24px;
        word-break: break-all;
        border: 1px solid rgba(255, 255, 255, 0.04);
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
        animation: scaleIn var(--anim-speed, 200ms) cubic-bezier(0.34, 1.56, 0.64, 1) both;
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
            transform: translate(-50%, -50%) scale(0.92) translateY(8px);
        }
        to {
            opacity: 1;
            transform: translate(-50%, -50%) scale(1) translateY(0);
        }
    }
</style>
