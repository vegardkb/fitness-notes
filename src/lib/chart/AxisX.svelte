<script lang="ts">
    import { getContext } from "svelte";
    import type { Readable } from "svelte/store";
    import type { ScaleTime } from "d3-scale";

    interface Props {
        ticks?: number;
        format?: (v: Date) => string;
    }

    const defaultFormat = (v: Date) =>
        v.toLocaleDateString(undefined, { month: "short", day: "numeric" });

    let { ticks = 5, format = defaultFormat }: Props = $props();

    const { height, xScale } = getContext<{
        height: Readable<number>;
        xScale: Readable<ScaleTime<number, number>>;
    }>("LayerCake");

    const tickValues = $derived($xScale.ticks(ticks));
</script>

<g class="axis-x" transform="translate(0, {$height})">
    {#each tickValues as v}
        <g class="tick" transform="translate({$xScale(v)}, 0)">
            <text y="16" text-anchor="middle">{format(v)}</text>
        </g>
    {/each}
</g>

<style>
    .tick text {
        fill: var(--text-muted);
        font-size: 11px;
        font-family: inherit;
    }
</style>
