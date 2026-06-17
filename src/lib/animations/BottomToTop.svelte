<script lang="ts">
    import { onMount, onDestroy, tick } from "svelte";
    import { gsap } from "gsap";

    export let text: string = "";
    export let speed: number = 200; // duration in ms
    export let className: string = "";
    export let timeOffset: number | undefined = undefined;

    let containerEl: HTMLElement | undefined = undefined;
    let timeline: any = null;

    function buildTimeline() {
        if (!containerEl) return;

        const target = containerEl.querySelector<HTMLElement>(
            "[data-popup-target]",
        );
        if (!target) return;

        timeline?.kill();

        const durationSec = speed / 1000;

        timeline = gsap.timeline({ paused: timeOffset !== undefined });

        // Bottom-to-top text animation
        timeline.fromTo(
            target,
            {
                opacity: 0,
                y: 20, // start lower
            },
            {
                opacity: 1,
                y: 0, // move to normal position
                duration: durationSec,
                ease: "power2.out",
            },
        );

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
    <span bind:this={containerEl} class="pop-up-wrapper {className}">
        <span
            class="pop-up-target"
            data-popup-target="true"
            style="opacity: 0; transform: translateY(20px);"
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
        will-change: transform, opacity;
    }
</style>
