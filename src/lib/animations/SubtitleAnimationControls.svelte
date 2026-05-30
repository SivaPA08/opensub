<script lang="ts">
    import { subtitleAnimation } from "../store.js";
    import ColorPicker from "../colorpicker/ColorPicker.svelte";

    // Bindings for local settings, initialized from store
    let fontSize = $subtitleAnimation.fontSize;
    let fontColor = $subtitleAnimation.fontColor;
    let backgroundColor = $subtitleAnimation.backgroundColor;
    let customFont = $subtitleAnimation.customFont;
    let fontOpacity = $subtitleAnimation.fontOpacity ?? 1.0;
    let backgroundOpacity = $subtitleAnimation.backgroundOpacity ?? 0.85;
    let customFontName = "";

    // Track active picker sections: "font" | "bg" | null
    let activePicker: "font" | "bg" | null = null;

    // Reactively update store when local values change
    $: subtitleAnimation.set({
        fontSize,
        fontColor,
        backgroundColor,
        customFont,
        fontOpacity,
        backgroundOpacity
    });

    // Helper to dynamically parse any hex/rgb/rgba to custom opacity rgba
    function hexOrRgbToRgba(color: string, opacity: number): string {
        if (!color) return `rgba(0,0,0,${opacity})`;
        
        // If it's already rgba, replace the alpha
        if (color.startsWith('rgba')) {
            return color.replace(/[\d\.]+\)$/, `${opacity})`);
        }
        
        // If it's rgb, convert to rgba
        if (color.startsWith('rgb')) {
            return color.replace('rgb', 'rgba').replace(')', `, ${opacity})`);
        }
        
        // If it's hex (#fff or #ffffff)
        if (color.startsWith('#')) {
            let hex = color.slice(1);
            if (hex.length === 3) {
                hex = hex.split('').map(c => c + c).join('');
            }
            const r = parseInt(hex.slice(0, 2), 16) || 0;
            const g = parseInt(hex.slice(2, 4), 16) || 0;
            const b = parseInt(hex.slice(4, 6), 16) || 0;
            return `rgba(${r}, ${g}, ${b}, ${opacity})`;
        }
        
        return color; // Fallback
    }

    // Handle Font File selection and loading
    function handleFontFile(event: Event) {
        const input = event.target as HTMLInputElement;
        if (input.files && input.files[0]) {
            const file = input.files[0];
            loadFont(file);
        }
    }

    function handleDragOver(e: DragEvent) {
        e.preventDefault();
    }

    function handleDrop(e: DragEvent) {
        e.preventDefault();
        if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0]) {
            loadFont(e.dataTransfer.files[0]);
        }
    }

    function loadFont(file: File) {
        const url = URL.createObjectURL(file);
        const fontName = `custom-${file.name.replace(/\W+/g, '-')}`;
        
        // Dynamic @font-face injection
        const style = document.createElement('style');
        style.id = `font-face-${fontName}`;
        style.innerHTML = `
            @font-face {
                font-family: '${fontName}';
                src: url('${url}');
                font-weight: normal;
                font-style: normal;
            }
        `;
        
        // Remove existing element of the same id if present
        const existing = document.getElementById(style.id);
        if (existing) {
            existing.remove();
        }
        document.head.appendChild(style);
        
        customFont = fontName;
        customFontName = file.name;
    }

    function resetFont() {
        customFont = "";
        customFontName = "";
    }

    function togglePicker(picker: "font" | "bg") {
        if (activePicker === picker) {
            activePicker = null;
        } else {
            activePicker = picker;
        }
    }
</script>

<div class="animation-controls animate-fade-in">
    <div class="header-row">
        <h3 class="section-title">
            <span class="sparkle-icon">✨</span> Subtitle Styling
        </h3>
    </div>

    <!-- Font Size Section -->
    <div class="control-card">
        <div class="card-header">
            <label for="fontSizeInput" class="card-label">Font Size</label>
            <span class="value-badge">{fontSize}px</span>
        </div>
        <div class="slider-container">
            <input 
                id="fontSizeInput" 
                type="range" 
                min="12" 
                max="80" 
                bind:value={fontSize} 
                class="premium-slider"
            />
            <input 
                type="number" 
                min="12" 
                max="80" 
                bind:value={fontSize} 
                class="size-number-input"
            />
        </div>
    </div>

    <!-- Font Color Picker Section -->
    <div class="control-card">
        <div class="card-header">
            <span class="card-label">Font Color</span>
            <!-- Interactive Color Swatch -->
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
                class="color-swatch-badge {activePicker === 'font' ? 'active' : ''}" 
                onclick={() => togglePicker('font')}
            >
                <span class="swatch" style="background-color: {fontColor};"></span>
                <span class="color-hex">{fontColor}</span>
            </div>
        </div>

        {#if activePicker === 'font'}
            <div class="picker-drawer animate-slide-down">
                <ColorPicker 
                    value={fontColor} 
                    onChange={(color) => fontColor = color} 
                />
            </div>
        {/if}

        <div class="slider-row-sub">
            <label for="fontOpacityInput" class="sub-label">Text Opacity</label>
            <input 
                id="fontOpacityInput" 
                type="range" 
                min="0" 
                max="1" 
                step="0.05" 
                bind:value={fontOpacity} 
                class="premium-slider"
            />
            <span class="sub-value-badge">{Math.round(fontOpacity * 100)}%</span>
        </div>
    </div>

    <!-- Background Color Picker Section -->
    <div class="control-card">
        <div class="card-header">
            <span class="card-label">Background Color</span>
            <!-- Interactive Color Swatch -->
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
                class="color-swatch-badge {activePicker === 'bg' ? 'active' : ''}" 
                onclick={() => togglePicker('bg')}
            >
                <span class="swatch" style="background-color: {backgroundColor};"></span>
                <span class="color-hex">{backgroundColor}</span>
            </div>
        </div>

        {#if activePicker === 'bg'}
            <div class="picker-drawer animate-slide-down">
                <ColorPicker 
                    value={backgroundColor} 
                    onChange={(color) => backgroundColor = color} 
                />
            </div>
        {/if}

        <div class="slider-row-sub">
            <label for="bgOpacityInput" class="sub-label">BG Opacity</label>
            <input 
                id="bgOpacityInput" 
                type="range" 
                min="0" 
                max="1" 
                step="0.05" 
                bind:value={backgroundOpacity} 
                class="premium-slider"
            />
            <span class="sub-value-badge">{Math.round(backgroundOpacity * 100)}%</span>
        </div>
    </div>

    <!-- Custom Font Upload Section -->
    <div class="control-card">
        <div class="card-header">
            <span class="card-label">Custom Typography</span>
            {#if customFont}
                <button class="reset-btn" onclick={resetFont}>Reset</button>
            {/if}
        </div>

        {#if customFont}
            <div class="active-font-badge">
                <span class="font-icon">🔤</span>
                <span class="font-name" title={customFontName}>{customFontName}</span>
            </div>
        {:else}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
                class="dropzone"
                ondragover={handleDragOver}
                ondrop={handleDrop}
            >
                <input 
                    id="fontFileInput"
                    type="file" 
                    accept=".ttf,.otf,.woff,.woff2" 
                    onchange={handleFontFile} 
                    class="hidden-file-input"
                />
                <label for="fontFileInput" class="dropzone-label">
                    <span class="upload-icon">📥</span>
                    <span class="dropzone-title">Upload Custom Font</span>
                    <span class="dropzone-subtitle">Drag & drop or browse (.ttf, .otf, .woff)</span>
                </label>
            </div>
        {/if}
    </div>

    <!-- Live Preview Display -->
    <div class="live-preview-box">
        <div class="preview-title">Live Preview</div>
        <div class="preview-canvas">
            <div class="preview-text-wrapper" style="
                background: {hexOrRgbToRgba(backgroundColor, backgroundOpacity)};
                font-family: {customFont || 'inherit'};
                font-size: {fontSize * 0.8}px;
                color: {hexOrRgbToRgba(fontColor, fontOpacity)};
                -webkit-backdrop-filter: blur({8 * backgroundOpacity}px);
                backdrop-filter: blur({8 * backgroundOpacity}px);
                border: 1px solid rgba(255, 255, 255, {0.1 * backgroundOpacity});
                box-shadow: 0 8px 32px rgba(0, 0, 0, {0.6 * backgroundOpacity}), inset 0 0 0 1px rgba(255, 255, 255, {0.15 * backgroundOpacity});
            ">
                Elegant Subtitle Preview
            </div>
        </div>
    </div>
</div>

<style>
    .animation-controls {
        background: #1e1e24;
        border: 1px solid #2d2d38;
        border-radius: 14px;
        padding: 20px;
        color: #e2e8f0;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
        display: flex;
        flex-direction: column;
        gap: 16px;
        max-width: 440px;
        box-sizing: border-box;
    }

    .header-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid #2d2d38;
        padding-bottom: 12px;
    }

    .section-title {
        font-size: 15px;
        font-weight: 700;
        margin: 0;
        letter-spacing: 0.03em;
        color: #00bcd4;
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .sparkle-icon {
        font-size: 16px;
    }

    /* Cards */
    .control-card {
        background: #25252f;
        border: 1px solid #323242;
        border-radius: 10px;
        padding: 12px;
        display: flex;
        flex-direction: column;
        gap: 10px;
        transition: border-color 0.2s, box-shadow 0.2s;
    }

    .control-card:hover {
        border-color: #3f3f54;
    }

    .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .card-label {
        font-size: 12px;
        font-weight: 600;
        color: #94a3b8;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .value-badge {
        font-size: 11px;
        font-weight: 700;
        background: #00bcd4;
        color: #111;
        padding: 2px 8px;
        border-radius: 12px;
    }

    /* Slider styling */
    .slider-container {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .premium-slider {
        flex: 1;
        accent-color: #00bcd4;
        cursor: pointer;
        height: 6px;
        border-radius: 3px;
        background: #3e3e4f;
        outline: none;
    }

    .size-number-input {
        width: 54px;
        background: #181820;
        border: 1px solid #323242;
        border-radius: 6px;
        color: #fff;
        font-size: 13px;
        font-weight: 600;
        text-align: center;
        padding: 4px;
        outline: none;
        transition: border-color 0.2s;
    }

    .size-number-input:focus {
        border-color: #00bcd4;
    }

    /* Swatches & Drawers */
    .color-swatch-badge {
        display: flex;
        align-items: center;
        gap: 8px;
        background: #181820;
        border: 1px solid #323242;
        border-radius: 20px;
        padding: 4px 10px;
        cursor: pointer;
        transition: border-color 0.2s, background-color 0.2s;
    }

    .color-swatch-badge:hover,
    .color-swatch-badge.active {
        border-color: #00bcd4;
        background: #1e1e28;
    }

    .swatch {
        width: 14px;
        height: 14px;
        border-radius: 50%;
        border: 1px solid rgba(255,255,255,0.2);
        box-shadow: 0 1px 3px rgba(0,0,0,0.5);
    }

    .color-hex {
        font-size: 11px;
        font-family: monospace;
        color: #e2e8f0;
        font-weight: 600;
    }

    .picker-drawer {
        margin-top: 4px;
        display: flex;
        justify-content: center;
        overflow: hidden;
        border-radius: 8px;
    }

    /* Font Uploader and Dropzone */
    .reset-btn {
        background: rgba(239, 68, 68, 0.15);
        border: 1px solid rgba(239, 68, 68, 0.4);
        color: #ef4444;
        font-size: 10px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        padding: 3px 10px;
        border-radius: 12px;
        cursor: pointer;
        transition: background-color 0.2s, border-color 0.2s;
    }

    .reset-btn:hover {
        background: #ef4444;
        color: #fff;
        border-color: #ef4444;
    }

    .active-font-badge {
        display: flex;
        align-items: center;
        gap: 8px;
        background: rgba(0, 188, 212, 0.1);
        border: 1px solid rgba(0, 188, 212, 0.3);
        border-radius: 8px;
        padding: 8px 12px;
        color: #00bcd4;
    }

    .font-icon {
        font-size: 16px;
    }

    .font-name {
        font-size: 12px;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        flex: 1;
    }

    .dropzone {
        border: 2px dashed #3f3f54;
        border-radius: 8px;
        background: #181820;
        padding: 16px;
        text-align: center;
        cursor: pointer;
        transition: border-color 0.2s, background-color 0.2s;
        position: relative;
    }

    .dropzone:hover {
        border-color: #00bcd4;
        background: #1c1c26;
    }

    .hidden-file-input {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        opacity: 0;
        cursor: pointer;
    }

    .dropzone-label {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 6px;
        pointer-events: none;
    }

    .upload-icon {
        font-size: 20px;
        color: #94a3b8;
    }

    .dropzone-title {
        font-size: 12px;
        font-weight: 700;
        color: #e2e8f0;
    }

    .dropzone-subtitle {
        font-size: 10px;
        color: #64748b;
    }

    /* Live Preview styling */
    .live-preview-box {
        margin-top: 8px;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .preview-title {
        font-size: 11px;
        font-weight: 700;
        color: #64748b;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .preview-canvas {
        background: #111;
        background-image: radial-gradient(rgba(255, 255, 255, 0.15) 1px, transparent 0);
        background-size: 12px 12px;
        border-radius: 10px;
        padding: 24px;
        min-height: 80px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1px solid #25252f;
    }

    .preview-text-wrapper {
        padding: 8px 18px;
        border-radius: 6px;
        font-weight: 600;
        text-align: center;
        max-width: 90%;
        word-wrap: break-word;
        transition: all 0.2s ease;
    }

    /* Animations */
    .animate-fade-in {
        animation: fadeIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
    }

    .animate-slide-down {
        animation: slideDown 0.3s cubic-bezier(0.16, 1, 0.3, 1) both;
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(4px); }
        to { opacity: 1; transform: translateY(0); }
    }

    @keyframes slideDown {
        from { opacity: 0; max-height: 0; transform: translateY(-10px); }
        to { opacity: 1; max-height: 250px; transform: translateY(0); }
    }

    .slider-row-sub {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-top: 8px;
        border-top: 1px solid #323242;
        padding-top: 8px;
    }

    .sub-label {
        font-size: 11px;
        font-weight: 600;
        color: #64748b;
        min-width: 70px;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .sub-value-badge {
        font-size: 10px;
        font-weight: 700;
        background: #181820;
        color: #00bcd4;
        padding: 2px 6px;
        border-radius: 8px;
        min-width: 32px;
        text-align: center;
        border: 1px solid #323242;
    }
</style>
