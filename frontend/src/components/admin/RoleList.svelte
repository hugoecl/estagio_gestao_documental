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

    let roles = $state<Record<string, Role>>({});
    let isLoading = $state(true);
    let error = $state<string | null>(null);

    let roleModalRef: HTMLDialogElement;
    let selectedRole = $state<Role | null>(null); // State for selected role

    const columns: TableColumn[] = [
        { header: "ID", field: "id" },
        { header: "Nome", field: "name" },
        { header: "Descrição", field: "description" },
        { header: "É Admin?", field: "is_admin" },
        { header: "Criado Em", field: "created_at" },
        { header: "Atualizado Em", field: "updated_at" },
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
            const rolesRecord: Record<string, Role> = {};
            rolesArray.forEach((role) => {
                rolesRecord[role.id.toString()] = role;
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
        roles[newRole.id.toString()] = newRole;
        roles = { ...roles };
    }

    function handleRoleUpdated(updatedRole: Role) {
        if (roles[updatedRole.id.toString()]) {
            roles[updatedRole.id.toString()] = updatedRole;
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
    searchFields={["name", "description"]}
    onRowClick={handleRowClick}
    rowClassName="hover:bg-base-300 cursor-pointer"
>
    <svelte:fragment slot="column-is_admin" let:row>
        {#if row.is_admin}
            <span class="badge badge-primary badge-sm">Sim</span>
        {:else}
            <span class="badge badge-ghost badge-sm">Não</span>
        {/if}
    </svelte:fragment>
    <svelte:fragment slot="column-created_at" let:row>
        {new Date(row.created_at).toLocaleString("pt-PT")}
    </svelte:fragment>
    <svelte:fragment slot="column-updated_at" let:row>
        {new Date(row.updated_at).toLocaleString("pt-PT")}
    </svelte:fragment>
</Table>

<!-- Pass role one-way and provide onClose callback -->
<RoleFormModal
    bind:modalRef={roleModalRef}
    role={selectedRole}
    onClose={handleModalClose}
    onRoleCreated={handleRoleCreated}
    onRoleUpdated={handleRoleUpdated}
    onRoleDeleted={handleRoleDeleted}
/>
