<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { gsap } from "gsap";

    export let text: string | string[] = "";
    export let as: string = "div";
    export let typingSpeed: number = 50;
    export let initialDelay: number = 0;
    export let pauseDuration: number = 2000;
    export let deletingSpeed: number = 30;
    export let loop: boolean = true;
    export let className: string = "";
    export let showCursor: boolean = true;
    export let hideCursorWhileTyping: boolean = false;
    export let cursorCharacter: string = "|";
    export let cursorClassName: string = "";
    export let cursorBlinkDuration: number = 0.5;
    export let textColors: string[] = [];
    export let variableSpeed: { min: number; max: number } | undefined = undefined;
    export let onSentenceComplete: ((sentence: string, index: number) => void) | undefined = undefined;
    export let startOnVisible: boolean = false;
    export let reverseMode: boolean = false;
    export let timeOffset: number | undefined = undefined;

    let displayedText: string = "";
    let currentCharIndex: number = 0;
    let isDeleting: boolean = false;
    let currentTextIndex: number = 0;
    let isVisible: boolean = !startOnVisible;

    let containerEl: HTMLElement | undefined = undefined;
    let cursorEl: HTMLElement | undefined = undefined;
    let observer: IntersectionObserver | undefined = undefined;
    let cursorTween: any = null;
    let timer: any = null;
    let cancelled: boolean = false;
    let started: boolean = false;
    let visibleResolver: (() => void) | null = null;

    let currentRunId: number = 0;
    let initialRender: boolean = true;

    $: textArray = Array.isArray(text) ? text : [text];
    $: currentText = textArray[currentTextIndex] ?? "";
    $: processedText = reverseMode
        ? currentText.split("").reverse().join("")
        : currentText;
    $: currentTextColor = textColors.length
        ? textColors[currentTextIndex % textColors.length]
        : "inherit";
    $: shouldHideCursor =
        hideCursorWhileTyping &&
        (isDeleting || currentCharIndex < processedText.length);

    // Reactively watch for text or speed changes to restart typing animation (only in real-time mode)
    $: {
        const _deps = [text, typingSpeed, loop];
        if (!initialRender) {
            if (timeOffset === undefined) {
                restartAnimation();
            }
        } else {
            initialRender = false;
        }
    }

    // Frame-accurate rendering mode driven by external clock
    $: if (timeOffset !== undefined) {
        const speedInSeconds = typingSpeed / 1000;
        if (speedInSeconds <= 0) {
            displayedText = currentText;
            currentCharIndex = currentText.length;
        } else {
            const count = Math.floor(timeOffset / speedInSeconds);
            currentCharIndex = Math.min(count, currentText.length);
            displayedText = currentText.slice(0, currentCharIndex);
        }
    }

    function clearTimer(): void {
        if (timer) {
            clearTimeout(timer);
            timer = null;
        }
    }

    function sleep(ms: number): Promise<void> {
        return new Promise((resolve) => {
            timer = setTimeout(resolve, ms);
        });
    }

    function getSpeed(): number {
        if (!variableSpeed) return typingSpeed;
        const { min, max } = variableSpeed;
        return Math.random() * (max - min) + min;
    }

    function waitForVisible(): Promise<void> {
        if (isVisible) return Promise.resolve();
        return new Promise((resolve) => {
            visibleResolver = resolve;
        });
    }

    async function runAnimation(): Promise<void> {
        if (started) return;
        started = true;

        const runId = ++currentRunId;

        if (initialDelay > 0) {
            await sleep(initialDelay);
            if (runId !== currentRunId) return;
        }

        while (runId === currentRunId && !cancelled) {
            if (!isVisible) {
                await waitForVisible();
                if (runId !== currentRunId || cancelled) break;
            }

            const sentence = textArray[currentTextIndex] ?? "";
            const typed = reverseMode
                ? sentence.split("").reverse().join("")
                : sentence;

            displayedText = "";
            currentCharIndex = 0;
            isDeleting = false;

            for (let i = 0; i < typed.length && runId === currentRunId && !cancelled; i++) {
                displayedText += typed[i];
                currentCharIndex = i + 1;
                await sleep(variableSpeed ? getSpeed() : typingSpeed);
            }

            if (runId !== currentRunId || cancelled) break;

            if (onSentenceComplete) {
                onSentenceComplete(sentence, currentTextIndex);
            }

            if (!loop && currentTextIndex === textArray.length - 1) {
                break;
            }

            await sleep(pauseDuration);
            if (runId !== currentRunId || cancelled) break;

            isDeleting = true;

            while (displayedText.length > 0 && runId === currentRunId && !cancelled) {
                displayedText = displayedText.slice(0, -1);
                currentCharIndex = Math.max(0, currentCharIndex - 1);
                await sleep(deletingSpeed);
            }

            isDeleting = false;
            currentTextIndex = (currentTextIndex + 1) % textArray.length;
        }
    }

    function restartAnimation(): void {
        clearTimer();
        displayedText = "";
        currentCharIndex = 0;
        isDeleting = false;
        currentTextIndex = 0;
        started = false;
        runAnimation();
    }

    onMount(() => {
        if (
            startOnVisible &&
            typeof IntersectionObserver !== "undefined" &&
            containerEl
        ) {
            observer = new IntersectionObserver(
                (entries) => {
                    for (const entry of entries) {
                        if (entry.isIntersecting) {
                            isVisible = true;
                            if (visibleResolver) {
                                visibleResolver();
                                visibleResolver = null;
                            }
                            observer?.disconnect();
                            break;
                        }
                    }
                },
                { threshold: 0.1 },
            );

            observer.observe(containerEl);
        } else {
            isVisible = true;
        }

        if (showCursor && cursorEl) {
            gsap.set(cursorEl, { opacity: 1 });
            cursorTween = gsap.to(cursorEl, {
                opacity: 0,
                duration: cursorBlinkDuration,
                repeat: -1,
                yoyo: true,
                ease: "power2.inOut",
            });
        }

        if (timeOffset === undefined) {
            runAnimation();
        }

        return () => {
            cancelled = true;
            clearTimer();
            observer?.disconnect();
            cursorTween?.kill();
        };
    });

    onDestroy(() => {
        cancelled = true;
        clearTimer();
        observer?.disconnect();
        cursorTween?.kill();
    });
</script>

<svelte:element
    this={as}
    bind:this={containerEl}
    class={`text-type ${className}`.trim()}
    {...$$restProps}
>
    <span class="text-type__content" style={`color: ${currentTextColor}`}
        >{displayedText}</span
    >

    {#if showCursor}
        <span
            bind:this={cursorEl}
            class={`text-type__cursor ${cursorClassName} ${shouldHideCursor ? "text-type__cursor--hidden" : ""}`.trim()}
        >
            {cursorCharacter}
        </span>
    {/if}
</svelte:element>

<style>
    .text-type {
        display: inline-block;
        white-space: pre-wrap;
    }

    .text-type__cursor {
        margin-left: 0.25rem;
        display: inline-block;
        opacity: 1;
    }

    .text-type__cursor--hidden {
        display: none;
    }
</style>
