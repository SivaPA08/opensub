<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { pushUndoSnapshot } from "../store.js";

    // ── Props ──────────────────────────────────────────────────────────────────
    export let value: string = "rgba(255, 255, 255, 0.15)";
    export let onChange: (color: string) => void = () => {};

    // ── Internal HSV + Alpha state ─────────────────────────────────────────────
    let hue = 0; // 0-360
    let saturation = 0; // 0-100
    let brightness = 100; // 0-100
    let alpha = 0.15; // 0-1

    // ── DOM refs ───────────────────────────────────────────────────────────────
    let gradientCanvas: HTMLCanvasElement;
    let hueCanvas: HTMLCanvasElement;
    let alphaCanvas: HTMLCanvasElement;

    let gradientEl: HTMLDivElement;
    let hueEl: HTMLDivElement;
    let alphaEl: HTMLDivElement;

    // ── Drag state ─────────────────────────────────────────────────────────────
    type DragTarget = "gradient" | "hue" | "alpha" | null;
    let dragging: DragTarget = null;

    // ── Derived RGBA string ────────────────────────────────────────────────────
    $: rgbaString = toRgba(hue, saturation, brightness, alpha);
    $: {
        value = rgbaString;
        onChange(rgbaString);
        if (gradientCanvas) drawGradient();
        if (alphaCanvas) drawAlpha();
    }

    // ── Helpers ────────────────────────────────────────────────────────────────
    function hsvToRgb(
        h: number,
        s: number,
        v: number,
    ): [number, number, number] {
        s /= 100;
        v /= 100;
        const k = (n: number) => (n + h / 60) % 6;
        const f = (n: number) =>
            v * (1 - s * Math.max(0, Math.min(k(n), 4 - k(n), 1)));
        return [
            Math.round(f(5) * 255),
            Math.round(f(3) * 255),
            Math.round(f(1) * 255),
        ];
    }

    function toRgba(h: number, s: number, v: number, a: number): string {
        const [r, g, b] = hsvToRgb(h, s, v);
        const aStr = parseFloat(a.toFixed(2));
        return `rgba(${r}, ${g}, ${b}, ${aStr})`;
    }

    function hueToHex(h: number): string {
        const [r, g, b] = hsvToRgb(h, 100, 100);
        return `rgb(${r},${g},${b})`;
    }

    // ── Canvas renderers ───────────────────────────────────────────────────────
    function drawGradient() {
        const ctx = gradientCanvas.getContext("2d")!;
        const w = gradientCanvas.width;
        const h = gradientCanvas.height;
        ctx.clearRect(0, 0, w, h);

        // White → hue color (horizontal)
        const hGrad = ctx.createLinearGradient(0, 0, w, 0);
        hGrad.addColorStop(0, "#fff");
        hGrad.addColorStop(1, hueToHex(hue));
        ctx.fillStyle = hGrad;
        ctx.fillRect(0, 0, w, h);

        // Transparent → black (vertical)
        const vGrad = ctx.createLinearGradient(0, 0, 0, h);
        vGrad.addColorStop(0, "rgba(0,0,0,0)");
        vGrad.addColorStop(1, "rgba(0,0,0,1)");
        ctx.fillStyle = vGrad;
        ctx.fillRect(0, 0, w, h);
    }

    function drawHue() {
        const ctx = hueCanvas.getContext("2d")!;
        const w = hueCanvas.width;
        const h = hueCanvas.height;
        const grad = ctx.createLinearGradient(0, 0, 0, h);
        const stops = [0, 60, 120, 180, 240, 300, 360];
        stops.forEach((s) => {
            grad.addColorStop(s / 360, `hsl(${s},100%,50%)`);
        });
        ctx.fillStyle = grad;
        ctx.fillRect(0, 0, w, h);
    }

    function drawAlpha() {
        const ctx = alphaCanvas.getContext("2d")!;
        const w = alphaCanvas.width;
        const h = alphaCanvas.height;

        // Checkerboard
        const tileSize = 6;
        for (let x = 0; x < w; x += tileSize) {
            for (let y = 0; y < h; y += tileSize) {
                ctx.fillStyle =
                    (Math.floor(x / tileSize) + Math.floor(y / tileSize)) %
                        2 ===
                    0
                        ? "#666"
                        : "#999";
                ctx.fillRect(x, y, tileSize, tileSize);
            }
        }

        // Color gradient overlay
        const [r, g, b] = hsvToRgb(hue, saturation, brightness);
        const grad = ctx.createLinearGradient(0, 0, w, 0);
        grad.addColorStop(0, `rgba(${r},${g},${b},0)`);
        grad.addColorStop(1, `rgba(${r},${g},${b},1)`);
        ctx.fillStyle = grad;
        ctx.fillRect(0, 0, w, h);
    }

    // ── Gradient picker interaction ────────────────────────────────────────────
    function getRelativePos(e: MouseEvent | TouchEvent, el: HTMLElement) {
        const rect = el.getBoundingClientRect();
        const client = "touches" in e ? e.touches[0] : e;
        return {
            x: Math.max(0, Math.min(client.clientX - rect.left, rect.width)),
            y: Math.max(0, Math.min(client.clientY - rect.top, rect.height)),
            w: rect.width,
            h: rect.height,
        };
    }

    function handleGradientInteraction(e: MouseEvent | TouchEvent) {
        const { x, y, w, h } = getRelativePos(e, gradientEl);
        saturation = (x / w) * 100;
        brightness = 100 - (y / h) * 100;
    }

    function handleHueInteraction(e: MouseEvent | TouchEvent) {
        const { y, h } = getRelativePos(e, hueEl);
        hue = (y / h) * 360;
    }

    function handleAlphaInteraction(e: MouseEvent | TouchEvent) {
        const { x, w } = getRelativePos(e, alphaEl);
        alpha = parseFloat((x / w).toFixed(2));
    }

    // ── Global mouse/touch move + up ──────────────────────────────────────────
    function onMove(e: MouseEvent | TouchEvent) {
        if (!dragging) return;
        e.preventDefault();
        if (dragging === "gradient") handleGradientInteraction(e);
        if (dragging === "hue") handleHueInteraction(e);
        if (dragging === "alpha") handleAlphaInteraction(e);
    }

    function onUp() {
        dragging = null;
    }

    // ── Thumb positions ────────────────────────────────────────────────────────
    $: gradientThumbX = saturation; // percent
    $: gradientThumbY = 100 - brightness; // percent
    $: hueThumbY = (hue / 360) * 100; // percent
    $: alphaThumbX = alpha * 100; // percent

    // ── Lifecycle ─────────────────────────────────────────────────────────────
    onMount(() => {
        drawGradient();
        drawHue();
        drawAlpha();

        window.addEventListener("mousemove", onMove);
        window.addEventListener("mouseup", onUp);
        window.addEventListener("touchmove", onMove, { passive: false });
        window.addEventListener("touchend", onUp);
    });

    onDestroy(() => {
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
        window.removeEventListener("touchmove", onMove);
        window.removeEventListener("touchend", onUp);
    });

    // ── Hex input ─────────────────────────────────────────────────────────────
    let hexInput = "";
    $: {
        const [r, g, b] = hsvToRgb(hue, saturation, brightness);
        hexInput = rgbToHex(r, g, b);
    }

    function rgbToHex(r: number, g: number, b: number): string {
        return (
            "#" +
            [r, g, b]
                .map((v) => v.toString(16).padStart(2, "0"))
                .join("")
                .toUpperCase()
        );
    }

    function onHexChange(e: Event) {
        const raw = (e.target as HTMLInputElement).value
            .replace(/[^0-9a-fA-F]/g, "")
            .slice(0, 6);
        if (raw.length === 6) {
            const r = parseInt(raw.slice(0, 2), 16);
            const g = parseInt(raw.slice(2, 4), 16);
            const b = parseInt(raw.slice(4, 6), 16);
            const hsv = rgbToHsv(r, g, b);
            hue = hsv[0];
            saturation = hsv[1];
            brightness = hsv[2];
        }
    }

    function rgbToHsv(
        r: number,
        g: number,
        b: number,
    ): [number, number, number] {
        r /= 255;
        g /= 255;
        b /= 255;
        const max = Math.max(r, g, b),
            min = Math.min(r, g, b),
            d = max - min;
        let h = 0;
        if (d !== 0) {
            if (max === r) h = ((g - b) / d + 6) % 6;
            else if (max === g) h = (b - r) / d + 2;
            else h = (r - g) / d + 4;
            h = (h / 6) * 360;
        }
        return [h, max === 0 ? 0 : (d / max) * 100, max * 100];
    }
</script>

<!-- ── Markup ──────────────────────────────────────────────────────────────── -->
<div class="cp-root">
    <!-- Header -->
    <div class="cp-header">
        <span class="cp-icon">◑</span>
        <span class="cp-value">{rgbaString}</span>
    </div>

    <!-- Gradient + Side sliders row -->
    <div class="cp-body">
        <!-- Gradient panel -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="cp-gradient-wrap"
            bind:this={gradientEl}
            on:mousedown={(e) => {
                pushUndoSnapshot(true);
                dragging = "gradient";
                handleGradientInteraction(e);
            }}
            on:touchstart|preventDefault={(e) => {
                pushUndoSnapshot(true);
                dragging = "gradient";
                handleGradientInteraction(e);
            }}
        >
            <canvas
                bind:this={gradientCanvas}
                class="cp-canvas"
                width="340"
                height="200"
            />
            <!-- Thumb -->
            <div
                class="cp-thumb"
                style="left:{gradientThumbX}%; top:{gradientThumbY}%;"
            />
        </div>

        <!-- Right column: Hue + Alpha sliders -->
        <div class="cp-sliders">
            <!-- Hue slider -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                class="cp-hue-wrap"
                bind:this={hueEl}
                on:mousedown={(e) => {
                    pushUndoSnapshot(true);
                    dragging = "hue";
                    handleHueInteraction(e);
                }}
                on:touchstart|preventDefault={(e) => {
                    pushUndoSnapshot(true);
                    dragging = "hue";
                    handleHueInteraction(e);
                }}
            >
                <canvas
                    bind:this={hueCanvas}
                    class="cp-canvas"
                    width="22"
                    height="200"
                />
                <div class="cp-slider-thumb" style="top:{hueThumbY}%;" />
            </div>

            <!-- Alpha slider (vertical) -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                class="cp-alpha-wrap"
                bind:this={alphaEl}
                on:mousedown={(e) => {
                    pushUndoSnapshot(true);
                    dragging = "alpha";
                    handleAlphaInteraction(e);
                }}
                on:touchstart|preventDefault={(e) => {
                    pushUndoSnapshot(true);
                    dragging = "alpha";
                    handleAlphaInteraction(e);
                }}
            >
                <canvas
                    bind:this={alphaCanvas}
                    class="cp-canvas cp-alpha-canvas"
                    width="22"
                    height="200"
                />
                <div
                    class="cp-slider-thumb"
                    style="top:{(1 - alpha) * 100}%;"
                />
            </div>
        </div>
    </div>

    <!-- Footer: info label -->
    <div class="cp-footer">
        <div class="cp-info">
            Shorthand property for setting border width, style, and color.
        </div>
        <div class="cp-compat">
            <span class="cp-check">✓✓</span>
            <em>Widely available across major browsers (Baseline since 2015)</em
            >
        </div>
    </div>
</div>

<!-- ── Styles ─────────────────────────────────────────────────────────────── -->
<style>
    .cp-root {
        --bg: #2a2a2a;
        --bg-panel: #333333;
        --border: #444;
        --text: #e8e8e8;
        --text-dim: #aaa;
        --radius: 6px;
        --thumb-sz: 12px;

        background: var(--bg);
        border: 1px solid var(--border);
        border-radius: var(--radius);
        overflow: hidden;
        width: 420px;
        font-family: "Segoe UI", system-ui, sans-serif;
        font-size: 13px;
        color: var(--text);
        user-select: none;
    }

    /* Header */
    .cp-header {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        background: var(--bg-panel);
        border-bottom: 1px solid var(--border);
    }
    .cp-icon {
        font-size: 16px;
        opacity: 0.85;
    }
    .cp-value {
        font-size: 12px;
        color: var(--text-dim);
        letter-spacing: 0.02em;
    }

    /* Body */
    .cp-body {
        display: flex;
        gap: 8px;
        padding: 8px;
    }

    /* Gradient */
    .cp-gradient-wrap {
        position: relative;
        flex: 1;
        border-radius: 4px;
        overflow: hidden;
        cursor: crosshair;
    }

    .cp-canvas {
        display: block;
        width: 100%;
        height: 100%;
        border-radius: 4px;
    }

    /* Gradient thumb */
    .cp-thumb {
        position: absolute;
        width: var(--thumb-sz);
        height: var(--thumb-sz);
        border-radius: 50%;
        border: 2px solid #fff;
        transform: translate(-50%, -50%);
        pointer-events: none;
        box-shadow:
            0 0 0 1px rgba(0, 0, 0, 0.5),
            0 1px 4px rgba(0, 0, 0, 0.6);
    }

    /* Right sliders column */
    .cp-sliders {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    /* Hue & Alpha track wrappers */
    .cp-hue-wrap,
    .cp-alpha-wrap {
        position: relative;
        width: 22px;
        flex: 1;
        border-radius: 4px;
        overflow: visible;
        cursor: pointer;
    }

    .cp-alpha-canvas {
        border-radius: 4px;
    }

    /* Horizontal thumb on vertical sliders */
    .cp-slider-thumb {
        position: absolute;
        left: 50%;
        width: 22px;
        height: 6px;
        border-radius: 3px;
        background: #ddd;
        border: 1px solid #888;
        transform: translate(-50%, -50%);
        pointer-events: none;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
    }

    /* Footer */
    .cp-footer {
        padding: 8px 12px 10px;
        border-top: 1px solid var(--border);
        background: var(--bg-panel);
    }

    .cp-info {
        color: var(--text);
        font-size: 13px;
        margin-bottom: 4px;
    }

    .cp-compat {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 12px;
        color: var(--text-dim);
        font-style: italic;
    }

    .cp-check {
        color: #5c9e5c;
        font-style: normal;
        font-size: 13px;
    }
</style>
