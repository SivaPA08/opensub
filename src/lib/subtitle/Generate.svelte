<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { subtitle, clearUndoHistory, type Subtitle } from "../store";
    async function generateSubtitle(videoUrl: string, count: number) {
        const sub = await invoke<any>("getvideo", {
            filename: videoUrl,
            maxWords: count,
        });
        if (sub && sub.status === "ok" && Array.isArray(sub.message)) {
            const mapped = sub.message.map((s: any) => ({
                start: s.start,
                end: s.end,
                content: s.text || s.content || ""
            }));
            clearUndoHistory();
            subtitle.set(mapped);
        }
    }
</script>
