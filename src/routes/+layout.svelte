<script lang="ts">
    import "../app.css";
    import { onMount } from "svelte";
    let { children } = $props();
    onMount(async () => {
        //const settings = await invoke<{ dark_mode: boolean }>(
        //    "get_settings_frontend",
        //);
        const settings = { dark_mode: false };
        document.documentElement.dataset.theme = settings.dark_mode
            ? "dark"
            : "light";
    });
</script>

{@render children()}
