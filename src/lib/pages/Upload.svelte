<script lang="ts">
    import { goto } from "$app/navigation";
    import { videoPath, wordPerFrame } from "$lib/store";

    let videoFile = $state<File | null>(null);
    let localWordPerFrame = $state<number>(0);

    function handleFileUpload(event: Event) {
        const input = event.target as HTMLInputElement;
        const file = input.files?.[0];
        if (file) {
            videoFile = file;
            videoPath.set((file as any).path);
        }
    }

    function handelWordsPerFrame(event: Event) {
        const input = event.target as HTMLInputElement;
        localWordPerFrame = Number(input.value);
        wordPerFrame.set(localWordPerFrame);
    }

    async function generate() {
        if (!videoFile || localWordPerFrame <= 0) {
            alert("Upload a video file and enter words per frame");
            return;
        }
        await goto("/loading");
    }
</script>

<main class="cont">
    <h1>Welcome to OpenSub</h1>
    <input type="file" accept="video/*" onchange={handleFileUpload} />
    <input type="number" min="1" oninput={handelWordsPerFrame} />
    <button onclick={generate}>Generate</button>
</main>
