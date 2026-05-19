<script lang="ts">
    import { goto } from "$app/navigation";
    import { subtitle, videoPath, type Subtitle } from "$lib/store";
    import { invoke } from "@tauri-apps/api/core";
    type Pyres = {
        status: string;
        message: Subtitle[] | string;
    };
    async function generateSubtitle(videoUrl: string, count: number) {
        const sub = await invoke<Pyres>("run_python", {
            name: videoUrl,
            count: count,
        });
        if (sub.status == "ok") {
            await goto("/editor");
        } else {
            alert("some error has occured click ok to go back");
            //need to write the go back function
        }
    }
</script>

<main>
    <h1>Generating Subtitles....</h1>
</main>
