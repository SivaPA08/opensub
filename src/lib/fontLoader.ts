import { invoke } from "@tauri-apps/api/core";

const loadedFonts = new Set<string>();

export async function ensureFontLoaded(fontPath: string): Promise<string> {
    if (!fontPath) return "";
    const fileName = fontPath.split('/').pop() || fontPath;
    const fontName = `custom-${fileName.replace(/\W+/g, '-')}`;
    
    if (loadedFonts.has(fontPath)) {
        return fontName;
    }
    
    try {
        const base64Data = await invoke<string>("get_font_base64", { path: fontPath });
        
        const style = document.createElement('style');
        style.id = `font-face-${fontName}`;
        style.innerHTML = `
            @font-face {
                font-family: '${fontName}';
                src: url('data:font/truetype;charset=utf-8;base64,${base64Data}');
                font-weight: normal;
                font-style: normal;
            }
        `;
        
        const existing = document.getElementById(style.id);
        if (existing) {
            existing.remove();
        }
        document.head.appendChild(style);
        loadedFonts.add(fontPath);
        return fontName;
    } catch (err) {
        console.error("Failed to load font background-wise:", err);
        return "";
    }
}
