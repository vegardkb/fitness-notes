<script lang="ts">
    let props = $props();
</script>

<div class="history-header">
    <a class="back-btn" href={props.feedHref}>←</a>
    <h1>Body {props.date}</h1>
    <div class="header-tabs">
        <a
            class="header-tab"
            class:header-tab--active={props.activeTab === "sets"}
            href={props.setsHref}
            aria-label="Sets"
        >
            <svg
                width="18"
                height="18"
                viewBox="0 0 20 20"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
            >
                <line x1="4" y1="6" x2="16" y2="6" />
                <line x1="4" y1="10" x2="16" y2="10" />
                <line x1="4" y1="14" x2="16" y2="14" />
            </svg>
        </a>
        <a
            class="header-tab"
            class:header-tab--active={props.activeTab === "history"}
            href={props.historyHref}
            aria-label="History"
        >
            <svg
                width="18"
                height="18"
                viewBox="0 0 20 20"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <circle cx="10" cy="10" r="8" />
                <polyline points="10,6 10,10 13,12" />
            </svg>
        </a>
        <a
            class="header-tab"
            class:header-tab--active={props.activeTab === "graph"}
            href={props.graphHref}
            aria-label="Graph"
        >
            <svg
                width="18"
                height="18"
                viewBox="0 0 20 20"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <polyline points="2,15 7,9 11,12 18,4" />
                <line x1="2" y1="18" x2="18" y2="18" />
            </svg>
        </a>
        <a
            class="header-tab"
            class:header-tab--active={props.activeTab === "prs"}
            href={props.prsHref}
            aria-label="PRs"
        >
            <svg
                width="18"
                height="18"
                viewBox="0 0 20 20"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M6 2h8v7a4 4 0 0 1-8 0V2z" />
                <path d="M6 4H3a2 2 0 0 0 0 4h3" />
                <path d="M14 4h3a2 2 0 0 0 0 4h-3" />
                <line x1="10" y1="13" x2="10" y2="16" />
                <line x1="7" y1="16" x2="13" y2="16" />
                <line x1="8" y1="18" x2="12" y2="18" />
                <line x1="8" y1="16" x2="8" y2="18" />
                <line x1="12" y1="16" x2="12" y2="18" />
            </svg>
        </a>
    </div>
</div>
