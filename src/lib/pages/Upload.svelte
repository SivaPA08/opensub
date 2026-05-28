<script lang="ts">
    import { goto } from "$app/navigation";
    import { videoPath, wordPerFrame, subtitle } from "$lib/store";
    import { open } from "@tauri-apps/plugin-dialog";

    let selectedFilePath = $state<string | null>(null);
    let localWordPerFrame = $state<number>(3);

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

        <button
            type="button"
            class="btn-generate"
            onclick={generate}
            disabled={!selectedFilePath || localWordPerFrame <= 0}
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
        max-width: 440px;
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
</style>
