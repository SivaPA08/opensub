<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { gsap } from "gsap";

    export let text: string = "";
    export let as: string = "span";
    export let speed: number = 50;
    export let maxIterations: number = 10;
    export let sequential: boolean = false;
    export let revealDirection: "start" | "end" | "center" = "start";
    export let useOriginalCharsOnly: boolean = false;
    export let characters: string =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()_+";
    export let className: string = "";
    export let parentClassName: string = "";
    export let encryptedClassName: string = "";
    export let animateOn: "hover" | "click" | "view" | "inViewHover" = "hover";
    export let clickMode: "once" | "toggle" = "once";
    export let timeOffset: number | undefined = undefined;

    let displayText: string = text;
    let isAnimating: boolean = false;
    let revealedIndices: Set<number> = new Set<number>();
    let hasAnimated: boolean = false;
    let isDecrypted: boolean = animateOn !== "click";
    let direction: "forward" | "reverse" = "forward";

    let containerRef: HTMLElement | undefined = undefined;
    let cursorTween: any = null;
    let observer: IntersectionObserver | undefined = undefined;

    let runId: number = 0;
    let prevText: string = text;
    let prevAnimateOn: string = animateOn;

    $: availableChars = useOriginalCharsOnly
        ? Array.from(new Set(String(text).split(""))).filter(
              (char) => char !== " ",
          )
        : characters.split("");

    $: if (availableChars.length === 0) {
        availableChars = [" "];
    }

    function sleep(ms: number): Promise<void> {
        return new Promise((resolve) => setTimeout(resolve, ms));
    }

    function shuffleText(originalText: string, currentRevealed: Set<number>): string {
        return String(originalText)
            .split("")
            .map((char, i) => {
                if (char === " ") return " ";
                if (currentRevealed.has(i)) return originalText[i];
                return availableChars[
                    Math.floor(Math.random() * availableChars.length)
                ];
            })
            .join("");
    }

    function computeOrder(len: number): number[] {
        const order: number[] = [];
        if (len <= 0) return order;

        if (revealDirection === "start") {
            for (let i = 0; i < len; i++) order.push(i);
            return order;
        }

        if (revealDirection === "end") {
            for (let i = len - 1; i >= 0; i--) order.push(i);
            return order;
        }

        const middle = Math.floor(len / 2);
        let offset = 0;

        while (order.length < len) {
            if (offset === 0) {
                if (middle >= 0 && middle < len) order.push(middle);
            } else if (offset % 2 === 1) {
                const right = middle + Math.ceil(offset / 2);
                if (right >= 0 && right < len) order.push(right);
            } else {
                const left = middle - offset / 2;
                if (left >= 0 && left < len) order.push(left);
            }
            offset++;
        }

        return order.slice(0, len);
    }

    function fillAllIndices(): Set<number> {
        const s = new Set<number>();
        for (let i = 0; i < String(text).length; i++) s.add(i);
        return s;
    }

    function removeRandomIndices(set: Set<number>, count: number): Set<number> {
        const arr = Array.from(set);
        for (let i = 0; i < count && arr.length > 0; i++) {
            const idx = Math.floor(Math.random() * arr.length);
            arr.splice(idx, 1);
        }
        return new Set<number>(arr);
    }

    function stopCurrentRun(): void {
        runId += 1;
        isAnimating = false;
    }

    function encryptInstantly(): void {
        const emptySet = new Set<number>();
        revealedIndices = emptySet;
        displayText = shuffleText(text, emptySet);
        isDecrypted = false;
    }

    function resetToPlainText(): void {
        stopCurrentRun();
        revealedIndices = new Set<number>();
        displayText = String(text);
        isDecrypted = true;
        direction = "forward";
    }

    async function triggerDecrypt(): Promise<void> {
        stopCurrentRun();
        const id = runId;

        revealedIndices = new Set<number>();
        direction = "forward";
        isAnimating = true;
        isDecrypted = false;

        if (sequential) {
            const order = computeOrder(String(text).length);

            for (let i = 0; i < order.length; i++) {
                if (id !== runId) return;
                const nextIndex = order[i];
                const nextSet = new Set(revealedIndices);
                nextSet.add(nextIndex);
                revealedIndices = nextSet;
                displayText = shuffleText(text, nextSet);
                await sleep(speed);
            }

            if (id !== runId) return;
            displayText = String(text);
            isAnimating = false;
            isDecrypted = true;
            return;
        }

        for (let iter = 0; iter < maxIterations; iter++) {
            if (id !== runId) return;
            displayText = shuffleText(text, revealedIndices);
            await sleep(speed);
        }

        if (id !== runId) return;
        displayText = String(text);
        isAnimating = false;
        isDecrypted = true;
    }

    async function triggerReverse(): Promise<void> {
        stopCurrentRun();
        const id = runId;

        direction = "reverse";
        isAnimating = true;
        isDecrypted = false;

        if (sequential) {
            const order = computeOrder(String(text).length).slice().reverse();
            revealedIndices = fillAllIndices();
            displayText = shuffleText(text, revealedIndices);

            for (let i = 0; i < order.length; i++) {
                if (id !== runId) return;
                const idxToRemove = order[i];
                const nextSet = new Set(revealedIndices);
                nextSet.delete(idxToRemove);
                revealedIndices = nextSet;
                displayText = shuffleText(text, nextSet);
                await sleep(speed);

                if (nextSet.size === 0) {
                    if (id !== runId) return;
                    isAnimating = false;
                    isDecrypted = false;
                    return;
                }
            }

            if (id !== runId) return;
            isAnimating = false;
            isDecrypted = false;
            return;
        }

        revealedIndices = fillAllIndices();
        displayText = shuffleText(text, revealedIndices);

        for (let iter = 0; iter < maxIterations; iter++) {
            if (id !== runId) return;

            const removeCount = Math.max(
                1,
                Math.ceil(String(text).length / Math.max(1, maxIterations)),
            );
            const nextSet = removeRandomIndices(revealedIndices, removeCount);
            revealedIndices = nextSet;
            displayText = shuffleText(text, nextSet);
            await sleep(speed);

            if (nextSet.size === 0) {
                break;
            }
        }

        if (id !== runId) return;
        displayText = shuffleText(text, new Set<number>());
        isAnimating = false;
        isDecrypted = false;
    }

    function handleClick(): void {
        if (animateOn !== "click") return;

        if (clickMode === "once") {
            if (isDecrypted) return;
            direction = "forward";
            triggerDecrypt();
            return;
        }

        if (clickMode === "toggle") {
            if (isDecrypted) {
                triggerReverse();
            } else {
                direction = "forward";
                triggerDecrypt();
            }
        }
    }

    function triggerHoverDecrypt(): void {
        if (isAnimating) return;
        direction = "forward";
        triggerDecrypt();
    }

    onMount(() => {
        if (timeOffset === undefined) {
            if (animateOn === "click") {
                encryptInstantly();
            } else {
                displayText = String(text);
                isDecrypted = true;
            }

            if (animateOn === "view" || animateOn === "inViewHover") {
                observer = new IntersectionObserver(
                    (entries) => {
                        for (const entry of entries) {
                            if (entry.isIntersecting && !hasAnimated) {
                                hasAnimated = true;
                                triggerDecrypt();
                                break;
                            }
                        }
                    },
                    { root: null, rootMargin: "0px", threshold: 0.1 },
                );

                if (containerRef) observer.observe(containerRef);
            }
        }

        return () => {
            stopCurrentRun();
            observer?.disconnect();
            cursorTween?.kill();
        };
    });

    onDestroy(() => {
        stopCurrentRun();
        observer?.disconnect();
        cursorTween?.kill();
    });

    $: if (text !== prevText || animateOn !== prevAnimateOn) {
        prevText = text;
        prevAnimateOn = animateOn;

        if (timeOffset === undefined) {
            stopCurrentRun();
            revealedIndices = new Set<number>();
            direction = "forward";
            hasAnimated = false;

            if (animateOn === "click") {
                encryptInstantly();
            } else {
                displayText = String(text);
                isDecrypted = true;
            }
        }
    }

    // Frame-accurate rendering mode driven by external clock
    $: if (timeOffset !== undefined) {
        const speedInSeconds = speed / 1000;
        if (sequential) {
            const order = computeOrder(String(text).length);
            const count = speedInSeconds <= 0 ? order.length : Math.floor(timeOffset / speedInSeconds);
            const nextSet = new Set<number>();
            for (let i = 0; i < Math.min(count, order.length); i++) {
                nextSet.add(order[i]);
            }
            revealedIndices = nextSet;
            if (count >= order.length) {
                displayText = String(text);
                isAnimating = false;
                isDecrypted = true;
            } else {
                isAnimating = true;
                isDecrypted = false;
                displayText = shuffleText(text, nextSet);
            }
        } else {
            const iter = speedInSeconds <= 0 ? maxIterations : Math.floor(timeOffset / speedInSeconds);
            if (iter >= maxIterations) {
                displayText = String(text);
                isAnimating = false;
                isDecrypted = true;
                revealedIndices = fillAllIndices();
            } else {
                isAnimating = true;
                isDecrypted = false;
                revealedIndices = new Set<number>();
                displayText = shuffleText(text, revealedIndices);
            }
        }
    }

    $: shouldUseHover = animateOn === "hover" || animateOn === "inViewHover";
    $: shouldUseClick = animateOn === "click";
</script>

<svelte:element
    this={as}
    bind:this={containerRef}
    class={`decrypted-text ${parentClassName}`.trim()}
    on:mouseenter={shouldUseHover ? triggerHoverDecrypt : undefined}
    on:mouseleave={shouldUseHover ? resetToPlainText : undefined}
    on:click={shouldUseClick ? handleClick : undefined}
    {...$$restProps}
>
    <span class="sr-only">{displayText}</span>

    <span aria-hidden="true">
        {#each displayText.split("") as char, index}
            {@const isRevealedOrDone =
                revealedIndices.has(index) || (!isAnimating && isDecrypted)}
            <span class={isRevealedOrDone ? className : encryptedClassName}>
                {char}
            </span>
        {/each}
    </span>
</svelte:element>

<style>
    .decrypted-text {
        display: inline-block;
        white-space: pre-wrap;
    }

    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        border: 0;
    }
</style>
