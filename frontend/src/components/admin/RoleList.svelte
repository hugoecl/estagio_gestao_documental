<script lang="ts">
    import { onMount, tick } from "svelte";
    import Table from "@components/common/Table.svelte";
    import RoleFormModal from "./RoleFormModal.svelte";
    import type { Role } from "@lib/types/roles";
    import type { TableColumn } from "@lib/types/table";
    import { getRoles } from "@api/roles-api";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    interface FormattedRole extends Role {
        created_at_formatted: string;
        updated_at_formatted: string;
        is_admin_translated: string;
        is_holiday_role_translated: string; // New field
    }
    let roles = $state<Record<string, FormattedRole>>({});
    let isLoading = $state(true);
    let error = $state<string | null>(null);

    let roleModalRef: HTMLDialogElement;
    let selectedRole = $state<Role | null>(null); // State for selected role

    // Helper function to format a Role object for display
    function formatRoleForDisplay(role: Role): FormattedRole {
        return {
            ...role,
            created_at_formatted: new Date(role.created_at).toLocaleString("pt-PT", { dateStyle: "short", timeStyle: "medium" }),
            updated_at_formatted: new Date(role.updated_at).toLocaleString("pt-PT", { dateStyle: "short", timeStyle: "medium" }),
            is_admin_translated: role.is_admin ? "Sim" : "Não",
            is_holiday_role_translated: role.is_holiday_role ? "Sim" : "Não",
        };
    }

    const columns: TableColumn[] = [
        { header: "ID", field: "id" },
        { header: "Nome", field: "name" },
        { header: "Descrição", field: "description" },
        { header: "É Admin?", field: "is_admin_translated" },
        { header: "Função de Férias?", field: "is_holiday_role_translated" }, // New column
        { header: "Criado Em", field: "created_at_formatted" },
        { header: "Atualizado Em", field: "updated_at_formatted" },
    ];

    onMount(async () => {
        await loadRoles();
    });

    async function loadRoles() {
        // ... (loadRoles logic remains the same) ...
        isLoading = true;
        error = null;
        try {
            const rolesArray = await getRoles();
            const rolesRecord: Record<string, FormattedRole> = {};
            rolesArray.forEach((role) => {
                rolesRecord[role.id.toString()] = {
                    ...role,
                    created_at_formatted: new Date(role.created_at).toLocaleString("pt-PT", { dateStyle: "short", timeStyle: "medium" }),
                    updated_at_formatted: new Date(role.updated_at).toLocaleString("pt-PT", { dateStyle: "short", timeStyle: "medium" }),
                    is_admin_translated: role.is_admin ? "Sim" : "Não",
                    is_holiday_role_translated: role.is_holiday_role ? "Sim" : "Não", // Populate new field
                };
            });
            roles = rolesRecord;
        } catch (e: any) {
            error = `Erro ao carregar funções: ${e.message}`;
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoading = false;
        }
    }

    async function handleRowClick(id: string, row: Role) {
        if (row.id === 1) {
            showAlert(
                "A função Admin principal não pode ser editada por aqui.",
                AlertType.WARNING,
                AlertPosition.TOP,
            );
            return;
        }
        selectedRole = row; // Set the state
        await tick(); // Wait for DOM update (modal gets the new prop)
        if (roleModalRef) {
            roleModalRef.showModal(); // Show the modal
        } else {
            console.error(
                "Modal reference (roleModalRef) not found after tick in handleRowClick.",
            );
        }
    }

    async function openCreateModal() {
        selectedRole = null; // Set state for create mode
        await tick(); // Wait for DOM update (modal gets null prop)
        if (roleModalRef) {
            roleModalRef.showModal(); // Show the modal
        } else {
            console.error(
                "Modal reference (roleModalRef) not found after tick in openCreateModal.",
            );
        }
    }

    // Expose globally
    if (typeof window !== "undefined") {
        (window as any).openCreateRoleModal = openCreateModal;
    }

    // --- New Function to handle modal close ---
    function handleModalClose() {
        selectedRole = null; // Reset the selection when modal signals it's closed
    }
    // --- End New Function ---

    function handleRoleCreated(newRole: Role) {
        roles[newRole.id.toString()] = formatRoleForDisplay(newRole);
        roles = { ...roles };
    }

    function handleRoleUpdated(updatedRole: Role) {
        if (roles[updatedRole.id.toString()]) {
            roles[updatedRole.id.toString()] = formatRoleForDisplay(updatedRole);
            roles = { ...roles };
        }
    }

    function handleRoleDeleted(deletedId: number) {
        if (roles[deletedId.toString()]) {
            delete roles[deletedId.toString()];
            roles = { ...roles };
        }
    }
</script>

{#if error}
    <div class="alert alert-error">{error}</div>
{/if}

<Table
    data={roles}
    {columns}
    loading={isLoading}
    emptyMessage="Nenhuma função encontrada."
    keyField="id"
    searchFields={["name", "description", "is_admin_translated", "is_holiday_role_translated"]}
    onRowClick={handleRowClick}
    rowClassName="hover:bg-base-300 cursor-pointer"
></Table>

<!-- Pass role one-way and provide onClose callback -->
<RoleFormModal
    bind:modalRef={roleModalRef}
    role={selectedRole}
    onClose={handleModalClose}
    onRoleCreated={handleRoleCreated}
    onRoleUpdated={handleRoleUpdated}
    onRoleDeleted={handleRoleDeleted}
/>
