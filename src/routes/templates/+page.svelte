<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "$lib/tauri";
    import { goto } from "$app/navigation";
    import { Calendar, PlusIcon, Settings } from "lucide-svelte";

    import TemplateCard from "$lib/TemplateCard.svelte";
    import type { NamedId } from "$lib/exercise";

    const date = new URLSearchParams(location.search).get("date");

    let loading = $state(false);
    let templates = $state<NamedId[]>([]);

    onMount(async () => {
        await listTemplates();
    });

    async function listTemplates() {
        loading = true;
        invoke<NamedId[]>("list_templates", {}).then((result) => {
            templates = result.sort((a, b) => a.name.localeCompare(b.name));
            loading = false;
        });
    }

    async function createTemplate() {
        const name = "New Template";
        await invoke("create_template", { name });
        await listTemplates();
    }
</script>

<div class="page">
    <div class="app-header-wrap">
        <header class="app-header">
            <button class="back-btn" onclick={() => goto(`templates/${date}`)}
                >←</button
            >
            <span class="app-title">Templates</span>

            <div class="app-header-icons">
                <button
                    class="back-btn"
                    aria-label="Add Template"
                    onclick={() => createTemplate()}
                >
                    <PlusIcon size={20} strokeWidth={1.5} />
                </button>
            </div>
        </header>
    </div>

    <div class="feed">
        {#each templates as template (template.id)}
            <TemplateCard {template} {listTemplates} />
        {/each}
    </div>
</div>
