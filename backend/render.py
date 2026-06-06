import sys
import json
import re
import os
import base64
import socket
import http.server
import socketserver
import threading
import subprocess
from playwright.sync_api import sync_playwright


def get_free_port():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.bind(("", 0))
    port = s.getsockname()[1]
    s.close()
    return port


class SPABackupHandler(http.server.SimpleHTTPRequestHandler):
    """
    HTTP Handler that serves Svelte SPA static files and falls back
    to index.html for virtual client-side SPA routing (like /render).
    """

    def do_GET(self):
        # Resolve real path of requested resource
        path = self.translate_path(self.path)
        if not os.path.exists(path) or os.path.isdir(path):
            # Fall back to Svelte SPA single entry point
            self.path = "/index.html"
        super().do_GET()

    def log_message(self, format, *args):
        # Silence HTTP server logging to avoid polluting stdout/stderr
        pass


def get_video_info(filepath: str):
    """Get video resolution, framerate, and duration using ffprobe."""
    try:
        result = subprocess.run(
            [
                "ffprobe", "-v", "quiet",
                "-print_format", "json",
                "-show_streams", "-show_format", filepath
            ],
            capture_output=True, text=True
        )
        info = json.loads(result.stdout)
        width, height, fps, duration = 1280, 720, 30.0, 0.0

        if "format" in info and "duration" in info["format"]:
            duration = float(info["format"]["duration"])

        for stream in info.get("streams", []):
            if stream.get("codec_type") == "video":
                width = int(stream["width"])
                height = int(stream["height"])

                fps_str = stream.get("avg_frame_rate", "30/1")
                if "/" in fps_str:
                    num, den = fps_str.split("/")
                    if float(den) > 0:
                        fps = float(num) / float(den)
                else:
                    fps = float(fps_str)

                if "duration" in stream:
                    duration = float(stream["duration"])
                break

        return width, height, fps, duration
    except Exception as e:
        print(f"DEBUG ERROR: Failed to probe video: {e}", file=sys.stderr, flush=True)
        return 1280, 720, 30.0, 0.0


def run():
    if len(sys.argv) < 2:
        print(json.dumps({"status": "error", "message": "Config path is required"}))
        return

    config_path = sys.argv[1]
    server = None

    try:
        with open(config_path, "r", encoding="utf-8") as f:
            config = json.load(f)

        print(f"DEBUG: Starting frame-by-frame HTML/CSS render pipeline...", file=sys.stderr, flush=True)
        print(f"DEBUG CONFIG: {json.dumps(config, indent=2)}", file=sys.stderr, flush=True)

        input_path = config["input_path"]
        output_path = config["output_path"]
        subtitles = config["subtitles"]
        style = config["style"]
        pos = config.get("position", {"subX": 50, "subY": 85, "subWidth": 70})

        # Get exact video parameters
        width, height, fps, duration = get_video_info(input_path)
        if duration <= 0:
            duration = 10.0  # fallback duration

        total_frames = int(round(duration * fps))
        if total_frames <= 0:
            total_frames = 300

        # Calculate scale factor (editor px -> video px)
        video_info = config.get("video_info", {})
        container_width = video_info.get("container_width", width)
        container_height = video_info.get("container_height", height)
        scale_factor = height / container_height if container_height > 0 else 1.0

        print(f"DEBUG: Video resolution: {width}x{height}, FPS: {fps:.2f}, Duration: {duration:.2f}s, Total Frames: {total_frames}", file=sys.stderr, flush=True)
        print(f"DEBUG: Computed scale factor: {scale_factor:.4f} (height={height}, container_height={container_height})", file=sys.stderr, flush=True)

        # -------------------------------------------------------------------------
        # Start a local static HTTP server serving the Svelte built site
        # -------------------------------------------------------------------------
        script_dir = os.path.dirname(os.path.abspath(__file__))
        workspace_dir = os.path.dirname(script_dir)
        build_dir = os.path.join(workspace_dir, "build")

        if not os.path.isdir(build_dir):
            raise Exception(f"Svelte build directory not found at {build_dir}. Please build the project first.")

        port = get_free_port()

        def start_server():
            nonlocal server
            # Serve the static Svelte SPA
            handler = lambda *args, **kwargs: SPABackupHandler(*args, directory=build_dir, **kwargs)
            server = socketserver.TCPServer(("127.0.0.1", port), handler)
            server.serve_forever()

        server_thread = threading.Thread(target=start_server, daemon=True)
        server_thread.start()
        print(f"DEBUG: Local Svelte SPA web server started at http://127.0.0.1:{port}", file=sys.stderr, flush=True)

        # -------------------------------------------------------------------------
        # Pre-load custom font as base64 if defined
        # -------------------------------------------------------------------------
        font_file = style.get("customFontFile", "").strip()
        font_base64 = ""
        if font_file:
            font_path = os.path.join(script_dir, "fonts", font_file)
            if os.path.isfile(font_path):
                try:
                    with open(font_path, "rb") as font_f:
                        font_base64 = base64.b64encode(font_f.read()).decode("utf-8")
                    print(f"DEBUG: Loaded custom font '{font_file}' as base64 ({len(font_base64)} chars)", file=sys.stderr, flush=True)
                except Exception as e:
                    print(f"DEBUG ERROR: Failed to encode custom font: {e}", file=sys.stderr, flush=True)

        # -------------------------------------------------------------------------
        # Launch Playwright (Chromium) and navigate to the render page
        # -------------------------------------------------------------------------
        with sync_playwright() as p:
            print("DEBUG: Spawning headless Chromium...", file=sys.stderr, flush=True)
            browser = p.chromium.launch(
                headless=True,
                args=[
                    "--hide-scrollbars",
                    "--mute-audio",
                    "--disable-web-security"
                ]
            )

            # Viewport size matches video size perfectly for pixel accuracy
            context = browser.new_context(
                viewport={"width": width, "height": height},
                device_scale_factor=1.0
            )
            page = context.new_page()

            # Load Svelte render page
            render_url = f"http://127.0.0.1:{port}/render"
            print(f"DEBUG: Navigating page to: {render_url}", file=sys.stderr, flush=True)
            page.goto(render_url)

            # Wait for Svelte app initialization
            page.wait_for_function("window.renderPageReady === true")
            print("DEBUG: Svelte render page successfully loaded and initialized.", file=sys.stderr, flush=True)

            # -------------------------------------------------------------------------
            # Prepare FFmpeg process to receive transparent PNG stream
            # -------------------------------------------------------------------------
            # -f image2pipe, -vcodec png, -i - receives our frames on stdin
            # -filter_complex "[0:v][1:v]overlay=0:0" overlays them on the video
            cmd = [
                "ffmpeg", "-y",
                "-i", input_path,
                "-f", "image2pipe",
                "-framerate", f"{fps:.6f}",
                "-vcodec", "png",
                "-i", "-",
                "-filter_complex", "[0:v][1:v]overlay=0:0",
                "-c:v", "libx264",
                "-pix_fmt", "yuv420p",
                "-c:a", "aac",
                "-map", "0:a?",
                "-progress", "pipe:1",
                output_path
            ]

            print(f"DEBUG FFmpeg Command: {' '.join(cmd)}", file=sys.stderr, flush=True)

            # Route FFmpeg stdout & progress to DEVNULL because we read stderr
            process = subprocess.Popen(
                cmd,
                stdin=subprocess.PIPE,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL
            )

            # -------------------------------------------------------------------------
            # -------------------------------------------------------------------------
            # Frame-by-frame screenshot rendering loop
            # -------------------------------------------------------------------------
            prev_sub_id = None
            empty_frame_bytes = None
            active_frame_bytes = None

            # Determine if subtitle uses a continuous animation (needs per-frame capture)
            animation_type = style.get("animationType", "none")
            is_animated = animation_type not in ("none", "", "pop-up", "scale-in")
            frame_interval_ms = round(1000 / fps)  # time between frames in ms

            print(f"DEBUG: Animation type: '{animation_type}', is_animated (per-frame): {is_animated}", file=sys.stderr, flush=True)
            print(f"DEBUG: Starting frame loop for {total_frames} frames...", file=sys.stderr, flush=True)

            for frame_idx in range(total_frames):
                t = frame_idx / fps

                # Find active subtitle segment at current timestamp
                active_sub = None
                for sub in subtitles:
                    if t >= sub["start"] and t <= sub["end"]:
                        active_sub = sub
                        break

                if active_sub is None:
                    # No active subtitle in this frame
                    if prev_sub_id is not None or empty_frame_bytes is None:
                        # Clear subtitle page state
                        page.evaluate("window.setRenderState({ content: '' })")
                        # Capture empty frame (fully transparent PNG) once
                        empty_frame_bytes = page.screenshot(type="png", omit_background=True)
                        prev_sub_id = None

                    frame_bytes = empty_frame_bytes
                else:
                    # Active subtitle found
                    sub_id = (active_sub["start"], active_sub["end"], active_sub["content"])

                    if sub_id != prev_sub_id or active_frame_bytes is None or is_animated:
                        # Subtitle content or segment changed — update state and screenshot
                        # Pre-scale fontSize for the video resolution (editor px → video px)
                        scaled_style = dict(style)
                        scaled_style["fontSize"] = style.get("fontSize", 28) * scale_factor

                        render_state = {
                            "content": active_sub["content"],
                            "subX": pos.get("subX", 50),
                            "subY": pos.get("subY", 85),
                            "subWidth": pos.get("subWidth", 70),
                            "scaleFactor": scale_factor,
                            "style": scaled_style
                        }

                        # Inject custom font base64 binary on initial active frame
                        if font_base64 and prev_sub_id is None:
                            render_state["customFontBase64"] = font_base64

                        # Set state in page Svelte component
                        page.evaluate("state => window.setRenderState(state)", render_state)

                        # For animated subtitles, wait one frame interval for CSS animation to advance
                        if is_animated:
                            page.wait_for_timeout(frame_interval_ms)

                        # Capture transparent PNG subtitle block frame
                        active_frame_bytes = page.screenshot(type="png", omit_background=True)
                        prev_sub_id = sub_id

                    frame_bytes = active_frame_bytes

                # Write frame to FFmpeg stdin
                process.stdin.write(frame_bytes)

                # Output render progress percent to stderr (forwarded to Svelte frontend)
                if frame_idx % 5 == 0 or frame_idx == total_frames - 1:
                    pct = (frame_idx / total_frames) * 100.0
                    print(f"PROGRESS:{pct:.1f}", file=sys.stderr, flush=True)

            print("DEBUG: All frames processed, finalizing video file...", file=sys.stderr, flush=True)

            # Close stdin to signal end of stream to FFmpeg
            process.stdin.close()
            process.wait()

            browser.close()

        if process.returncode == 0:
            print("PROGRESS:100.0", file=sys.stderr, flush=True)
            print(json.dumps({"status": "ok", "message": "Render completed successfully"}))
        else:
            print(json.dumps({
                "status": "error",
                "message": f"ffmpeg failed with exit code {process.returncode}"
            }))

    except Exception as e:
        import traceback
        traceback.print_exc(file=sys.stderr)
        print(json.dumps({"status": "error", "message": str(e)}))

    finally:
        if server:
            try:
                server.shutdown()
                server.server_close()
                print("DEBUG: Web server successfully stopped.", file=sys.stderr, flush=True)
            except Exception:
                pass


if __name__ == "__main__":
    run()