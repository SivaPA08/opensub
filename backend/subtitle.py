from faster_whisper import WhisperModel

model = WhisperModel("small", compute_type="float32")


def getvideo(filename: str, max_words: int):
    if filename==None or max_words==None:
        raise Exception("Filename and max_words are required")

    segments, info = model.transcribe(
        filename,
        word_timestamps=True
    )

    words = []

    for segment in segments:
        for word in segment.words:
            words.append({
                "word": word.word.strip(),
                "start": word.start,
                "end": word.end
            })

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