export type VideoPreviewMetrics = {
    videoWidth: number;
    videoHeight: number;
    containerWidth: number;
    containerHeight: number;
};

export function getPreviewScale(metrics: VideoPreviewMetrics): number {
    if (metrics.videoHeight <= 0 || metrics.containerHeight <= 0) {
        return 1;
    }
    return metrics.containerHeight / metrics.videoHeight;
}

/** Convert editor-preview pixels to video-native pixels for export. */
export function toVideoFontSize(
    previewFontSize: number,
    metrics: VideoPreviewMetrics,
): number {
    const scale = getPreviewScale(metrics);
    if (scale === 1) return previewFontSize;
    return Math.round(previewFontSize / scale);
}

/** Convert stored video-native pixels to editor-preview pixels. */
export function toPreviewFontSize(
    videoFontSize: number,
    metrics: VideoPreviewMetrics,
): number {
    return Math.round(videoFontSize * getPreviewScale(metrics));
}

export function getRenderScaleFactor(metrics: VideoPreviewMetrics): number {
    if (metrics.videoHeight <= 0 || metrics.containerHeight <= 0) {
        return 1;
    }
    return metrics.videoHeight / metrics.containerHeight;
}
