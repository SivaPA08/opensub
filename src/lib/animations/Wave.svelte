<script lang="ts">
    import { onMount, onDestroy, tick } from "svelte";
    import { gsap } from "gsap";

    export let text: string = "";
    export let speed: number = 200;
    export let className: string = "";
    export let timeOffset: number | undefined = undefined;

    let containerEl: HTMLElement | undefined = undefined;
    let timeline: any = null;

    $: chars = Array.from(text ?? "");

    function buildTimeline() {
        if (!containerEl) return;

        const letters =
            containerEl.querySelectorAll<HTMLElement>("[data-wave-char]");
        if (!letters.length) return;

        timeline?.kill();

        const durationSec = speed / 1000;
        const staggerSec = Math.max(0.03, durationSec * 0.08);

        timeline = gsap.timeline({ paused: timeOffset !== undefined });

        gsap.set(letters, {
            opacity: 0,
            y: 12,
            scale: 0.96,
            transformOrigin: "center bottom",
        });

        letters.forEach((el, i) => {
            const start = i * staggerSec;

            timeline.to(
                el,
                {
                    opacity: 1,
                    y: -14,
                    scale: 1.08,
                    duration: durationSec * 0.35,
                    ease: "power2.out",
                },
                start,
            );

            timeline.to(
                el,
                {
                    y: 0,
                    scale: 1,
                    duration: durationSec * 0.3,
                    ease: "sine.inOut",
                },
                start + durationSec * 0.35,
            );
        });

        if (timeOffset !== undefined) {
            timeline.seek(timeOffset);
        }
    }

    $: if (timeOffset !== undefined && timeline) {
        timeline.seek(timeOffset);
    }

    let signature = "";
    $: {
        const newSig = `${text}|${speed}`;
        if (newSig !== signature) {
            signature = newSig;
            tick().then(() => buildTimeline());
        }
    }

    onMount(() => {
        tick().then(() => buildTimeline());

        return () => {
            timeline?.kill();
        };
    });

    onDestroy(() => {
        timeline?.kill();
    });
</script>

{#key text}
    <span bind:this={containerEl} class="wave-wrapper {className}">
        {#each chars as ch}
            <span class="wave-char" data-wave-char="true" aria-hidden="true">
                {ch === " " ? "\u00A0" : ch}
            </span>
        {/each}
    </span>
{/key}

<style>
    .wave-wrapper {
        display: inline-block;
        white-space: pre;
    }

    .wave-char {
        display: inline-block;
        will-change: transform, opacity;
    }
</style>
