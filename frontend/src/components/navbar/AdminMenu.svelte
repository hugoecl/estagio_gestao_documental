<script lang="ts">
    import { isAdmin } from "@stores/auth-store";
</script>

{#if $isAdmin}
    <li>
        <a href="/admin/vacations/">
            <i class="fa-solid fa-user-shield"></i>
            Gestão de Férias (Admin)
        </a>
    </li>
{/if}
