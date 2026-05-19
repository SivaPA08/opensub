<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { subtitle, videoPath, type Subtitle } from "$lib/store";
    import { invoke } from "@tauri-apps/api/core";
    import { get } from "svelte/store";

    type Pyres = {
        status: string;
        message: Subtitle[] | string;
    };

    async function generateSubtitle(videoUrl: string, count: number) {
        const sub = await invoke<Pyres>("run_python", {
            name: videoUrl,
            count: count,
        });

        if (sub.status === "ok") {
            subtitle.set(sub.message as Subtitle[]);
            await goto("/editor");
        } else {
            alert("Some error has occurred");
            await goto("/");
        }
    }

    onMount(async () => {
        const video = get(videoPath);
        if (!video) {
            await goto("/");
            return;
        }

        await generateSubtitle(video, 3);
    });
</script>

<main>
    <h1>Generating Subtitles....</h1>
</main>
