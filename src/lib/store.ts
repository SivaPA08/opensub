import { writable } from "svelte/store";

export const videoPath = writable("");
export const videoDuration = writable(120); // Default timeline duration
export const videoCurrentTime = writable(0);

//for subtitle
export interface Subtitle {
    start: number;
    end: number;
    content: string;
    subX?: number;
    subY?: number;
    subWidth?: number;
    fontSize?: number;
    fontColor?: string;
    backgroundColor?: string;
    customFont?: string;
    customFontFile?: string;
    fontOpacity?: number;
    backgroundOpacity?: number;
    animationType?: string;
    animationSpeed?: number;
}
export const subtitle = writable<Subtitle[]>([]);
export const selectedSubtitleIndices = writable<number[]>([]);

export function updateSubtitleProperties(indices: number[], properties: Partial<Subtitle>) {
    subtitle.update(items => {
        return items.map((item, idx) => {
            if (indices.includes(idx)) {
                return { ...item, ...properties };
            }
            return item;
        });
    });
}

export const wordPerFrame = writable<number>(-1);
export const subtitleFontSize = writable(28);

// Subtitle animation style settings
export type SubtitleAnimationStyle = {
    fontSize: number;
    fontColor: string;
    backgroundColor: string;
    customFont: string; // Font family name or URL
    customFontFile: string; // Filename on disk
    fontOpacity: number; // 0 to 1
    backgroundOpacity: number; // 0 to 1
    animationType?: string;
    animationSpeed?: number;
};

export const subtitleAnimation = writable<SubtitleAnimationStyle>({
    fontSize: 28,
    fontColor: '#ffffff',
    backgroundColor: 'rgba(10, 10, 10, 0.85)',
    customFont: '',
    customFontFile: '',
    fontOpacity: 1.0,
    backgroundOpacity: 0.85,
    animationType: 'none',
    animationSpeed: 200
});