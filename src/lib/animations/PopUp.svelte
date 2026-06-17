<script lang="ts">
    import { onMount, onDestroy, tick } from "svelte";
    import { gsap } from "gsap";

    export let text: string = "";
    export let speed: number = 200; // speed/duration in ms
    export let className: string = "";
    export let timeOffset: number | undefined = undefined;

    let containerEl: HTMLElement | undefined = undefined;
    let timeline: any = null;

    function buildTimeline() {
        if (!containerEl) return;

        const target = containerEl.querySelector<HTMLElement>('[data-popup-target]');
        if (!target) return;

        timeline?.kill();

        const durationSec = speed / 1000;

        timeline = gsap.timeline({ paused: timeOffset !== undefined });

        // Match the original CSS keyframes: scale(0.88)+translateY(8px)+blur(2px) → scale(1.04)+translateY(0) → scale(1)
        timeline.fromTo(
            target,
            {
                opacity: 0,
                scale: 0.88,
                y: 8,
                filter: "blur(2px)",
            },
            {
                opacity: 1,
                scale: 1.04,
                y: 0,
                filter: "blur(0px)",
                duration: durationSec * 0.65,
                ease: "power2.out",
            },
        );

        timeline.to(target, {
            scale: 1,
            duration: durationSec * 0.35,
            ease: "power2.inOut",
        });

        if (timeOffset !== undefined) {
            timeline.seek(timeOffset);
        }
    }

    // Reactively seek when timeOffset changes during headless rendering
    $: if (timeOffset !== undefined && timeline) {
        timeline.seek(timeOffset);
    }

    // Rebuild when text or speed changes
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
    <span
        bind:this={containerEl}
        class="pop-up-wrapper {className}"
    >
        <span
            class="pop-up-target"
            data-popup-target="true"
            style="opacity: 0; transform: scale(0.88) translateY(8px); filter: blur(2px);"
        >
            {text}
        </span>
    </span>
{/key}

<style>
    .pop-up-wrapper {
        display: inline-block;
        width: 100%;
    }

    .pop-up-target {
        display: inline-block;
        transform-origin: center;
        will-change: transform, opacity, filter;
    }
</style>
