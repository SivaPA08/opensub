<script lang="ts">
    import { goto } from "$app/navigation";
    import { videoPath, wordPerFrame, subtitle, clearUndoHistory, selectedModel } from "$lib/store";
    import { open } from "@tauri-apps/plugin-dialog";
    import { onMount, onDestroy } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { get } from "svelte/store";

    let selectedFilePath = $state<string | null>(null);
    let localWordPerFrame = $state<number>(3);
    let currentModel = $state<string>(get(selectedModel));

    let downloadProgress = $state<Record<string, number>>({});
    let modelStatuses = $state<Record<string, boolean>>({});
    let isDownloading = $state<Record<string, boolean>>({});
    let unlistenProgress: (() => void) | null = null;

    const models = [
        { id: "tiny", name: "Tiny", size: "~75 MB", speed: "Very Fast", accuracy: "Lowest" },
        { id: "base", name: "Base", size: "~150 MB", speed: "Fast", accuracy: "Good" },
        { id: "small", name: "Small", size: "~488 MB", speed: "Medium", accuracy: "Better" },
        { id: "medium", name: "Medium", size: "~1.5 GB", speed: "Slow", accuracy: "High" },
    ];

    $effect(() => {
        selectedModel.set(currentModel);
    });

    async function checkStatuses() {
        try {
            const status = await invoke<Record<string, boolean>>("check_models_status");
            modelStatuses = status;
        } catch (err) {
            console.error("Failed to check model statuses:", err);
        }
    }

    async function downloadModel(modelId: string) {
        if (isDownloading[modelId] || modelStatuses[modelId]) return;
        isDownloading[modelId] = true;
        downloadProgress[modelId] = 0;
        try {
            await invoke("download_model", { modelName: modelId });
            modelStatuses[modelId] = true;
        } catch (err) {
            alert(`Failed to download model ${modelId}: ${err}`);
        } finally {
            isDownloading[modelId] = false;
        }
    }

    async function handleFileSelect() {
        try {
            const selected = await open({
                multiple: false,
                directory: false,
                filters: [
                    {
                        name: "Video",
                        extensions: ["mp4", "mkv", "avi", "mov", "webm"],
                    },
                ],
            });
            if (selected && typeof selected === "string") {
                selectedFilePath = selected;
                videoPath.set(selected);
                clearUndoHistory();
                subtitle.set([]); // Instantly clear previous subtitle array to free memory
            }
        } catch (error) {
            console.error("Error selecting file:", error);
            alert("Failed to select file: " + error);
        }
    }

    async function generate() {
        if (!selectedFilePath || localWordPerFrame <= 0) {
            alert("Upload a video file and enter words per frame");
            return;
        }
        wordPerFrame.set(localWordPerFrame);
        await goto("/loading");
    }

    onMount(async () => {
        await checkStatuses();
        unlistenProgress = await listen<{ model: string; progress: number }>(
            "model-download-progress",
            (event) => {
                const { model, progress } = event.payload;
                downloadProgress[model] = Math.round(progress);
                if (progress >= 100) {
                    isDownloading[model] = false;
                    modelStatuses[model] = true;
                }
            }
        );
    });

    onDestroy(() => {
        if (unlistenProgress) {
            unlistenProgress();
        }
    });
</script>

<main class="container">
    <div class="card animate-fade-in">
        <div class="header">
            <div class="logo-mark">OS</div>
            <div class="header-text">
                <h1 class="title">OpenSub</h1>
                <p class="subtitle">Subtitle generator for your videos</p>
            </div>
        </div>

        <div class="divider"></div>

        <div class="form-group">
            <span class="input-label">Video File</span>
            <div class="file-uploader">
                <button
                    type="button"
                    class="btn-select"
                    onclick={handleFileSelect}
                >
                    <span class="btn-select-icon">↑</span>
                    Select File
                </button>
                {#if selectedFilePath}
                    <div class="file-info animate-slide-in">
                        <span class="file-dot"></span>
                        <span class="file-path"
                            >{selectedFilePath.split("/").pop()}</span
                        >
                    </div>
                {:else}
                    <span class="no-file">No file chosen</span>
                {/if}
            </div>
        </div>

        <div class="form-group">
            <label for="wpf-input" class="input-label">Words Per Frame</label>
            <div class="input-wrapper">
                <input
                    id="wpf-input"
                    type="number"
                    min="1"
                    bind:value={localWordPerFrame}
                    placeholder="3"
                    class="num-input"
                />
                <span class="input-suffix">words</span>
            </div>
            <p class="helper-text">How many words appear on screen at once.</p>
        </div>

        <div class="form-group">
            <span class="input-label">Whisper Model</span>
            <div class="model-table-container">
                <table class="model-table">
                    <thead>
                        <tr>
                            <th>Select</th>
                            <th>Model</th>
                            <th>Size</th>
                            <th>Speed</th>
                            <th>Accuracy</th>
                            <th>Action</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each models as model}
                            <!-- svelte-ignore a11y_click_events_have_key_events -->
                            <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                            <tr
                                class="model-row"
                                class:active={currentModel === model.id}
                                onclick={() => {
                                    if (!isDownloading[model.id]) {
                                        currentModel = model.id;
                                    }
                                }}
                            >
                                <td class="td-radio">
                                    <input
                                        type="radio"
                                        name="model-select"
                                        value={model.id}
                                        bind:group={currentModel}
                                        disabled={isDownloading[model.id]}
                                        onclick={(e) => e.stopPropagation()}
                                    />
                                </td>
                                <td class="td-name">{model.name}</td>
                                <td class="td-meta">{model.size}</td>
                                <td class="td-meta">{model.speed}</td>
                                <td class="td-meta">{model.accuracy}</td>
                                <td class="td-action" onclick={(e) => e.stopPropagation()}>
                                    {#if modelStatuses[model.id]}
                                        <span class="badge badge-success">Downloaded</span>
                                    {:else if isDownloading[model.id]}
                                        <div class="download-progress-container">
                                            <div class="progress-bar">
                                                <div class="progress-fill" style="width: {downloadProgress[model.id] || 0}%"></div>
                                            </div>
                                            <span class="progress-pct">{downloadProgress[model.id] || 0}%</span>
                                        </div>
                                    {:else}
                                        <button
                                            type="button"
                                            class="btn-download"
                                            onclick={() => downloadModel(model.id)}
                                        >
                                            Download
                                        </button>
                                    {/if}
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
            {#if !modelStatuses[currentModel] && !isDownloading[currentModel]}
                <p class="helper-text warning-text">⚠️ Please download the selected model to generate subtitles.</p>
            {:else if isDownloading[currentModel]}
                <p class="helper-text info-text">⏳ Downloading model, please wait...</p>
            {/if}
        </div>

        <button
            type="button"
            class="btn-generate"
            onclick={generate}
            disabled={!selectedFilePath || localWordPerFrame <= 0 || !modelStatuses[currentModel] || isDownloading[currentModel]}
        >
            <span>Generate Subtitles</span>
            <span class="arrow">→</span>
        </button>
    </div>
</main>

<style>
    *,
    *::before,
    *::after {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }

    .container {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 100vh;
        padding: 2rem 1rem;
        background: #000000;
        color: #ffffff;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
            sans-serif;
    }

    /* ── Card ── */

    .card {
        background: #111111;
        border: 1px solid #2a2a2a;
        border-radius: 16px;
        width: 100%;
        max-width: 520px;
        padding: 2rem;
    }

    /* ── Animations ── */

    @keyframes fadeIn {
        from {
            opacity: 0;
            transform: translateY(16px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    @keyframes slideIn {
        from {
            opacity: 0;
            transform: translateX(-6px);
        }
        to {
            opacity: 1;
            transform: translateX(0);
        }
    }

    .animate-fade-in {
        animation: fadeIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
    }
    .animate-slide-in {
        animation: slideIn 0.25s ease both;
    }

    /* ── Header ── */

    .header {
        display: flex;
        align-items: center;
        gap: 0.85rem;
        margin-bottom: 1.5rem;
    }

    .logo-mark {
        width: 44px;
        height: 44px;
        background: #ffffff;
        color: #000000;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.75rem;
        font-weight: 800;
        letter-spacing: 0.05em;
        flex-shrink: 0;
    }

    .header-text {
        display: flex;
        flex-direction: column;
        gap: 0.15rem;
    }

    .title {
        font-size: 1.25rem;
        font-weight: 700;
        color: #ffffff;
        letter-spacing: -0.02em;
        line-height: 1;
    }

    .subtitle {
        font-size: 0.78rem;
        color: #666666;
        line-height: 1;
    }

    /* ── Divider ── */

    .divider {
        height: 1px;
        background: #1f1f1f;
        margin-bottom: 1.5rem;
    }

    /* ── Form ── */

    .form-group {
        margin-bottom: 1.35rem;
    }

    .input-label {
        display: block;
        font-size: 0.72rem;
        font-weight: 600;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: #888888;
        margin-bottom: 0.5rem;
    }

    .helper-text {
        font-size: 0.75rem;
        color: #555555;
        margin-top: 0.4rem;
    }

    /* ── File Uploader ── */

    .file-uploader {
        display: flex;
        align-items: center;
        gap: 0.65rem;
        flex-wrap: wrap;
    }

    .btn-select {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.5rem 0.9rem;
        background: #1e1e1e;
        color: #ffffff;
        border: 1px solid #333333;
        border-radius: 8px;
        font-size: 0.82rem;
        font-weight: 500;
        cursor: pointer;
        transition:
            background 120ms ease,
            border-color 120ms ease;
        white-space: nowrap;
        font-family: inherit;
    }

    .btn-select:hover {
        background: #2a2a2a;
        border-color: #444444;
    }

    .btn-select:active {
        background: #222222;
    }

    .btn-select-icon {
        font-size: 0.9rem;
        line-height: 1;
    }

    .no-file {
        font-size: 0.78rem;
        color: #444444;
    }

    .file-info {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        padding: 0.4rem 0.7rem;
        background: #1a1a1a;
        border: 1px solid #2a2a2a;
        border-radius: 6px;
        max-width: 180px;
        min-width: 0;
    }

    .file-dot {
        width: 6px;
        height: 6px;
        border-radius: 50%;
        background: #ffffff;
        flex-shrink: 0;
    }

    .file-path {
        font-size: 0.75rem;
        color: #cccccc;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    /* ── Number Input ── */

    .input-wrapper {
        display: flex;
        align-items: stretch;
        background: #1a1a1a;
        border: 1px solid #2e2e2e;
        border-radius: 8px;
        max-width: 150px;
        overflow: hidden;
        transition: border-color 120ms ease;
    }

    .input-wrapper:focus-within {
        border-color: #555555;
    }

    .num-input {
        flex: 1;
        padding: 0.5rem 0.7rem;
        font-size: 0.95rem;
        font-weight: 600;
        color: #ffffff;
        background: transparent;
        border: none;
        outline: none;
        -moz-appearance: textfield;
        min-width: 0;
        font-family: inherit;
    }

    .num-input::-webkit-outer-spin-button,
    .num-input::-webkit-inner-spin-button {
        -webkit-appearance: none;
        margin: 0;
    }

    .num-input::placeholder {
        color: #3a3a3a;
    }

    .input-suffix {
        display: flex;
        align-items: center;
        padding: 0 0.65rem;
        font-size: 0.7rem;
        font-weight: 500;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: #555555;
        border-left: 1px solid #2e2e2e;
        user-select: none;
    }

    /* ── Generate Button ── */

    .btn-generate {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        padding: 0.75rem 1rem;
        margin-top: 0.25rem;
        background: #ffffff;
        color: #000000;
        border: none;
        border-radius: 10px;
        font-size: 0.88rem;
        font-weight: 600;
        cursor: pointer;
        transition:
            background 120ms ease,
            opacity 120ms ease,
            transform 80ms ease;
        font-family: inherit;
    }

    .btn-generate:hover:not(:disabled) {
        background: #e8e8e8;
        transform: translateY(-1px);
    }

    .btn-generate:active:not(:disabled) {
        transform: translateY(0);
    }

    .btn-generate:disabled {
        background: #1e1e1e;
        color: #3a3a3a;
        cursor: not-allowed;
    }

    .arrow {
        font-size: 1rem;
        transition: transform 120ms ease;
    }

    .btn-generate:hover:not(:disabled) .arrow {
        transform: translateX(3px);
    }

    /* ── Responsive ── */

    @media (max-width: 500px) {
        .card {
            padding: 1.5rem;
        }

        .file-uploader {
            flex-direction: column;
            align-items: flex-start;
        }

        .file-info {
            max-width: 100%;
        }
    }
    /* ── Model Table ── */
    .model-table-container {
        margin-top: 0.5rem;
        background: #161616;
        border: 1px solid #222222;
        border-radius: 8px;
        overflow-x: auto;
    }

    .model-table {
        width: 100%;
        border-collapse: collapse;
        font-size: 0.8rem;
        text-align: left;
        min-width: 440px;
    }

    .model-table th {
        background: #1d1d1d;
        color: #888888;
        font-weight: 600;
        text-transform: uppercase;
        font-size: 0.65rem;
        letter-spacing: 0.08em;
        padding: 0.6rem 0.8rem;
        border-bottom: 1px solid #222222;
    }

    .model-row {
        cursor: pointer;
        transition: background 150ms ease;
        border-bottom: 1px solid #1a1a1a;
    }

    .model-row:last-child {
        border-bottom: none;
    }

    .model-row:hover {
        background: #1d1d1d;
    }

    .model-row.active {
        background: #202020;
    }

    .model-table td {
        padding: 0.7rem 0.8rem;
        vertical-align: middle;
    }

    .td-radio {
        width: 30px;
    }

    .td-name {
        font-weight: 600;
        color: #ffffff;
    }

    .td-meta {
        color: #aaaaaa;
    }

    .td-action {
        text-align: right;
    }

    .badge {
        display: inline-block;
        padding: 0.2rem 0.5rem;
        border-radius: 4px;
        font-size: 0.7rem;
        font-weight: 600;
    }

    .badge-success {
        background: rgba(46, 213, 115, 0.15);
        color: #2ed573;
        border: 1px solid rgba(46, 213, 115, 0.3);
    }

    .btn-download {
        background: #ffffff;
        color: #000000;
        border: none;
        border-radius: 4px;
        padding: 0.25rem 0.6rem;
        font-size: 0.7rem;
        font-weight: 600;
        cursor: pointer;
        transition: background 120ms ease;
        font-family: inherit;
    }

    .btn-download:hover {
        background: #e8e8e8;
    }

    .download-progress-container {
        display: inline-flex;
        align-items: center;
        gap: 0.4rem;
        width: 100px;
    }

    .progress-bar {
        flex: 1;
        height: 6px;
        background: #222222;
        border-radius: 3px;
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        background: #ffffff;
        transition: width 150ms ease;
    }

    .progress-pct {
        font-size: 0.7rem;
        color: #ffffff;
        font-weight: 600;
        min-width: 28px;
    }

    .warning-text {
        color: #ffa502;
        margin-top: 0.5rem;
    }

    .info-text {
        color: #70a1ff;
        margin-top: 0.5rem;
    }
</style>
