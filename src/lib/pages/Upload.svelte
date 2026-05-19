<script lang="ts">
    import { goto } from "$app/navigation";
    import { videoPath } from "../store.js";

    //uploading file
    let videoFile = $state<File | null>(null);
    let wordPerFrame = $state<number>(3);
    // let videUrl = $state<string>("");

    function handleFileUpload(event: Event) {
        const input = event.target as HTMLInputElement;
        const file = input.files?.[0];
        if (file) {
            videoFile = file;
            //setting video path to store.js
            videoPath.set(URL.createObjectURL(file));
        }
    }
    function handelWordsPerFrame(event: Event) {
        const input = event.target as HTMLInputElement;
        wordPerFrame = Number(input.value);
    }
    async function generate() {
        if (!videoFile || !wordPerFrame) {
            alert("Upload video file or write the no of words per frame");
            return;
        }
        await goto("/loading");
    }
</script>

<main class="cont">
    <h1>Welcome to OpenSub</h1>
    <!-- File Uploading -->
    <input type="file" accept="video/*" onchange={handleFileUpload} />
    <input type="number" onchange={handelWordsPerFrame} />
    <button onclick={generate}>Generate</button>
</main>
