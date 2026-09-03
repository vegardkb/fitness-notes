<script lang="ts">
    import { onMount, tick } from "svelte";
    import { invoke } from "$lib/tauri";
    import { goto } from "$app/navigation";
    import { PlusIcon } from "lucide-svelte";

    import TemplateCard from "$lib/TemplateCard.svelte";
    import type { NamedId } from "$lib/exercise";

    const date = new URLSearchParams(location.search).get("date");
    const targetTemplate = new URLSearchParams(location.search).get(
        "fromTemplate",
    );

    let templates = $state<NamedId[]>([]);

    let scrolled = false;

    function handleTemplateLoaded(id: number) {
        if (scrolled) return;
        if (id === Number(targetTemplate)) {
            scrolled = true;
            tick().then(() => scrollToTemplate(id));
        }
    }

    onMount(async () => {
        await listTemplates();
    });

    function scrollToTemplate(id: number) {
        document
            .getElementById(`template-${id}`)
            ?.scrollIntoView({ behavior: "instant", block: "center" });
    }

    function listTemplates(): Promise<void> {
        return invoke<NamedId[]>("list_templates", {}).then((result) => {
            templates = result.sort((a, b) => a.name.localeCompare(b.name));
        });
    }

    async function createTemplate() {
        const name = "New Template";
        await invoke("create_template", { name });
        listTemplates();
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
            <TemplateCard
                {template}
                {listTemplates}
                onLoaded={handleTemplateLoaded}
            />
        {/each}
    </div>
</div>
