<script lang="ts">
    import { tick } from "svelte";
    import { subtitleAnimation, selectedSubtitleIndices, subtitle, updateSubtitleProperties, pushUndoSnapshot } from "../store.js";

    let animationType = "none";
    let animationSpeed = 200;

    let lastInspectedIndex = -2;
    let suppressApply = false;

    $: hasSelection = $selectedSubtitleIndices.length > 0;
    $: inspectedIndex = hasSelection ? $selectedSubtitleIndices[0] : -1;

    $: if (inspectedIndex !== lastInspectedIndex) {
        lastInspectedIndex = inspectedIndex;
        suppressApply = true;
        if (inspectedIndex >= 0 && $subtitle[inspectedIndex]) {
            const sub = $subtitle[inspectedIndex];
            animationType = sub.animationType !== undefined ? sub.animationType : ($subtitleAnimation.animationType || "none");
            animationSpeed = sub.animationSpeed !== undefined ? sub.animationSpeed : ($subtitleAnimation.animationSpeed ?? 200);
        } else {
            animationType = $subtitleAnimation.animationType || "none";
            animationSpeed = $subtitleAnimation.animationSpeed ?? 200;
        }
        tick().then(() => { suppressApply = false; });
    }

    $: {
        if (!suppressApply && hasSelection) {
            updateSubtitleProperties($selectedSubtitleIndices, {
                animationType,
                animationSpeed
            }, { recordUndo: false });
        }
    }
</script>

<div class="animation-selector-panel animate-fade-in">
    <div class="header-row">
        <h3 class="section-title">
            <span class="movie-icon">🎬</span> Subtitle Animation
        </h3>
    </div>

    {#if !hasSelection}
        <p class="selection-hint">Select a subtitle on the timeline or video to edit animations.</p>
    {/if}

    <div class="controls-body" class:disabled={!hasSelection}>
    <!-- Animation Type Select -->
    <div class="control-card">
        <div class="card-header">
            <label for="animationTypeSelect" class="card-label">Active Animation</label>
        </div>
        <div class="select-container">
            <select
                id="animationTypeSelect"
                bind:value={animationType}
                class="premium-select"
                onmousedown={() => pushUndoSnapshot(true)}
            >
                <option value="none">None (Static)</option>
                <option value="pop-up">Pop Up</option>
                <option value="bottom-to-top">Bottom To Top</option>
                <option value="scale-in">Scale In</option>
                <option value="glitch">Glitch Text</option>
                <option value="split-text">Split Text</option>
                <option value="typing">Typing</option>
                <option value="decrypt">Decrypt Text</option>
            </select>
        </div>
    </div>

    <!-- Conditional Animation Controls -->
    {#if animationType !== "none"}
        <div class="control-card">
            <div class="card-header">
                <label for="animationSpeedInput" class="card-label">Animation Speed</label>
                <span class="value-badge">{animationSpeed}ms</span>
            </div>
            <div class="slider-container">
                <input
                    id="animationSpeedInput"
                    type="range"
                    min="0"
                    max="1000"
                    step="10"
                    bind:value={animationSpeed}
                    class="premium-slider"
                    onmousedown={() => pushUndoSnapshot(true)}
                />
                <input
                    type="number"
                    min="0"
                    max="1000"
                    bind:value={animationSpeed}
                    class="speed-number-input"
                    onfocus={() => pushUndoSnapshot(true)}
                />
            </div>
        </div>
    {/if}
    </div>
</div>

<style>
    .animation-selector-panel {
        background: #1e1e24;
        border: 1px solid #2d2d38;
        border-radius: 14px;
        padding: 20px;
        color: #e2e8f0;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
        display: flex;
        flex-direction: column;
        gap: 16px;
        max-width: 440px;
        box-sizing: border-box;
        margin-bottom: 16px;
    }

    .header-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid #2d2d38;
        padding-bottom: 12px;
    }

    .section-title {
        font-size: 15px;
        font-weight: 700;
        margin: 0;
        letter-spacing: 0.03em;
        color: #00bcd4;
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .movie-icon {
        font-size: 16px;
    }

    .control-card {
        background: #25252f;
        border: 1px solid #323242;
        border-radius: 10px;
        padding: 12px;
        display: flex;
        flex-direction: column;
        gap: 10px;
        transition: border-color 0.2s, box-shadow 0.2s;
    }

    .control-card:hover {
        border-color: #3f3f54;
    }

    .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .card-label {
        font-size: 12px;
        font-weight: 600;
        color: #94a3b8;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .value-badge {
        font-size: 11px;
        font-weight: 700;
        background: #00bcd4;
        color: #111;
        padding: 2px 8px;
        border-radius: 12px;
    }

    .select-container {
        position: relative;
        width: 100%;
    }

    .premium-select {
        width: 100%;
        background: #181820;
        border: 1px solid #323242;
        border-radius: 8px;
        color: #fff;
        padding: 10px 12px;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        outline: none;
        transition: border-color 0.2s, box-shadow 0.2s;
        appearance: none;
    }

    .premium-select:focus {
        border-color: #00bcd4;
        box-shadow: 0 0 0 2px rgba(0, 188, 212, 0.2);
    }

    .select-container::after {
        content: "▼";
        font-size: 10px;
        color: #94a3b8;
        position: absolute;
        right: 12px;
        top: 50%;
        transform: translateY(-50%);
        pointer-events: none;
    }

    .slider-container {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .premium-slider {
        flex: 1;
        accent-color: #00bcd4;
        cursor: pointer;
        height: 6px;
        border-radius: 3px;
        background: #3e3e4f;
        outline: none;
    }

    .speed-number-input {
        width: 64px;
        background: #181820;
        border: 1px solid #323242;
        border-radius: 6px;
        color: #fff;
        font-size: 13px;
        font-weight: 600;
        text-align: center;
        padding: 4px;
        outline: none;
        transition: border-color 0.2s;
    }

    .speed-number-input:focus {
        border-color: #00bcd4;
    }

    .animate-fade-in {
        animation: fadeIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(4px); }
        to { opacity: 1; transform: translateY(0); }
    }

    .selection-hint {
        margin: 0;
        padding: 10px 12px;
        font-size: 12px;
        color: #94a3b8;
        background: rgba(0, 188, 212, 0.08);
        border: 1px solid rgba(0, 188, 212, 0.2);
        border-radius: 8px;
        line-height: 1.4;
    }

    .controls-body.disabled {
        opacity: 0.45;
        pointer-events: none;
        user-select: none;
    }
</style>
