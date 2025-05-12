<script lang="ts">
    import { onMount, tick } from "svelte"; // Import tick
    import Table from "@components/common/Table.svelte";
    import AdminEditUserModal from "./AdminEditUserModal.svelte"; // Import the renamed modal
    import type { UserWithRoles } from "@lib/types/user";
    import type { Role } from "@lib/types/roles";
    import type { TableColumn } from "@lib/types/table";
    import { getUsersWithRoles } from "@api/user-api";
    import { getRoles } from "@api/roles-api";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    let users = $state<Record<string, UserWithRoles>>({});
    let allRoles = $state<Role[]>([]);
    let isLoading = $state(true);
    let error = $state<string | null>(null);

    let editModalRef: HTMLDialogElement; // Keep as direct reference
    let selectedUser = $state<UserWithRoles | null>(null);

    const columns: TableColumn[] = [
        { header: "ID", field: "id" },
        { header: "Nome Utilizador", field: "username" },
        { header: "Email", field: "email" },
        { header: "Funções", field: "roles" }, // Added Roles column
    ];

    // --- Renamed Function to Load Data ---
    async function loadUsersAndRoles() {
        isLoading = true;
        error = null;
        try {
            // Fetch both users and roles
            const [usersArray, rolesArray] = await Promise.all([
                getUsersWithRoles(),
                getRoles(),
            ]);

            const usersRecord: Record<string, UserWithRoles> = {};
            usersArray.forEach((user) => {
                usersRecord[user.id.toString()] = user;
            });
            users = usersRecord; // Update users state
            allRoles = rolesArray; // Update roles state
        } catch (e: any) {
            error = `Erro ao carregar utilizadores ou funções: ${e.message}`;
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoading = false;
        }
    }

    // --- End Renamed Function ---

    onMount(async () => {
        await loadUsersAndRoles(); // Call the combined loading function on mount
    });

    // --- Updated handleRowClick ---
    async function handleRowClick(id: string, row: UserWithRoles) {
        selectedUser = row;
        await tick(); // Wait for Svelte to update the DOM (render the modal)
        if (editModalRef) {
            // Check if ref is now bound
            editModalRef.showModal();
        } else {
            console.error(
                "Modal reference (editModalRef) not found after tick.",
            );
        }
    }

    function handleRolesUpdated(userId: number, updatedRoles: Role[]) {
        if (users[userId.toString()]) {
            users[userId.toString()].roles = updatedRoles;
            users = { ...users }; // Trigger reactivity
        }
    }

    function handleUserDetailsUpdated(userId: number, newUsername: string, newEmail: string) {
        const userIdStr = userId.toString();
        if (users[userIdStr]) {
            users[userIdStr].username = newUsername;
            users[userIdStr].email = newEmail;
            users = { ...users }; // Trigger reactivity
        }
    }

    // Function to refresh the list, exposed globally
    async function refreshUserList() {
        await loadUsersAndRoles(); // Call the renamed data loading function
    }
    if (typeof window !== "undefined") {
        // Make sure the type definition includes the global function if using strict TS
        (window as any).refreshUserList = refreshUserList;
    }
</script>

{#if error}
    <div class="alert alert-error">{error}</div>
{/if}

<Table
    data={users}
    {columns}
    loading={isLoading}
    emptyMessage="Nenhum utilizador encontrado."
    keyField="id"
    searchFields={["username", "email", "roles"]}
    onRowClick={handleRowClick}
    rowClassName="hover:bg-base-300 cursor-pointer"
/>

{#if selectedUser}
    <AdminEditUserModal
        bind:modalRef={editModalRef}
        user={selectedUser}
        {allRoles}
        onRolesUpdated={handleRolesUpdated}
        onUserDetailsUpdated={handleUserDetailsUpdated}
    />
{/if}
