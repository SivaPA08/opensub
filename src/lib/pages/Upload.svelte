<script lang="ts">
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
</script>

<main class="cont">
    <h1>Welcome to OpenSub</h1>
    <!-- File Uploading -->
    <input type="file" accept="video/*" onchange={handleFileUpload} />
    <input type="number" onchange={handelWordsPerFrame} />
</main>
