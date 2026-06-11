<script lang="ts">
    import { onDestroy, onMount, tick } from "svelte";
    import { gsap } from "gsap";

    type SplitType = "chars" | "words" | "lines" | string;
    type MotionVars = Record<string, any>;

    export let text: string = "";
    export let className: string = "";
    export let delay: number = 50;
    export let duration: number = 1.25;
    export let ease: string = "power3.out";
    export let splitType: SplitType = "chars";
    export let from: MotionVars = { opacity: 0, y: 40 };
    export let to: MotionVars = { opacity: 1, y: 0 };
    export let threshold: number = 0.1;
    export let rootMargin: string = "-100px";
    export let textAlign: "left" | "center" | "right" | "start" | "end" =
        "center";
    export let tag: keyof HTMLElementTagNameMap = "p";
    export let onLetterAnimationComplete: (() => void) | undefined = undefined;
    export let timeOffset: number | undefined = undefined;

    let ref: HTMLElement | null = null;
    let fontsLoaded = false;
    let inView = true;
    let animationCompleted = false;
    let observer: IntersectionObserver | null = null;
    let timeline: any = null;

    let splitItems: Array<{
        id: number;
        value: string;
        kind: "char" | "word" | "line" | "space";
        isSpace: boolean;
    }> = [];

    let signature = "";

    function buildTransformStyle(vars: MotionVars) {
        const parts: string[] = [];

        if (vars.opacity !== undefined) {
            parts.push(`opacity:${vars.opacity}`);
        }

        const transforms: string[] = [];
        if (typeof vars.x === "number")
            transforms.push(`translateX(${vars.x}px)`);
        if (typeof vars.y === "number")
            transforms.push(`translateY(${vars.y}px)`);
        if (typeof vars.z === "number")
            transforms.push(`translateZ(${vars.z}px)`);
        if (typeof vars.scale === "number")
            transforms.push(`scale(${vars.scale})`);
        if (typeof vars.rotate === "number")
            transforms.push(`rotate(${vars.rotate}deg)`);
        if (typeof vars.rotateX === "number")
            transforms.push(`rotateX(${vars.rotateX}deg)`);
        if (typeof vars.rotateY === "number")
            transforms.push(`rotateY(${vars.rotateY}deg)`);

        if (transforms.length) {
            parts.push(`transform:${transforms.join(" ")}`);
        }

        if (vars.filter) {
            parts.push(`filter:${vars.filter}`);
        }

        return parts.join("; ");
    }

    function buildSplitItems() {
        const mode = (splitType || "chars").toLowerCase();

        if (!text) {
            splitItems = [];
            return;
        }

        if (mode.includes("chars")) {
            splitItems = Array.from(text).map((ch, i) => ({
                id: i,
                value: ch,
                kind: /\s/.test(ch) ? "space" : "char",
                isSpace: /\s/.test(ch),
            }));
            return;
        }

        if (mode.includes("words")) {
            const parts = text.split(/(\s+)/);
            splitItems = parts.map((part, i) => ({
                id: i,
                value: part,
                kind: /^\s+$/.test(part) ? "space" : "word",
                isSpace: /^\s+$/.test(part),
            }));
            return;
        }

        const lines = text.split("\n");
        splitItems = lines.map((line, i) => ({
            id: i,
            value: line,
            kind: "line",
            isSpace: false,
        }));
    }

    function resetAnimation() {
        animationCompleted = false;
        timeline?.kill();
        timeline = null;
    }

    function runAnimation() {
        if (!ref || !fontsLoaded || !inView || animationCompleted) return;

        const targets = ref.querySelectorAll<HTMLElement>(
            '[data-split-target="true"]',
        );
        if (!targets.length) return;

        timeline?.kill();

        timeline = gsap.fromTo(
            targets,
            { ...from },
            {
                ...to,
                duration,
                ease,
                stagger: delay / 1000,
                force3D: true,
                willChange: "transform, opacity",
                paused: timeOffset !== undefined,
                onComplete: () => {
                    animationCompleted = true;
                    onLetterAnimationComplete?.();
                },
            },
        );

        if (timeOffset !== undefined) {
            timeline.seek(timeOffset);
        }
    }

    $: if (timeOffset !== undefined && timeline) {
        timeline.seek(timeOffset);
    }

    function refresh() {
        buildSplitItems();
        resetAnimation();
        tick().then(() => {
            if (fontsLoaded && inView) runAnimation();
        });
    }

    $: signature = [
        text,
        splitType,
        delay,
        duration,
        ease,
        threshold,
        rootMargin,
        JSON.stringify(from),
        JSON.stringify(to),
    ].join("|");

    $: if (signature) {
        refresh();
    }

    onMount(() => {
        if (typeof document !== "undefined" && (document as any).fonts) {
            if (document.fonts.status === "loaded") {
                fontsLoaded = true;
            } else {
                document.fonts.ready.then(() => {
                    fontsLoaded = true;
                    tick().then(() => {
                        if (inView) runAnimation();
                    });
                });
            }
        } else {
            fontsLoaded = true;
        }

        observer = new IntersectionObserver(
            ([entry]) => {
                inView = entry.isIntersecting;
                if (inView) runAnimation();
            },
            {
                threshold,
                rootMargin,
            },
        );

        if (ref) {
            observer.observe(ref);
        }

        tick().then(() => {
            if (fontsLoaded && inView) runAnimation();
        });

        return () => {
            observer?.disconnect();
            timeline?.kill();
        };
    });

    onDestroy(() => {
        observer?.disconnect();
        timeline?.kill();
    });

    $: wrapperStyle = `text-align:${textAlign}; overflow:hidden; display:inline-block; white-space:pre-wrap; word-break:break-word; will-change:transform, opacity;`;

    $: targetStyle = buildTransformStyle(from);

    $: classes = `split-parent ${className}`.trim();
</script>

<svelte:element this={tag} bind:this={ref} class={classes} style={wrapperStyle}>
    {#if splitType.toLowerCase().includes("lines")}
        {#each splitItems as item, i (item.id)}
            <span
                class="split-line"
                data-split-target="true"
                style={targetStyle}
            >
                {item.value}
            </span>
            {#if i < splitItems.length - 1}
                <br />
            {/if}
        {/each}
    {:else}
        {#each splitItems as item (item.id)}
            {#if item.isSpace}
                <span class="split-space">{item.value}</span>
            {:else}
                <span
                    class="split-target"
                    data-split-target="true"
                    style={targetStyle}
                >
                    {item.value}
                </span>
            {/if}
        {/each}
    {/if}
</svelte:element>

<style>
    .split-parent {
        width: 100%;
        position: relative;
        box-sizing: border-box;
    }

    .split-target,
    .split-line {
        display: inline-block;
        white-space: pre;
        box-sizing: border-box;
        backface-visibility: hidden;
        transform-origin: center;
    }

    .split-line {
        display: block;
    }

    .split-space {
        white-space: pre;
    }
</style>
