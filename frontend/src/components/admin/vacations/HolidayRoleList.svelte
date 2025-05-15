<script lang="ts">
    import { onMount } from "svelte";
    import Table from "@components/common/Table.svelte";
    import type { Role } from "@lib/types/roles";
    import type { TableColumn } from "@lib/types/table";
    import { getHolidayRoles } from "@api/admin-vacation-api"; // Ensure this API function exists
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    // Type for display, including formatted fields if any (though Role itself is fine for now)
    interface DisplayRole extends Role {
        // Add any client-side formatted fields if needed, e.g., created_at_formatted
        // For now, we can directly use Role fields if formatting is simple or done in Table component
    }

    let holidayRoles = $state<Record<string, DisplayRole>>({});
    let isLoading = $state(true);
    let error = $state<string | null>(null);

    const columns: TableColumn[] = [
        { header: "ID", field: "id" },
        { header: "Nome da Função", field: "name" },
        { header: "Descrição", field: "description" },
        // Add more columns if relevant, e.g., number of users in this role
    ];

    onMount(async () => {
        isLoading = true;
        error = null;
        try {
            const rolesArray = await getHolidayRoles();
            const rolesRecord: Record<string, DisplayRole> = {};
            rolesArray.forEach((role) => {
                // Perform any necessary formatting here if DisplayRole differs from Role
                rolesRecord[role.id.toString()] = {
                    ...role,
                    // Example formatting:
                    // created_at_formatted: new Date(role.created_at).toLocaleDateString("pt-PT"),
                };
            });
            holidayRoles = rolesRecord;
        } catch (e: any) {
            console.error("Error fetching holiday roles:", e);
            error = `Erro ao carregar funções de férias: ${e.message}`;
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoading = false;
        }
    });

    function handleRowClick(id: string, role: DisplayRole) {
        // Navigate to the page for managing requests for this role
        // This will be like /admin/vacations/roles/{role.id}/requests/
        if (typeof window !== "undefined") {
            window.location.href = `/admin/vacations/roles/${id}/requests/`;
        }
    }
</script>

{#if error}
    <div class="alert alert-error my-4">{error}</div>
{/if}

<Table
    data={holidayRoles}
    {columns}
    loading={isLoading}
    emptyMessage="Nenhuma função de férias encontrada. Edite as funções existentes para marcá-las como funções de férias."
    searchEmptyMessage="Nenhuma função de férias encontrada para a sua pesquisa."
    keyField="id"
    searchFields={["name", "description"]}
    onRowClick={handleRowClick}
    rowClassName="hover:bg-base-300 cursor-pointer"
/>
