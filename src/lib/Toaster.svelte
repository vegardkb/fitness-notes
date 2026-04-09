<script lang="ts">
    import { toast } from "$lib/toast";
</script>

{#if $toast}
    <div
        class="toast"
        class:toast--error={$toast.level === "error"}
        role="alert"
        onclick={() => toast.dismiss()}
    >
        {$toast.message}
    </div>
{/if}
