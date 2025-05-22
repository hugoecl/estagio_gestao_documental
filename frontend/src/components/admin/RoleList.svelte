<script lang="ts">
    import { onMount, tick } from "svelte";
    import Table from "@components/common/Table.svelte";
    import RoleFormModal from "./RoleFormModal.svelte";
    import type { Role, RoleWithInterferingRoles } from "@lib/types/roles";
    import type { TableColumn } from "@lib/types/table";
    import { getRoles, getRoleWithInterferingRoles } from "@api/roles-api";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    interface FormattedRole extends Role {
        created_at_formatted: string;
        updated_at_formatted: string;
        is_admin_translated: string;
        has_interfering_roles?: boolean;
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
        };
    }

    const columns: TableColumn[] = [
        { header: "ID", field: "id" },
        { header: "Nome", field: "name" },
        { header: "Descrição", field: "description" },
        { header: "É Admin?", field: "is_admin_translated" },
        { header: "Criado Em", field: "created_at_formatted" },
        { header: "Atualizado Em", field: "updated_at_formatted" },
    ];

    onMount(async () => {
        await loadRoles();
    });

    async function loadRoles() {
        isLoading = true;
        try {
            const rolesList = await getRoles();
            roles = {}; // Reset roles
            // Convert array to record indexed by role id
            rolesList.forEach((role) => {
                roles[role.id.toString()] = formatRoleForDisplay(role);
            });
        } catch (error: any) {
            showAlert(
                `Erro ao carregar funções: ${error.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            console.error("Failed to load roles:", error);
        } finally {
            isLoading = false;
        }
    }

    async function handleRowClick(row: FormattedRole) {
        try {
            // Ensure row.id is a number
            const roleId = typeof row.id === 'string' ? parseInt(row.id, 10) : row.id;
            
            if (!roleId || isNaN(roleId)) {
                throw new Error("Invalid role ID");
            }
            
            const roleWithInterfering = await getRoleWithInterferingRoles(roleId);
            console.log("API response:", roleWithInterfering);
            
            // The API returns the role data directly, not nested in a 'role' property
            if (!roleWithInterfering || typeof roleWithInterfering.id === 'undefined') {
                console.error("Invalid role data from API:", roleWithInterfering);
                // Fall back to using the row data directly
                selectedRole = { ...row };
            } else {
                selectedRole = {
                    ...roleWithInterfering,
                };
                console.log("Selected role for modal:", selectedRole);
            }
        } catch (error) {
            console.error("Error loading role with interfering roles:", error);
            // Ensure we set the full row object with its ID
            selectedRole = { ...row };
            console.log("Fallback selected role:", selectedRole);
        }
        
        await tick(); // Ensure DOM updates
        if (roleModalRef) {
            roleModalRef.showModal();
        } else {
            console.error("Modal reference not found after tick in handleRowClick.");
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
    searchFields={["name", "description", "is_admin_translated"]}
    onRowClick={(id, row) => handleRowClick(row)}
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
