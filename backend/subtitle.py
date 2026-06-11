import sys
import subprocess
import json

# Immediately signal that the process has started and libraries are loading
print("PROGRESS:5.0", file=sys.stderr, flush=True)

from faster_whisper import WhisperModel

# Signal that the model initialization has begun
print("PROGRESS:15.0", file=sys.stderr, flush=True)

model = WhisperModel("small", compute_type="float32")

# Signal that the model is loaded and ready for transcription
print("PROGRESS:40.0", file=sys.stderr, flush=True)


def get_audio_duration(filepath: str) -> float:
    """Get audio duration in seconds using ffprobe."""
    try:
        result = subprocess.run(
            [
                "ffprobe", "-v", "quiet",
                "-print_format", "json",
                "-show_format", filepath
            ],
            capture_output=True, text=True
        )
        info = json.loads(result.stdout)
        return float(info["format"]["duration"])
    except Exception:
        return 0.0


def getvideo(filename: str, max_words: int):
    if filename==None or max_words==None:
        raise Exception("Filename and max_words are required")

    segments, info = model.transcribe(
        filename,
        word_timestamps=True
    )

    # Use transcription info duration if available, fallback to ffprobe
    duration = info.duration if (info and hasattr(info, 'duration') and info.duration) else get_audio_duration(filename)

    words = []

    for segment in segments:
        # Report progress based on segment end time vs total duration scaled from 40% to 95%
        if duration > 0:
            pct = min((segment.end / duration) * 100, 99.0)
            scaled_pct = 40.0 + (pct * 0.55)
            print(f"PROGRESS:{scaled_pct:.1f}", file=sys.stderr, flush=True)

        for word in segment.words:
            words.append({
                "word": word.word.strip(),
                "start": word.start,
                "end": word.end
            })

    # Signal completion
    print("PROGRESS:100.0", file=sys.stderr, flush=True)

    subs = []

    curr_words = []
    start_time = None
    end_time = None

    punctuation = (".", "?", "!")

    for w in words:

        if start_time is None:
            start_time = w["start"]

        curr_words.append(w["word"])
        end_time = w["end"]

        should_break = False

        if len(curr_words) >= max_words:
            should_break = True

        if w["word"].endswith(punctuation):
            should_break = True

        if should_break:

            subs.append({
                "text": " ".join(curr_words),
                "start": start_time,
                "end": end_time
            })

            curr_words = []
            start_time = None
            end_time = None

    if curr_words:

        subs.append({
            "text": " ".join(curr_words),
            "start": start_time,
            "end": end_time
        })

    return subs