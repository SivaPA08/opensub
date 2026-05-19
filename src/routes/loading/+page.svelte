<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import {
        subtitle,
        videoPath,
        wordPerFrame,
        type Subtitle,
    } from "$lib/store";
    import { invoke } from "@tauri-apps/api/core";
    import { get } from "svelte/store";

    type Pyres = {
        status: string;
        message: Subtitle[] | string;
    };

    async function generateSubtitle(videoUrl: string, count: number) {
        console.log("invoking run_python with:", videoUrl, count);
        try {
            const sub = await invoke<Pyres>("run_python", {
                name: videoUrl,
                count: count,
            });
            console.log("invoke result:", sub);
            if (sub.status === "ok") {
                subtitle.set(sub.message as Subtitle[]);
                await goto("/editor");
            } else {
                alert(sub.message as string);
                await goto("/");
            }
        } catch (e) {
            console.error("invoke error:", e);
            alert("Backend error: " + e);
            await goto("/");
        }
    }

    onMount(async () => {
        const video = get(videoPath);
        const count = get(wordPerFrame);
        console.log("video:", video, "count:", count);
        if (!video) {
            await goto("/");
            return;
        }
        await generateSubtitle(video, count);
    });
</script>

<main>
    <h1>Generating Subtitles....</h1>
</main>
