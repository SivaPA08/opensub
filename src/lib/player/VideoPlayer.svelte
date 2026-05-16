<script lang="ts">
    import { onDestroy } from "svelte";
    import { videoPath } from "../store.js";

    let videoUrl = "";
    const unsubscribe = videoPath.subscribe((value) => {
        videoUrl = value;
    });

    onDestroy(unsubscribe);
</script>

<!-- svelte-ignore a11y_media_has_caption -->
<video src={videoUrl}></video>
