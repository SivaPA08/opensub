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
        try {
            const sub = await invoke<Pyres>("run_python", {
                name: videoUrl,
                count: count,
            });
            if (sub.status === "ok") {
                subtitle.set(sub.message as Subtitle[]);
                await goto("/editor");
            } else {
                alert(sub.message as string);
                await goto("/");
            }
        } catch (e) {
            alert("Backend error: " + e);
            await goto("/");
        }
    }

    onMount(() => {
        const video = get(videoPath);
        const count = get(wordPerFrame);
        if (!video) {
            goto("/");
            return;
        }
        setTimeout(() => generateSubtitle(video, count), 400);
    });
</script>

<main class="container">
    <div class="card">
        <div class="spinner"></div>
        <h1 class="title">Generating Subtitles</h1>
        <p class="subtitle">Processing your video, please wait...</p>
    </div>
</main>

<style>
    .container {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 100vh;
        background: #000000;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
            sans-serif;
        color: #ffffff;
    }

    .card {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.25rem;
        background: #111111;
        border: 1px solid #2a2a2a;
        border-radius: 16px;
        padding: 3rem 2.5rem;
        text-align: center;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid #2a2a2a;
        border-top-color: #ffffff;
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .title {
        font-size: 1.2rem;
        font-weight: 600;
        color: #ffffff;
        letter-spacing: -0.02em;
    }

    .subtitle {
        font-size: 0.82rem;
        color: #555555;
    }
</style>
