<script lang="ts">
    import DotsIcon from "$lib/icons/DotsIcon.svelte";

    type Item = { id: number; name: string };
    type MenuAction = {
        label: string;
        danger?: boolean;
        action: (id: number) => void;
    };

    let {
        items,
        placeholder = "New item",
        creating = $bindable(false),
        onselect,
        oncreate,
        onrename,
        extraActions = [],
    }: {
        items: Item[];
        placeholder?: string;
        creating: boolean;
        onselect: (item: Item) => void;
        oncreate: (name: string) => void;
        onrename?: (id: number, name: string) => void;
        extraActions?: MenuAction[];
    } = $props();

    let menuOpenId = $state<number | null>(null);
    let renamingId = $state<number | null>(null);
    let inputValue = $state("");

    $effect(() => {
        if (creating) inputValue = "";
    });

    function openMenu(id: number) {
        menuOpenId = id;
    }

    function closeMenu() {
        menuOpenId = null;
    }

    function startRename(id: number, currentName: string) {
        closeMenu();
        renamingId = id;
        inputValue = currentName;
    }

    function confirmRename(id: number) {
        const trimmed = inputValue.trim();
        if (trimmed && onrename) onrename(id, trimmed);
        renamingId = null;
        inputValue = "";
    }

    function cancelRename() {
        renamingId = null;
        inputValue = "";
    }

    function confirmCreate() {
        const trimmed = inputValue.trim();
        if (trimmed) oncreate(trimmed);
        creating = false;
        inputValue = "";
    }

    function cancelCreate() {
        creating = false;
        inputValue = "";
    }
</script>

{#if menuOpenId !== null || creating}
    <div
        class="menu-backdrop"
        onclick={() => {
            closeMenu();
            cancelCreate();
        }}
    ></div>
{/if}

<div class="list">
    {#each items as item (item.id)}
        <div class="list-item-wrap">
            {#if renamingId === item.id}
                <div class="list-item list-item--input">
                    <input
                        class="inline-input"
                        type="text"
                        bind:value={inputValue}
                        onkeydown={(e) => {
                            if (e.key === "Enter") confirmRename(item.id);
                            if (e.key === "Escape") cancelRename();
                        }}
                        autofocus
                    />
                    <button
                        class="add-btn-inline"
                        onpointerdown={(e) => e.stopPropagation()}
                        onclick={() => confirmRename(item.id)}>Save</button
                    >
                </div>
            {:else}
                <div
                    class="list-item"
                    role="button"
                    tabindex="0"
                    onclick={() => onselect(item)}
                    onkeydown={(e) => e.key === "Enter" && onselect(item)}
                >
                    <span>{item.name}</span>
                    <button
                        class="icon-btn"
                        onpointerdown={(e) => e.stopPropagation()}
                        onclick={(e) => {
                            e.stopPropagation();
                            openMenu(item.id);
                        }}
                    >
                        <DotsIcon />
                    </button>
                </div>
            {/if}

            {#if menuOpenId === item.id}
                <div class="item-menu">
                    {#if onrename}
                        <button
                            class="item-menu-btn"
                            onclick={() => startRename(item.id, item.name)}
                            >Rename</button
                        >
                    {/if}
                    {#each extraActions as menuAction}
                        <button
                            class="item-menu-btn"
                            class:item-menu-btn--danger={menuAction.danger}
                            onclick={() => {
                                closeMenu();
                                menuAction.action(item.id);
                            }}>{menuAction.label}</button
                        >
                    {/each}
                </div>
            {/if}
        </div>
    {/each}

    {#if creating}
        <div class="list-item list-item--input">
            <input
                class="inline-input"
                type="text"
                bind:value={inputValue}
                {placeholder}
                onkeydown={(e) => {
                    if (e.key === "Enter") confirmCreate();
                    if (e.key === "Escape") cancelCreate();
                }}
                autofocus
            />
            <button
                class="add-btn-inline"
                onpointerdown={(e) => e.stopPropagation()}
                onclick={confirmCreate}>Add</button
            >
        </div>
    {/if}
</div>
