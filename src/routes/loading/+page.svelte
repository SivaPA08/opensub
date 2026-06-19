<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { goto } from "$app/navigation";
    import {
        subtitle,
        videoPath,
        wordPerFrame,
        clearUndoHistory,
        selectedModel,
        type Subtitle,
    } from "$lib/store";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { get } from "svelte/store";

    type Pyres = {
        status: string;
        message: Subtitle[] | string;
    };

    let progress = $state<number>(0);
    let statusText = $state<string>("Initializing model...");
    let unlisten: (() => void) | null = null;
    let progressTimer: ReturnType<typeof setInterval> | null = null;

    // Smooth progress interpolation:
    // Backend events arrive in sparse jumps (e.g. 40 → 76 → 100).
    // We ease `progress` toward `targetProgress` and add a slow creep
    // between updates so the bar always feels alive.
    let targetProgress = 0;
    let smoothProgress = 0;

    function startProgressAnimation() {
        progressTimer = setInterval(() => {
            // Snap to 100 immediately when done
            if (targetProgress >= 100) {
                smoothProgress = 100;
                progress = 100;
                return;
            }

            if (smoothProgress < targetProgress) {
                // Catch up to real backend value quickly
                const diff = targetProgress - smoothProgress;
                smoothProgress += Math.max(0.5, diff * 0.15);
                if (smoothProgress > targetProgress) smoothProgress = targetProgress;
            } else if (targetProgress >= 5) {
                // Asymptotic creep toward 98%: decelerates as it gets higher,
                // so the bar always moves but never actually reaches 98%.
                // At 40%: ~4.6%/sec, at 70%: ~1.7%/sec, at 90%: ~0.6%/sec
                const remaining = 98 - smoothProgress;
                if (remaining > 0.1) {
                    smoothProgress += remaining * 0.004;
                }
            }

            progress = Math.round(smoothProgress);
        }, 50);
    }

    async function generateSubtitle(videoUrl: string, count: number, model: string) {
        try {
            const sub = await invoke<Pyres>("getvideo", {
                filename: videoUrl,
                maxWords: count,
                modelName: model,
            });
            if (sub.status === "ok") {
                const rawSubs = sub.message as any[];
                const mapped: Subtitle[] = rawSubs.map((s) => ({
                    start: s.start,
                    end: s.end,
                    content: s.text || s.content || "",
                }));
                clearUndoHistory();
                subtitle.set(mapped);
                console.log("Mapped Subtitles:", mapped);
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

    onMount(async () => {
        // Listen for progress events from the Rust backend
        unlisten = await listen<number>("subtitle-progress", (event) => {
            targetProgress = event.payload;
            if (targetProgress < 10) {
                statusText = "Loading model...";
            } else if (targetProgress < 40) {
                statusText = "Preparing audio...";
            } else if (targetProgress < 100) {
                statusText = "Transcribing audio...";
            } else {
                statusText = "Finalizing subtitles...";
            }
        });

        startProgressAnimation();

        const video = get(videoPath);
        const count = get(wordPerFrame);
        const model = get(selectedModel);
        if (!video) {
            goto("/");
            return;
        }
        setTimeout(() => generateSubtitle(video, count, model), 400);
    });

    onDestroy(() => {
        if (progressTimer) clearInterval(progressTimer);
        if (unlisten) unlisten();
    });
</script>

<main class="container">
    <div class="card">
        <div class="spinner-wrapper">
            <svg class="progress-ring" viewBox="0 0 100 100">
                <circle class="ring-bg" cx="50" cy="50" r="42" />
                <circle
                    class="ring-fill"
                    cx="50"
                    cy="50"
                    r="42"
                    stroke-dashoffset={264 - (264 * progress) / 100}
                />
            </svg>
            <span class="progress-number"
                >{progress}<span class="pct">%</span></span
            >
        </div>
        <h1 class="title">Generating Subtitles</h1>
        <p class="subtitle">{statusText}</p>
        <div class="progress-bar-track">
            <div class="progress-bar-fill" style="width: {progress}%"></div>
        </div>
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
        min-width: 320px;
        animation: fadeIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
    }

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

    /* ── Circular progress ── */

    .spinner-wrapper {
        position: relative;
        width: 100px;
        height: 100px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .progress-ring {
        width: 100%;
        height: 100%;
        transform: rotate(-90deg);
    }

    .ring-bg {
        fill: none;
        stroke: #1e1e1e;
        stroke-width: 5;
    }

    .ring-fill {
        fill: none;
        stroke: #ffffff;
        stroke-width: 5;
        stroke-linecap: round;
        stroke-dasharray: 264;
        transition: stroke-dashoffset 0.6s cubic-bezier(0.25, 1, 0.5, 1);
    }

    .progress-number {
        position: absolute;
        font-size: 1.5rem;
        font-weight: 700;
        color: #ffffff;
        letter-spacing: -0.03em;
        line-height: 1;
    }

    .pct {
        font-size: 0.75rem;
        font-weight: 500;
        color: #666666;
        margin-left: 1px;
    }

    /* ── Text ── */

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

    /* ── Linear progress bar ── */

    .progress-bar-track {
        width: 100%;
        height: 3px;
        background: #1e1e1e;
        border-radius: 2px;
        overflow: hidden;
        margin-top: 0.25rem;
    }

    .progress-bar-fill {
        height: 100%;
        background: #ffffff;
        border-radius: 2px;
        transition: width 0.6s cubic-bezier(0.25, 1, 0.5, 1);
    }
</style>
