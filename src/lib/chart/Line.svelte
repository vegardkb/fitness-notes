<script lang="ts">
    import { getContext } from "svelte";
    import type { Readable } from "svelte/store";

    interface Props {
        stroke?: string;
        strokeWidth?: number;
    }

    let { stroke = "var(--accent)", strokeWidth = 2 }: Props = $props();

    const { data, xGet, yGet } = getContext<{
        data: Readable<unknown[]>;
        xGet: Readable<(d: unknown) => number>;
        yGet: Readable<(d: unknown) => number>;
    }>("LayerCake");

    const path = $derived.by(() => {
        const pts = $data;
        if (pts.length === 0) return "";
        const x = $xGet;
        const y = $yGet;
        return pts
            .map((d, i) => `${i === 0 ? "M" : "L"}${x(d)},${y(d)}`)
            .join("");
    });
</script>

<path
    d={path}
    fill="none"
    {stroke}
    stroke-width={strokeWidth}
    stroke-linejoin="round"
    stroke-linecap="round"
/>
