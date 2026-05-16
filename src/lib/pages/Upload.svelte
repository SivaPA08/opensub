<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    let name = $state("");

    //uploading file
    let videoFile = $state<File | null>(null);
    let wordPerFrame = $state<number>(3);
    let videUrl = $state<string>("");

    function handleFileUpload(event: Event) {
        const input = event.target as HTMLInputElement;
        const file = input.files?.[0];
        if (file) {
            videoFile = file;
            videUrl = URL.createObjectURL(file);
        }
    }
    function handelWordsPerFrame(event: Event) {
        const input = event.target as HTMLInputElement;
        wordPerFrame = Number(input.value);
    }
</script>

<main class="cont">
    <h1>Welcome to Opensub</h1>
</main>
