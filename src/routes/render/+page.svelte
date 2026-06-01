<script lang="ts">
    import { onMount } from "svelte";

    // Subtitle properties
    let content = "";
    let subX = 50;
    let subY = 85;
    let subWidth = 70;

    // Styling properties
    let fontSize = 28;
    let fontColor = "#ffffff";
    let backgroundColor = "rgba(10, 10, 10, 0.85)";
    let customFont = "";
    let fontOpacity = 1.0;
    let backgroundOpacity = 0.85;

    // Helper to convert hex or rgb to rgba with a specific opacity
    function hexOrRgbToRgba(color: string, opacity: number): string {
        if (!color) return `rgba(0,0,0,${opacity})`;
        if (color.startsWith('rgba')) {
            return color.replace(/[\d\.]+\)$/, `${opacity})`);
        }
        if (color.startsWith('rgb')) {
            return color.replace('rgb', 'rgba').replace(')', `, ${opacity})`);
        }
        if (color.startsWith('#')) {
            let hex = color.slice(1);
            if (hex.length === 3) {
                hex = hex.split('').map(c => c + c).join('');
            }
            const r = parseInt(hex.slice(0, 2), 16);
            const g = parseInt(hex.slice(2, 4), 16);
            const b = parseInt(hex.slice(4, 6), 16);
            return `rgba(${r}, ${g}, ${b}, ${opacity})`;
        }
        return color;
    }

    onMount(() => {
        // Expose state update function to window for Playwright/Chromium control
        (window as any).setRenderState = (state: {
            content?: string;
            subX?: number;
            subY?: number;
            subWidth?: number;
            style?: {
                fontSize?: number;
                fontColor?: string;
                backgroundColor?: string;
                customFont?: string;
                fontOpacity?: number;
                backgroundOpacity?: number;
            };
            customFontBase64?: string; // Optional: Inject base64 font data directly
        }) => {
            if (state.content !== undefined) content = state.content;
            if (state.subX !== undefined) subX = state.subX;
            if (state.subY !== undefined) subY = state.subY;
            if (state.subWidth !== undefined) subWidth = state.subWidth;

            if (state.style) {
                const s = state.style;
                if (s.fontSize !== undefined) fontSize = s.fontSize;
                if (s.fontColor !== undefined) fontColor = s.fontColor;
                if (s.backgroundColor !== undefined) backgroundColor = s.backgroundColor;
                if (s.customFont !== undefined) customFont = s.customFont;
                if (s.fontOpacity !== undefined) fontOpacity = s.fontOpacity;
                if (s.backgroundOpacity !== undefined) backgroundOpacity = s.backgroundOpacity;
            }

            if (state.customFontBase64 && state.style?.customFont) {
                const fontName = state.style.customFont;
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
    {#if content}
        <div 
            class="subtitle-block"
            style="
                left: {subX}%; 
                top: {subY}%; 
                width: {subWidth}%; 
                transform: translate(-50%, -50%);
                background: {hexOrRgbToRgba(backgroundColor, backgroundOpacity)};
                font-family: {customFont || '-apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, sans-serif'};
                font-size: {fontSize}px;
                color: {hexOrRgbToRgba(fontColor, fontOpacity)};
                -webkit-backdrop-filter: blur({8 * backgroundOpacity}px);
                backdrop-filter: blur({8 * backgroundOpacity}px);
                border: 1px solid rgba(255, 255, 255, {0.1 * backgroundOpacity});
                box-shadow: 0 8px 32px rgba(0, 0, 0, {0.6 * backgroundOpacity}), inset 0 0 0 1px rgba(255, 255, 255, {0.15 * backgroundOpacity});
            "
        >
            <div 
                class="subtitle-text-render"
                style="
                    font-size: {fontSize}px; 
                    color: {hexOrRgbToRgba(fontColor, fontOpacity)}; 
                    font-family: {customFont || '-apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, sans-serif'};
                "
            >
                {content}
            </div>
        </div>
    {/if}
</div>

<style>
    :global(html), :global(body) {
        margin: 0;
        padding: 0;
        width: 100%;
        height: 100%;
        background: transparent !important;
        overflow: hidden;
    }

    .render-container {
        position: relative;
        width: 100%;
        height: 100%;
        background: transparent;
    }

    .subtitle-block {
        position: absolute;
        color: #ffffff;
        padding: 10px 24px;
        border-radius: 8px;
        text-align: center;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        min-height: 50px;
    }

    .subtitle-text-render {
        width: 100%;
        word-wrap: break-word;
        font-weight: 600;
        line-height: 1.4;
        position: relative;
    }
</style>
