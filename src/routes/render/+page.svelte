<script lang="ts">
    import { onMount } from "svelte";
    import GlitchText from "$lib/animations/GlitchText.svelte";
    import SplitText from "$lib/animations/SplitText.svelte";
    import TypingText from "$lib/animations/TypingText.svelte";
    import DecriptText from "$lib/animations/DecriptText.svelte";
    import PopUp from "$lib/animations/PopUp.svelte";

    type RenderSubtitle = {
        content: string;
        subX: number;
        subY: number;
        subWidth: number;
        timeOffset: number;
        fontSize: number;
        fontColor: string;
        backgroundColor: string;
        customFont: string;
        fontOpacity: number;
        backgroundOpacity: number;
        animationType: string;
        animationSpeed: number;
    };

    let activeSubtitles: RenderSubtitle[] = [];

    // Scaling factor (editor px -> video px)
    let scaleFactor = 1.0;

    // Helper to convert hex or rgb to rgba with a specific opacity
    function hexOrRgbToRgba(color: string, opacity: number): string {
        if (!color) return `rgba(0,0,0,${opacity})`;

        // If it's already rgba, replace the alpha
        if (color.startsWith("rgba")) {
            return color.replace(/[\d\.]+\)$/, `${opacity})`);
        }

        // If it's rgb, convert to rgba
        if (color.startsWith("rgb")) {
            return color.replace("rgb", "rgba").replace(")", `, ${opacity})`);
        }

        // If it's hex (#fff or #ffffff)
        if (color.startsWith("#")) {
            let hex = color.slice(1);
            if (hex.length === 3) {
                hex = hex
                    .split("")
                    .map((c) => c + c)
                    .join("");
            }
            const r = parseInt(hex.slice(0, 2), 16) || 0;
            const g = parseInt(hex.slice(2, 4), 16) || 0;
            const b = parseInt(hex.slice(4, 6), 16) || 0;
            return `rgba(${r}, ${g}, ${b}, ${opacity})`;
        }
        return color;
    }

    onMount(() => {
        // Expose state update function to window for Playwright/Chromium control
        (window as any).setRenderState = (state: {
            subtitles?: Array<{
                content: string;
                subX?: number;
                subY?: number;
                subWidth?: number;
                timeOffset?: number;
                style?: {
                    fontSize?: number;
                    fontColor?: string;
                    backgroundColor?: string;
                    customFont?: string;
                    fontOpacity?: number;
                    backgroundOpacity?: number;
                    animationType?: string;
                    animationSpeed?: number;
                };
            }>;
            content?: string;
            subX?: number;
            subY?: number;
            subWidth?: number;
            scaleFactor?: number;
            timeOffset?: number;
            style?: {
                fontSize?: number;
                fontColor?: string;
                backgroundColor?: string;
                customFont?: string;
                fontOpacity?: number;
                backgroundOpacity?: number;
                animationType?: string;
                animationSpeed?: number;
            };
            customFontBase64?: string;
            customFontName?: string;
        }) => {
            if (state.subtitles !== undefined) {
                activeSubtitles = state.subtitles.map((sub: any) => {
                    const s = sub.style || {};
                    return {
                        content: sub.content || "",
                        subX: sub.subX !== undefined ? sub.subX : 50,
                        subY: sub.subY !== undefined ? sub.subY : 85,
                        subWidth:
                            sub.subWidth !== undefined ? sub.subWidth : 70,
                        timeOffset:
                            sub.timeOffset !== undefined ? sub.timeOffset : 0,
                        fontSize: s.fontSize !== undefined ? s.fontSize : 28,
                        fontColor: s.fontColor || "#ffffff",
                        backgroundColor:
                            s.backgroundColor || "rgba(10, 10, 10, 0.85)",
                        customFont: s.customFont || "",
                        fontOpacity:
                            s.fontOpacity !== undefined ? s.fontOpacity : 1.0,
                        backgroundOpacity:
                            s.backgroundOpacity !== undefined
                                ? s.backgroundOpacity
                                : 0.85,
                        animationType: s.animationType || "none",
                        animationSpeed:
                            s.animationSpeed !== undefined
                                ? s.animationSpeed
                                : 200,
                    };
                });
            } else if (state.content !== undefined) {
                // Fallback for single subtitle
                const s = state.style || {};
                activeSubtitles = [
                    {
                        content: state.content || "",
                        subX: state.subX !== undefined ? state.subX : 50,
                        subY: state.subY !== undefined ? state.subY : 85,
                        subWidth:
                            state.subWidth !== undefined ? state.subWidth : 70,
                        timeOffset:
                            state.timeOffset !== undefined
                                ? state.timeOffset
                                : 0,
                        fontSize: s.fontSize !== undefined ? s.fontSize : 28,
                        fontColor: s.fontColor || "#ffffff",
                        backgroundColor:
                            s.backgroundColor || "rgba(10, 10, 10, 0.85)",
                        customFont: s.customFont || "",
                        fontOpacity:
                            s.fontOpacity !== undefined ? s.fontOpacity : 1.0,
                        backgroundOpacity:
                            s.backgroundOpacity !== undefined
                                ? s.backgroundOpacity
                                : 0.85,
                        animationType: s.animationType || "none",
                        animationSpeed:
                            s.animationSpeed !== undefined
                                ? s.animationSpeed
                                : 200,
                    },
                ];
            } else if (state.content === "") {
                activeSubtitles = [];
            }

            if (state.scaleFactor !== undefined)
                scaleFactor = state.scaleFactor;

            if (state.customFontBase64 && state.customFontName) {
                const fontName = state.customFontName;
                const styleId = `font-face-${fontName}`;
                let styleEl = document.getElementById(styleId);
                if (!styleEl) {
                    styleEl = document.createElement("style");
                    styleEl.id = styleId;
                    document.head.appendChild(styleEl);
                }
                styleEl.innerHTML = `
                    @font-face {
                        font-family: '${fontName}';
                        src: url('data:font/truetype;charset=utf-8;base64,${state.customFontBase64}');
                        font-weight: normal;
                        font-style: normal;
                    }
                `;
            }
        };

        // Let the renderer know we are ready
        (window as any).renderPageReady = true;
    });
</script>

<div class="render-container">
    {#each activeSubtitles as sub}
        {#if sub.content}
            <div
                class="subtitle-block"
                style="
                    left: {sub.subX}%; 
                    top: {sub.subY}%; 
                    width: {sub.subWidth}%; 
                    transform: translate(-50%, -50%);
                    background: {hexOrRgbToRgba(
                    sub.backgroundColor,
                    sub.backgroundOpacity,
                )};
                    font-family: {sub.customFont ||
                    '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'};
                    font-size: {sub.fontSize * scaleFactor}px;
                    color: {hexOrRgbToRgba(sub.fontColor, sub.fontOpacity)};
                    padding: {10 * scaleFactor}px {24 * scaleFactor}px;
                    border-radius: {8 * scaleFactor}px;
                    min-height: {50 * scaleFactor}px;
                    -webkit-backdrop-filter: blur({8 *
                    scaleFactor *
                    sub.backgroundOpacity}px);
                    backdrop-filter: blur({8 *
                    scaleFactor *
                    sub.backgroundOpacity}px);
                    border: {1 * scaleFactor}px solid rgba(255, 255, 255, {0.1 *
                    sub.backgroundOpacity});
                    box-shadow: 0 {8 * scaleFactor}px {32 *
                    scaleFactor}px rgba(0, 0, 0, {0.6 *
                    sub.backgroundOpacity}), inset 0 0 0 {1 *
                    scaleFactor}px rgba(255, 255, 255, {0.15 *
                    sub.backgroundOpacity});
                "
            >
                {#if sub.animationType === "glitch"}
                    <GlitchText
                        text={sub.content}
                        speed={sub.animationSpeed / 200}
                        enableShadows={true}
                        enableOnHover={false}
                    />
                {:else}
                    <div
                        class="subtitle-text-render"
                        style="
                            font-size: {sub.fontSize * scaleFactor}px; 
                            color: {hexOrRgbToRgba(
                            sub.fontColor,
                            sub.fontOpacity,
                        )}; 
                            font-family: {sub.customFont ||
                            '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif'};
                            line-height: 1.4;
                        "
                    >
                        {#if sub.animationType === "split-text"}
                            <SplitText
                                text={sub.content}
                                duration={sub.animationSpeed / 1000}
                                delay={sub.animationSpeed / 10 || 10}
                                timeOffset={sub.timeOffset}
                            />
                        {:else if sub.animationType === "typing"}
                            <TypingText
                                text={sub.content}
                                typingSpeed={sub.animationSpeed ?? 50}
                                timeOffset={sub.timeOffset}
                                loop={false}
                                showCursor={true}
                            />
                        {:else if sub.animationType === "decrypt"}
                            <DecriptText
                                text={sub.content}
                                speed={sub.animationSpeed ?? 50}
                                timeOffset={sub.timeOffset}
                                animateOn="view"
                                sequential={true}
                            />
                        {:else if sub.animationType === "pop-up"}
                            <PopUp
                                text={sub.content}
                                speed={sub.animationSpeed}
                                timeOffset={sub.timeOffset}
                            />
                        {:else}
                            {sub.content}
                        {/if}
                    </div>
                {/if}
            </div>
        {/if}
    {/each}
</div>

<style>
    :global(html),
    :global(body) {
        margin: 0;
        padding: 0;
        width: 100%;
        height: 100%;
        background: transparent !important;
        background-color: transparent !important;
        overflow: hidden;
    }

    .render-container {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: transparent !important;
        background-color: transparent !important;
        overflow: hidden;
        box-sizing: border-box;
    }

    .subtitle-block {
        position: absolute;
        color: #ffffff;
        text-align: center;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        word-wrap: break-word;
        white-space: pre-wrap;
    }

    .subtitle-text-render {
        width: 100%;
        word-wrap: break-word;
        font-weight: 600;
        position: relative;
    }
</style>
