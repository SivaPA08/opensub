import { writable } from "svelte/store";

export const videoPath = writable("");
export const videoDuration = writable(120); // Default timeline duration
export const videoCurrentTime = writable(0);

//for subtitle
export interface Subtitle {
    start: number;
    end: number;
    content: string;
}
export const subtitle = writable<Subtitle[]>([]);
export const wordPerFrame = writable<number>(-1);
export const subtitleFontSize = writable(28);