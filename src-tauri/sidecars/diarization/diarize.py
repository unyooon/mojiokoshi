"""Long-lived JSON-lines speaker diarization sidecar.

Reads requests from stdin, processes them with pyannote.audio,
and writes responses to stdout. Never crashes -- all errors are
wrapped in error response envelopes.

Set MOJIOKOSHI_DIARIZE_STUB=1 to use a mock pipeline that returns
two speakers alternating every 5 seconds.
"""

import json
import os
import sys
import traceback

STUB_MODE = os.environ.get("MOJIOKOSHI_DIARIZE_STUB") == "1"

_pipeline = None


def _get_pipeline():
    """Lazy-load the pyannote pipeline on first real request."""
    global _pipeline
    if _pipeline is None:
        from pyannote.audio import Pipeline

        _pipeline = Pipeline.from_pretrained(
            "pyannote/speaker-diarization-3.1",
            use_auth_token=os.environ.get("HF_TOKEN"),
        )
    return _pipeline


def _stub_diarize(audio_path, num_speakers):
    """Return mock segments: 2 speakers alternating every 5 seconds."""
    import soundfile as sf

    info = sf.info(audio_path) if os.path.exists(audio_path) else None
    duration = info.duration if info else 30.0
    segments = []
    t = 0.0
    speaker_idx = 0
    while t < duration:
        end = min(t + 5.0, duration)
        segments.append({
            "speaker": f"SPEAKER_{speaker_idx}",
            "start": round(t, 3),
            "end": round(end, 3),
        })
        speaker_idx = 1 - speaker_idx
        t = end
    return segments


def _real_diarize(audio_path, num_speakers):
    """Run pyannote pipeline on the audio file."""
    pipeline = _get_pipeline()
    kwargs = {}
    if num_speakers is not None:
        kwargs["num_speakers"] = num_speakers
    diarization = pipeline(audio_path, **kwargs)
    segments = []
    for turn, _, speaker in diarization.itertracks(yield_label=True):
        segments.append({
            "speaker": speaker,
            "start": round(turn.start, 3),
            "end": round(turn.end, 3),
        })
    return segments


def handle_request(request):
    """Process a single diarization request and return a response dict."""
    req_id = request.get("id", "unknown")
    try:
        req_type = request.get("type")
        if req_type != "diarize":
            return {
                "id": req_id,
                "type": "error",
                "payload": {"message": f"Unknown request type: {req_type}"},
            }

        payload = request.get("payload", {})
        audio_path = payload.get("audio_path", "")
        num_speakers = payload.get("num_speakers")

        if STUB_MODE:
            segments = _stub_diarize(audio_path, num_speakers)
        else:
            segments = _real_diarize(audio_path, num_speakers)

        return {
            "id": req_id,
            "type": "response",
            "payload": {"segments": segments},
        }
    except Exception:
        return {
            "id": req_id,
            "type": "error",
            "payload": {"message": traceback.format_exc()},
        }


def main():
    """Main loop: read JSON lines from stdin, write responses to stdout."""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError as e:
            response = {
                "id": "unknown",
                "type": "error",
                "payload": {"message": f"Invalid JSON: {e}"},
            }
            sys.stdout.write(json.dumps(response) + "\n")
            sys.stdout.flush()
            continue

        response = handle_request(request)
        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
