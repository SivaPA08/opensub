import { writable } from "svelte/store";

export const videoPath = writable("")

//for subtitle
export interface Subtitle {
    start: number;
    end: number;
    content: string;
}
export const subtitle = writable<Subtitle[]>([])
export const wordPerFrame = writable<number>(-1)