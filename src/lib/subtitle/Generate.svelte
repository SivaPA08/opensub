<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { subtitle, type Subtitle } from "../store";
    async function generateSubtitle(videoUrl: string, count: number) {
        const sub = await invoke("run_python", {
            name: videoUrl,
            count: count,
        });
        subtitle.set(sub as Subtitle[]);
    }
</script>
