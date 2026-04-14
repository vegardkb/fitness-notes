<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import SelectList from "$lib/SelectList.svelte";
    import { toast } from "$lib/toast";

    import type { Template } from "$lib/exercise";

    const date = $derived(page.params.date ?? "");

    let templates = $state<Template[]>([]);

    const templateActions = [
        { label: "Delete", danger: true, action: deleteTemplate },
    ];
    onMount(async () => {
        templates = await invoke("list_templates");
    });

    async function selectTemplate(template: Template) {
        try {
            await invoke("apply_template", {
                templateId: template.id,
                date: date,
            });
            goto(date ? `/?date=${date}` : "/");
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    async function renameTemplate(id: number, name: string) {
        try {
            await invoke("rename_template", { id, name });
            templates = await invoke("list_templates");
        } catch (e) {
            toast.show(String(e), "error");
        }
    }

    async function deleteTemplate(id: number) {
        try {
            await invoke("delete_template", { templateId: id });
            templates = await invoke("list_templates");
        } catch (e) {
            toast.show(String(e), "error");
        }
    }
</script>

<div class="page" role="presentation">
    <div class="header">
        <button
            class="back-btn"
            onclick={() => goto(date ? `/?date=${date}` : "/")}>←</button
        >
        <h1>Select template</h1>
    </div>
    <SelectList
        items={templates}
        creating={false}
        placeholder="New template"
        onselect={selectTemplate}
        onrename={renameTemplate}
        oncreate={() => {}}
        extraActions={templateActions}
    />
</div>
