<script lang="ts">
    import "../app.css";
    import { onMount } from "svelte";
    import { onNavigate } from "$app/navigation";
    import Toaster from "$lib/Toaster.svelte";

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
