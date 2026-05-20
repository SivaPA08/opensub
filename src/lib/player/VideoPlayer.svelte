<script lang="ts">
    import { videoPath } from "../store.js";
    import { invoke } from "@tauri-apps/api/core";

    let blobUrl = "";
    let loadedPath = "";

    $: if ($videoPath && $videoPath !== loadedPath) {
        loadedPath = $videoPath;
        invoke<number[]>("get_video_bytes", { path: $videoPath }).then(
            (bytes) => {
                const blob = new Blob([new Uint8Array(bytes)], {
                    type: "video/mp4",
                });
                if (blobUrl) URL.revokeObjectURL(blobUrl);
                blobUrl = URL.createObjectURL(blob);
            },
        );
    }
</script>

<!-- svelte-ignore a11y_media_has_caption -->
<video
    src={blobUrl}
    controls
    preload="metadata"
    class="w-full"
    on:error={(e) => {
        const err = (e.target as HTMLVideoElement).error;
        console.error("MediaError code:", err?.code, err?.message);
    }}
    on:loadeddata={() => console.log("video loaded!")}
>
    <track kind="captions" />
</video>
