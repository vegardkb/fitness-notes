<script lang="ts">
    import { getContext } from "svelte";
    import type { Readable } from "svelte/store";
    import type { ScaleLinear } from "d3-scale";

    interface Props {
        ticks?: number;
        format?: (v: number) => string;
    }

    let {
        ticks = 4,
        format = (v: number) => String(v),
    }: Props = $props();

    const { width, yScale } = getContext<{
        width: Readable<number>;
        yScale: Readable<ScaleLinear<number, number>>;
    }>("LayerCake");

    const tickValues = $derived($yScale.ticks(ticks));
</script>

<g class="axis-y">
    {#each tickValues as v}
        <g class="tick" transform="translate(0, {$yScale(v)})">
            <line x1="0" x2={$width} stroke-dasharray="3 3" />
            <text x="-6" dy="0.32em" text-anchor="end">{format(v)}</text>
        </g>
    {/each}
</g>

<style>
    .tick line {
        stroke: var(--border);
    }
    .tick text {
        fill: var(--text-muted);
        font-size: 11px;
        font-family: inherit;
    }
</style>
