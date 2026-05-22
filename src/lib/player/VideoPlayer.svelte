<script lang="ts">
    import { videoPath } from "../store.js";
    import { invoke } from "@tauri-apps/api/core";

    let src: string | null = null;
    let loading = false;
    let loaded = false;
    let errorMessage: string | null = null;
    let prevPath = "";

    function getMimeType(path: string) {
        const ext = path.split(".").pop()?.toLowerCase();
        switch (ext) {
            case "mp4":
                return "video/mp4";
            case "webm":
                return "video/webm";
            case "ogg":
            case "ogv":
                return "video/ogg";
            case "mov":
                return "video/quicktime";
            case "avi":
                return "video/x-msvideo";
            case "mkv":
                return "video/x-matroska";
            default:
                return "video/mp4"; // Fallback to avoid empty MIME types being rejected
        }
    }

    $: if ($videoPath && $videoPath !== prevPath) {
        prevPath = $videoPath;
        loading = true;
        loaded = false;
        errorMessage = null;
        src = null;
        invoke<ArrayBuffer | Uint8Array | number[]>("get_video_bytes", {
            path: $videoPath,
        })
            .then((data) => {
                const mimeType = getMimeType($videoPath);
                let buffer: ArrayBuffer | Uint8Array;
                if (data instanceof Uint8Array || data instanceof ArrayBuffer) {
                    buffer = data;
                } else if (Array.isArray(data)) {
                    buffer = new Uint8Array(data);
                } else {
                    buffer = data as any;
                }
                const blob = new Blob([buffer as any], { type: mimeType });
                if (src) URL.revokeObjectURL(src);
                src = URL.createObjectURL(blob);
            })
            .catch((e) => {
                console.error("Failed loading video:", e, { path: $videoPath });
                src = null;
                errorMessage = "Cannot load video file from disk.";
            })
            .finally(() => (loading = false));
    }
</script>

{#if loading}
    <div class="placeholder">Loading video...</div>
{:else if errorMessage}
    <div class="placeholder error">{errorMessage}</div>
{:else if src}
    <div class="video-wrapper">
        <!-- svelte-ignore a11y_media_has_caption -->
        <video
            {src}
            controls
            preload="metadata"
            class="video"
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
                console.error("Video element error:", mediaError, "Src:", src);
                errorMessage = `Playback failed: ${mediaError ? mediaError.message : "file may be unsupported or corrupted."} (Code: ${mediaError ? mediaError.code : "unknown"})\nURL: ${src}`;
                loaded = false;
                src = null;
            }}
        >
            <track kind="captions" />
        </video>
        <div class="status">
            {#if loaded}
                Video loaded successfully.
            {:else}
                Loading video...
            {/if}
        </div>
    </div>
{:else}
    <div class="placeholder">No video selected</div>
{/if}

<style>
    .video-wrapper {
        width: 100%;
        max-height: 90vh;
    }
    .video {
        width: 100%;
        max-height: 90vh;
        object-fit: contain;
        display: block;
    }
    .placeholder {
        width: 100%;
        aspect-ratio: 16/9;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #666;
        font-size: 14px;
        border: 1px dashed #444;
        border-radius: 4px;
        background: #000;
        position: relative;
        z-index: 1;
    }
    .placeholder.error {
        color: #ffb3b3;
        border-color: #ff6b6b;
        background: #210000;
    }
    .status {
        margin-top: 0.75rem;
        color: #a8a8a8;
        font-size: 0.9rem;
        text-align: center;
    }
</style>
