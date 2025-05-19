<script lang="ts">
    import { isAdmin } from "@stores/auth-store";
    import AdminLinks from "@components/welcome/AdminLinks.svelte";
</script>

{#if $isAdmin}
    <AdminLinks />
{/if}