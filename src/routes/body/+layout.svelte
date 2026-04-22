<script lang="ts">
    import { page } from "$app/state";
    import BodyHeader from "$lib/BodyHeader.svelte";
    import { bodyHrefs } from "$lib/body";

    let { children } = $props();

    const date = $derived(
        page.url.searchParams.get("from") ?? page.params.date ?? "",
    );

    const activeTab = $derived(
        page.route.id?.endsWith("/history")
            ? "history"
            : page.route.id?.endsWith("/graph")
              ? "graph"
              : page.route.id?.endsWith("/prs")
                ? "prs"
                : "log",
    );

    const hrefs = $derived(bodyHrefs(date));
</script>

<div class="page">
    <BodyHeader
        feedHref={hrefs.feedHref}
        logHref={hrefs.logHref}
        historyHref={hrefs.historyHref}
        graphHref={hrefs.graphHref}
        prsHref={hrefs.prsHref}
        {activeTab}
        {date}
    />
    {@render children()}
</div>
