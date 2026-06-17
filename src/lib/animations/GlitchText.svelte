<script lang="ts">
    import { onMount, onDestroy, tick } from "svelte";
    import { gsap } from "gsap";

    export let text: string = "";
    export let speed: number = 200;
    export let className: string = "";
    export let timeOffset: number | undefined = undefined;

    let containerEl: HTMLElement | undefined = undefined;
    let timeline: any = null;

    function buildTimeline() {
        if (!containerEl) return;

        const target = containerEl.querySelector<HTMLElement>(
            "[data-glitch-target]",
        );
        if (!target) return;

        timeline?.kill();

        const durationSec = speed / 1000;

        timeline = gsap.timeline({ paused: timeOffset !== undefined });

        gsap.set(target, {
            opacity: 0,
            x: 0,
            y: 0,
            skewX: 0,
            filter: "blur(0px)",
        });

        timeline.to(target, {
            opacity: 1,
            duration: durationSec * 0.12,
            ease: "none",
        });

        timeline.to(target, {
            keyframes: [
                { x: -4, y: 1, skewX: 8, duration: 0.04 },
                { x: 3, y: -1, skewX: -10, duration: 0.04 },
                { x: -2, y: 0, skewX: 6, duration: 0.03 },
                { x: 5, y: 1, skewX: -8, duration: 0.05 },
                { x: -3, y: -1, skewX: 4, duration: 0.03 },
                { x: 0, y: 0, skewX: 0, duration: 0.06 },
            ],
            duration: durationSec * 0.42,
            ease: "none",
        });

        timeline.to(target, {
            keyframes: [
                { x: 2, duration: 0.02 },
                { x: -2, duration: 0.02 },
                { x: 1, duration: 0.02 },
                { x: 0, duration: 0.04 },
            ],
            duration: durationSec * 0.18,
            ease: "none",
        });

        timeline.to(target, {
            opacity: 1,
            x: 0,
            y: 0,
            skewX: 0,
            duration: durationSec * 0.12,
            ease: "power2.out",
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
    <span bind:this={containerEl} class="pop-up-wrapper {className}">
        <span class="pop-up-target" data-glitch-target="true" data-text={text}>
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
        position: relative;
        display: inline-block;
        transform-origin: center;
        will-change: transform, opacity, filter;
        color: white;
        text-shadow: 0 0 12px rgba(255, 255, 255, 0.12);
    }

    .pop-up-target::before,
    .pop-up-target::after {
        content: attr(data-text);
        position: absolute;
        inset: 0;
        pointer-events: none;
        opacity: 0.95;
    }

    .pop-up-target::before {
        left: 2px;
        color: #00e5ff;
        text-shadow: -2px 0 #00e5ff;
        animation: glitchTop 1.4s infinite steps(1, end);
        clip-path: inset(0 0 80% 0);
    }

    .pop-up-target::after {
        left: -2px;
        color: #ff2bd6;
        text-shadow: 2px 0 #ff2bd6;
        animation: glitchBottom 1.1s infinite steps(1, end);
        clip-path: inset(75% 0 0 0);
    }

    @keyframes glitchTop {
        0% {
            clip-path: inset(0 0 90% 0);
            transform: translate(0, 0);
        }
        10% {
            clip-path: inset(0 0 65% 0);
            transform: translate(-2px, -1px);
        }
        20% {
            clip-path: inset(0 0 35% 0);
            transform: translate(4px, 0px);
        }
        30% {
            clip-path: inset(0 0 78% 0);
            transform: translate(-4px, 1px);
        }
        40% {
            clip-path: inset(0 0 50% 0);
            transform: translate(2px, -1px);
        }
        50% {
            clip-path: inset(0 0 20% 0);
            transform: translate(-3px, 1px);
        }
        60% {
            clip-path: inset(0 0 72% 0);
            transform: translate(5px, 0px);
        }
        70% {
            clip-path: inset(0 0 45% 0);
            transform: translate(-2px, -1px);
        }
        80% {
            clip-path: inset(0 0 85% 0);
            transform: translate(3px, 0px);
        }
        90% {
            clip-path: inset(0 0 30% 0);
            transform: translate(-1px, 1px);
        }
        100% {
            clip-path: inset(0 0 60% 0);
            transform: translate(0, 0);
        }
    }

    @keyframes glitchBottom {
        0% {
            clip-path: inset(85% 0 0 0);
            transform: translate(0, 0);
        }
        10% {
            clip-path: inset(60% 0 0 0);
            transform: translate(2px, 1px);
        }
        20% {
            clip-path: inset(30% 0 0 0);
            transform: translate(-4px, 0px);
        }
        30% {
            clip-path: inset(75% 0 0 0);
            transform: translate(4px, -1px);
        }
        40% {
            clip-path: inset(48% 0 0 0);
            transform: translate(-2px, 1px);
        }
        50% {
            clip-path: inset(15% 0 0 0);
            transform: translate(3px, -1px);
        }
        60% {
            clip-path: inset(70% 0 0 0);
            transform: translate(-5px, 1px);
        }
        70% {
            clip-path: inset(40% 0 0 0);
            transform: translate(2px, 0px);
        }
        80% {
            clip-path: inset(82% 0 0 0);
            transform: translate(-3px, 1px);
        }
        90% {
            clip-path: inset(25% 0 0 0);
            transform: translate(1px, -1px);
        }
        100% {
            clip-path: inset(55% 0 0 0);
            transform: translate(0, 0);
        }
    }
</style>
