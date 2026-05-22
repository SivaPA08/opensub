<script lang="ts">
    import { videoPath } from "../store.js";
    import { invoke } from "@tauri-apps/api/core";

    let videoSrc = "";
    let errorMessage = "";
    let loading = false;
    let loaded = false;
    let prevPath = "";

    $: if ($videoPath && $videoPath !== prevPath) {
        prevPath = $videoPath;
        loading = true;
        errorMessage = "";
        loaded = false;
        videoSrc = "";
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
        <!-- svelte-ignore a11y_media_has_caption -->
        <video
            src={videoSrc}
            controls
            autoplay
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
                const mediaError = (event.currentTarget as HTMLVideoElement).error;
                console.error("Video element error:", mediaError);
                errorMessage = `Playback failed: ${mediaError ? mediaError.message : "file may be unsupported or corrupted."}`;
                loaded = false;
                videoSrc = "";
            }}
        >
            <track kind="captions" />
        </video>
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
        max-height: 85vh;
        border-radius: 12px;
        overflow: hidden;
        border: 1px solid #2a2a2a;
        background: #000000;
        box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7);
    }
    
    .video {
        width: 100%;
        max-height: 80vh;
        object-fit: contain;
        display: block;
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
</style>
