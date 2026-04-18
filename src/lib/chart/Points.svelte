<script lang="ts" generics="T">
    import { getContext } from "svelte";
    import type { Readable } from "svelte/store";

    interface Props {
        fill?: string;
        r?: number;
        onpointclick?: (d: T, e: MouseEvent) => void;
        onpointenter?: (d: T, e: PointerEvent) => void;
        onpointleave?: () => void;
    }

    let {
        fill = "var(--accent)",
        r = 3,
        onpointclick,
        onpointenter,
        onpointleave,
    }: Props = $props();

    const { data, xGet, yGet } = getContext<{
        data: Readable<T[]>;
        xGet: Readable<(d: T) => number>;
        yGet: Readable<(d: T) => number>;
    }>("LayerCake");
</script>

{#each $data as d}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <circle
        cx={$xGet(d)}
        cy={$yGet(d)}
        {r}
        {fill}
        style:cursor={onpointclick ? "pointer" : "default"}
        onclick={onpointclick ? (e) => onpointclick(d, e) : undefined}
        onpointerenter={onpointenter ? (e) => onpointenter(d, e) : undefined}
        onpointerleave={onpointleave}
    />
{/each}
