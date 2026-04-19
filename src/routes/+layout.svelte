<script lang="ts">
    import "../app.css";
    import "@fontsource/dm-sans/400.css";
    import "@fontsource/dm-sans/600.css";
    import "@fontsource/source-serif-4/400.css";
    import "@fontsource/source-serif-4/600.css";
    import { onMount } from "svelte";
    import { onNavigate } from "$app/navigation";
    import Toaster from "$lib/Toaster.svelte";
    import { invoke } from "$lib/tauri";
    import type { Settings } from "$lib/settings";

    let { children } = $props();

    onMount(async () => {
        const settings = await invoke<Settings>("get_settings");
        document.documentElement.classList.toggle("dark", settings.dark_mode);
    });

    onNavigate((navigation) => {
        if (!document.startViewTransition) return;
        return new Promise((resolve) => {
            document.startViewTransition(async () => {
                resolve();
                await navigation.complete;
            });
        });
    });
</script>

{@render children()}
<Toaster />
