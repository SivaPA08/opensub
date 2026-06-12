import { get, writable } from "svelte/store";

const MAX_UNDO_HISTORY = 20;
const SNAPSHOT_DEBOUNCE_MS = 400;

const undoStack: Subtitle[][] = [];
let isUndoing = false;
let lastSnapshotTime = 0;

function cloneSubtitles(items: Subtitle[]): Subtitle[] {
    return items.map(item => ({ ...item }));
}

export function clearUndoHistory(): void {
    undoStack.length = 0;
    lastSnapshotTime = 0;
}

export function pushUndoSnapshot(force = false): void {
    if (isUndoing) return;

    const now = Date.now();
    if (!force && now - lastSnapshotTime < SNAPSHOT_DEBOUNCE_MS) return;

    lastSnapshotTime = now;
    undoStack.push(cloneSubtitles(get(subtitle)));
    if (undoStack.length > MAX_UNDO_HISTORY) {
        undoStack.shift();
    }
}

export function undoSubtitleChange(): boolean {
    if (undoStack.length === 0) return false;

    isUndoing = true;
    const previous = undoStack.pop()!;
    subtitle.set(cloneSubtitles(previous));
    isUndoing = false;
    return true;
}

export function setSubtitles(items: Subtitle[], options?: { recordUndo?: boolean }): void {
    if (options?.recordUndo !== false) {
        pushUndoSnapshot();
    }
    subtitle.set(items);
}

export function updateSubtitles(
    updater: (items: Subtitle[]) => Subtitle[],
    options?: { recordUndo?: boolean },
): void {
    if (options?.recordUndo !== false) {
        pushUndoSnapshot();
    }
    subtitle.update(updater);
}

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

export function updateSubtitleProperties(
    indices: number[],
    properties: Partial<Subtitle>,
    options?: { recordUndo?: boolean },
) {
    if (options?.recordUndo !== false) {
        pushUndoSnapshot();
    }
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

export const selectedModel = writable<string>("small"); // Default model is small