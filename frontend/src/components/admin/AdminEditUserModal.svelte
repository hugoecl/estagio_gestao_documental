<script lang="ts">
    import { tick, onMount } from "svelte";
    import type {
        UserWithRoles,
        AdminUpdateUserPayload,
        AdminSetPasswordPayload,
    } from "@lib/types/user";
    import type { Role } from "@lib/types/roles";
    import {
        assignRolesToUser,
        adminUpdateUserDetails,
        adminSetUserPassword,
        deleteUser,
    } from "@api/user-api";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    let {
        modalRef = $bindable(),
        user, // This is UserWithRoles
        allRoles,
        onRolesUpdated, // For role changes
        onUserDetailsUpdated, // For username/email changes
    }: {
        modalRef: HTMLDialogElement;
        user: UserWithRoles; // Assume UserWithRoles will be updated to include vacation_days_current_year
        allRoles: Role[];
        onRolesUpdated: (userId: number, updatedRoles: Role[]) => void;
        onUserDetailsUpdated: (
            userId: number,
            newUsername: string,
            newEmail: string,
            newVacationDays: number | null,
        ) => void;
    } = $props();

    // --- Common State ---
    let currentTab = $state<"roles" | "details" | "password">("roles");
    let isSubmittingRoles = $state(false);
    let isSubmittingDetails = $state(false);
    let isSubmittingPassword = $state(false);
    let isSubmittingDelete = $state(false);
    
    // --- Delete User Modal State ---
    let deleteModalRef = $state<HTMLDialogElement | null>(null);
    let userToDelete = $state<UserWithRoles | null>(null);

    // --- Roles State ---
    let selectedRoleIds = $state<Set<number>>(new Set());

    // --- Details State ---
    let editUsername = $state("");
    let editEmail = $state("");
    let editVacationDays = $state<number | null>(null);
    let detailsErrors = $state<Record<string, string>>({});

    // --- Password State ---
    let newPassword = $state("");
    let confirmNewPassword = $state("");
    let passwordErrors = $state<Record<string, string>>({});

    // Reactive effect to reset form fields when the user prop changes (modal opens for a new/different user)
    $effect(() => {
        if (user) {
            // Roles
            selectedRoleIds = new Set(user.roles.map((r) => r.id));

            // Details
            editUsername = user.username;
            editEmail = user.email;
            editVacationDays = user.vacation_days_current_year ?? 0; // Default to 0 if undefined/null
            detailsErrors = {};

            // Password
            newPassword = "";
            confirmNewPassword = "";
            passwordErrors = {};

            // Reset submitting states
            isSubmittingRoles = false;
            isSubmittingDetails = false;
            isSubmittingPassword = false;
            isSubmittingDelete = false;

            // Default to roles tab
            currentTab = "roles";
        } else {
            selectedRoleIds = new Set();
            editUsername = "";
            editEmail = "";
            editVacationDays = 0;
            newPassword = "";
            confirmNewPassword = "";
            currentTab = "roles";
        }
    });

    function closeModal() {
        modalRef?.close();
        // Reset states if needed when modal is closed externally or by cancel button
        // The $effect above handles reset when user prop changes, which is typical for opening.
    }

    // --- Roles Logic ---
    async function handleRolesSubmit(e: Event) {
        e.preventDefault();
        if (!user) return;
        isSubmittingRoles = true;
        const assignment = {
            user_id: user.id,
            role_ids: Array.from(selectedRoleIds),
        };

        try {
            const success = await assignRolesToUser(assignment);
            if (success) {
                showAlert(
                    "Funções atualizadas com sucesso!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                const updatedRoles = allRoles.filter((role) =>
                    selectedRoleIds.has(role.id),
                );
                onRolesUpdated(user.id, updatedRoles);
                closeModal(); // Or keep modal open on current tab?
            } else {
                throw new Error("Falha ao atualizar funções no backend.");
            }
        } catch (e: any) {
            showAlert(
                `Erro ao atualizar funções: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmittingRoles = false;
        }
    }

    function handleRoleCheckboxChange(roleId: number, checked: boolean) {
        if (checked) {
            selectedRoleIds.add(roleId);
        } else {
            selectedRoleIds.delete(roleId);
        }
        selectedRoleIds = new Set(selectedRoleIds);
    }

    // --- Details Logic ---
    function validateDetailsForm(): boolean {
        detailsErrors = {};
        if (!editUsername.trim()) {
            detailsErrors.username = "Nome de utilizador é obrigatório.";
        }
        if (!editEmail.trim()) {
            detailsErrors.email = "Email é obrigatório.";
        } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(editEmail)) {
            detailsErrors.email = "Formato de e-mail inválido.";
        }
        if (
            editVacationDays !== null &&
            (isNaN(editVacationDays) || editVacationDays < 0)
        ) {
            detailsErrors.vacationDays =
                "Dias de férias deve ser um número não negativo.";
        }

        // Check if anything actually changed
        if (
            editUsername === user?.username &&
            editEmail === user?.email &&
            editVacationDays === (user?.vacation_days_current_year ?? 0)
        ) {
            showAlert(
                "Nenhuma alteração detetada nos detalhes.",
                AlertType.INFO,
                AlertPosition.TOP,
            );
            return false; // Not an error, but no submission needed
        }
        return Object.keys(detailsErrors).length === 0;
    }

    async function handleDetailsSubmit(e: Event) {
        e.preventDefault();
        if (!user || !validateDetailsForm()) {
            if (Object.keys(detailsErrors).length > 0) {
                showAlert(
                    "Por favor corrija os erros no formulário de detalhes.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
            return;
        }
        isSubmittingDetails = true;
        const payload: AdminUpdateUserPayload = {};
        if (editUsername !== user.username) payload.username = editUsername;
        if (editEmail !== user.email) payload.email = editEmail;
        if (
            editVacationDays !== null &&
            editVacationDays !== (user.vacation_days_current_year ?? 0)
        ) {
            payload.vacation_days_current_year = editVacationDays;
        }

        try {
            const result = await adminUpdateUserDetails(user.id, payload);
            if (result.success) {
                showAlert(
                    result.message || "Detalhes do utilizador atualizados!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                onUserDetailsUpdated(
                    user.id,
                    editUsername,
                    editEmail,
                    editVacationDays,
                );
                closeModal();
            } else {
                showAlert(
                    result.message || "Falha ao atualizar detalhes.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
        } catch (e: any) {
            showAlert(
                `Erro ao atualizar detalhes: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmittingDetails = false;
        }
    }

    // --- Password Logic ---
    function validatePasswordForm(): boolean {
        passwordErrors = {};
        if (!newPassword) {
            passwordErrors.newPassword = "Nova palavra-passe é obrigatória.";
        } else if (newPassword.length < 3) {
            passwordErrors.newPassword =
                "Nova palavra-passe deve ter pelo menos 3 caracteres.";
        }
        if (newPassword !== confirmNewPassword) {
            passwordErrors.confirmNewPassword =
                "As novas palavras-passe não coincidem.";
        }
        return Object.keys(passwordErrors).length === 0;
    }

    async function handlePasswordSubmit(e: Event) {
        e.preventDefault();
        if (!user || !validatePasswordForm()) {
            showAlert(
                "Por favor corrija os erros no formulário de palavra-passe.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }
        isSubmittingPassword = true;
        const payload: AdminSetPasswordPayload = { new_password: newPassword };

        try {
            const result = await adminSetUserPassword(user.id, payload);
            if (result.success) {
                showAlert(
                    result.message || "Palavra-passe definida com sucesso!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                newPassword = "";
                confirmNewPassword = "";
                closeModal();
            } else {
                showAlert(
                    result.message || "Falha ao definir palavra-passe.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
        } catch (e: any) {
            showAlert(
                `Erro ao definir palavra-passe: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmittingPassword = false;
        }
    }
    
    // Open delete confirmation modal
    function openDeleteModal() {
        if (!user) return;
        userToDelete = user;
        deleteModalRef?.showModal();
    }
    
    // Close delete confirmation modal
    function closeDeleteModal() {
        deleteModalRef?.close();
        userToDelete = null;
    }
    
    // Delete user function
    async function handleDeleteUser() {
        if (!userToDelete) return;
        
        isSubmittingDelete = true;
        try {
            const result = await deleteUser(userToDelete.id);
            if (result.success) {
                showAlert(
                    result.message || "Utilizador eliminado com sucesso!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                closeDeleteModal();
                closeModal();
                // Refresh the user list if there's a global refreshUserList function
                if (typeof window !== "undefined" && (window as any).refreshUserList) {
                    (window as any).refreshUserList();
                }
            } else {
                showAlert(
                    result.message || "Falha ao eliminar o utilizador.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
        } catch (e: any) {
            showAlert(
                `Erro ao eliminar o utilizador: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmittingDelete = false;
        }
    }
</script>

<dialog class="modal" bind:this={modalRef}>
    <div class="modal-box w-11/12 max-w-2xl">
        <div class="flex justify-between items-center mb-4">
            <h3 class="font-bold text-lg">
                Editar Utilizador: {user?.username} ({user?.email})
            </h3>
            <button
                class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                onclick={closeModal}
                disabled={isSubmittingRoles ||
                    isSubmittingDetails ||
                    isSubmittingPassword}>✕</button
            >
        </div>

        <!-- Tabs -->
        <div role="tablist" class="tabs tabs-lifted tabs-lg mb-6">
            <button
                role="tab"
                class="tab"
                class:tab-active={currentTab === "roles"}
                onclick={() => (currentTab = "roles")}>Funções</button
            >
            <button
                role="tab"
                class="tab"
                class:tab-active={currentTab === "details"}
                onclick={() => (currentTab = "details")}>Detalhes</button
            >
            <button
                role="tab"
                class="tab"
                class:tab-active={currentTab === "password"}
                onclick={() => (currentTab = "password")}>Palavra-passe</button
            >
        </div>

        <!-- Tab Content -->
        <div>
            {#if currentTab === "roles"}
                <form onsubmit={handleRolesSubmit} class="space-y-4">
                    <p class="text-sm text-base-content/80">
                        Selecione as funções para atribuir ao utilizador:
                    </p>
                    <div
                        class="max-h-60 overflow-y-auto space-y-1 border p-3 rounded-md bg-base-200"
                    >
                        {#each allRoles as role (role.id)}
                            {@const isChecked = selectedRoleIds.has(role.id)}
                            <label
                                class="label cursor-pointer justify-start gap-3 p-1.5 hover:bg-base-300 rounded w-full"
                            >
                                <input
                                    type="checkbox"
                                    class="checkbox checkbox-primary checkbox-sm"
                                    checked={isChecked}
                                    onchange={(e) =>
                                        handleRoleCheckboxChange(
                                            role.id,
                                            (e.target as HTMLInputElement)
                                                .checked,
                                        )}
                                    disabled={isSubmittingRoles}
                                />
                                <span class="label-text text-sm"
                                    >{role.name}
                                    {#if role.is_admin}<span
                                            class="text-xs opacity-70 ml-1"
                                            >(Admin)</span
                                        >{/if}
                                </span>
                                {#if role.description}
                                    <span
                                        class="label-text-alt text-xs opacity-60 ml-auto truncate"
                                        title={role.description}
                                        >- {role.description}</span
                                    >
                                {/if}
                            </label>
                        {/each}
                    </div>
                    <div class="modal-action mt-6 flex justify-between w-full">
                        <button
                            type="button"
                            class="btn btn-error"
                            onclick={openDeleteModal}
                            disabled={isSubmittingRoles}
                        >
                            <i class="fa-solid fa-trash mr-1"></i>
                            Eliminar Utilizador
                        </button>
                        <div>
                            <button
                                type="button"
                                class="btn btn-ghost"
                                onclick={closeModal}
                                disabled={isSubmittingRoles || isSubmittingDelete}>Cancelar</button
                            >
                            <button
                                type="submit"
                                class="btn btn-primary"
                                disabled={isSubmittingRoles || isSubmittingDelete}
                            >
                                {#if isSubmittingRoles}
                                    <span class="loading loading-spinner loading-sm"
                                    ></span> A Guardar Funções...
                                {:else}
                                    Guardar Funções
                                {/if}
                            </button>
                        </div>
                    </div>
                </form>
            {/if}

            {#if currentTab === "details"}
                <form onsubmit={handleDetailsSubmit} class="space-y-4">
                    <label class="form-control w-full">
                        <div class="label">
                            <span class="label-text">Nome de Utilizador</span>
                        </div>
                        <input
                            type="text"
                            placeholder="Nome de utilizador"
                            class="input input-bordered w-full"
                            bind:value={editUsername}
                            disabled={isSubmittingDetails}
                            class:input-error={detailsErrors.username}
                        />
                        {#if detailsErrors.username}<span
                                class="text-error text-xs mt-1"
                                >{detailsErrors.username}</span
                            >{/if}
                    </label>
                    <label class="form-control w-full">
                        <div class="label">
                            <span class="label-text">Email</span>
                        </div>
                        <input
                            type="email"
                            placeholder="Email do utilizador"
                            class="input input-bordered w-full"
                            bind:value={editEmail}
                            disabled={isSubmittingDetails}
                            class:input-error={detailsErrors.email}
                        />
                        {#if detailsErrors.email}<span
                                class="text-error text-xs mt-1"
                                >{detailsErrors.email}</span
                            >{/if}
                    </label>
                    <label class="form-control w-full">
                        <div class="label">
                            <span class="label-text"
                                >Dias de Férias (Ano Atual)</span
                            >
                        </div>
                        <input
                            type="number"
                            min="0"
                            placeholder="Número de dias de férias"
                            class="input input-bordered w-full"
                            bind:value={editVacationDays}
                            disabled={isSubmittingDetails}
                            class:input-error={detailsErrors.vacationDays}
                        />
                        {#if detailsErrors.vacationDays}<span
                                class="text-error text-xs mt-1"
                                >{detailsErrors.vacationDays}</span
                            >{/if}
                    </label>
                    <div class="modal-action mt-6 flex justify-between w-full">
                        <button
                            type="button"
                            class="btn btn-error"
                            onclick={openDeleteModal}
                            disabled={isSubmittingDetails}
                        >
                            <i class="fa-solid fa-trash mr-1"></i>
                            Eliminar Utilizador
                        </button>
                        <div>
                            <button
                                type="button"
                                class="btn btn-ghost"
                                onclick={closeModal}
                                disabled={isSubmittingDetails || isSubmittingDelete}>Cancelar</button
                            >
                            <button
                                type="submit"
                                class="btn btn-primary"
                                disabled={isSubmittingDetails || isSubmittingDelete}
                            >
                                {#if isSubmittingDetails}
                                    <span class="loading loading-spinner loading-sm"
                                    ></span> A Guardar Detalhes...
                                {:else}
                                    Guardar Detalhes
                                {/if}
                            </button>
                        </div>
                    </div>
                </form>
            {/if}

            {#if currentTab === "password"}
                <form onsubmit={handlePasswordSubmit} class="space-y-4">
                    <p
                        class="text-sm text-warning-content bg-warning p-2 rounded-md"
                    >
                        <i class="fa-solid fa-triangle-exclamation mr-1"></i>
                        Alterar a palavra-passe aqui irá definir uma nova palavra-passe
                        para o utilizador. O utilizador não precisará da sua palavra-passe
                        antiga.
                    </p>
                    <label class="form-control w-full">
                        <div class="label">
                            <span class="label-text">Nova Palavra-passe</span>
                        </div>
                        <input
                            type="password"
                            placeholder="Nova palavra-passe (mín. 3 caracteres)"
                            class="input input-bordered w-full"
                            bind:value={newPassword}
                            disabled={isSubmittingPassword}
                            class:input-error={passwordErrors.newPassword}
                        />
                        {#if passwordErrors.newPassword}<span
                                class="text-error text-xs mt-1"
                                >{passwordErrors.newPassword}</span
                            >{/if}
                    </label>
                    <label class="form-control w-full">
                        <div class="label">
                            <span class="label-text"
                                >Confirmar Nova Palavra-passe</span
                            >
                        </div>
                        <input
                            type="password"
                            placeholder="Confirme a nova palavra-passe"
                            class="input input-bordered w-full"
                            bind:value={confirmNewPassword}
                            disabled={isSubmittingPassword}
                            class:input-error={passwordErrors.confirmNewPassword}
                        />
                        {#if passwordErrors.confirmNewPassword}<span
                                class="text-error text-xs mt-1"
                                >{passwordErrors.confirmNewPassword}</span
                            >{/if}
                    </label>
                    <div class="modal-action mt-6 flex justify-between w-full">
                        <button
                            type="button"
                            class="btn btn-error"
                            onclick={openDeleteModal}
                            disabled={isSubmittingPassword}
                        >
                            <i class="fa-solid fa-trash mr-1"></i>
                            Eliminar Utilizador
                        </button>
                        <div>
                            <button
                                type="button"
                                class="btn btn-ghost"
                                onclick={closeModal}
                                disabled={isSubmittingPassword || isSubmittingDelete}>Cancelar</button
                            >
                            <button
                                type="submit"
                                class="btn btn-primary"
                                disabled={isSubmittingPassword || isSubmittingDelete}
                            >
                                {#if isSubmittingPassword}
                                    <span class="loading loading-spinner loading-sm"
                                    ></span> A Guardar Palavra-passe...
                                {:else}
                                    Guardar Palavra-passe
                                {/if}
                            </button>
                        </div>
                    </div>
                </form>
            {/if}
        </div>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button
            onclick={closeModal}
            disabled={isSubmittingRoles ||
                isSubmittingDetails ||
                isSubmittingPassword}>close</button
        >
    </form>
</dialog>

<!-- Delete User Confirmation Modal -->
<dialog class="modal modal-bottom sm:modal-middle" bind:this={deleteModalRef}>
    <div class="modal-box">
        <h3 class="font-bold text-lg">Confirmar Eliminação</h3>
        <p class="py-4">
            Tem certeza que deseja eliminar o utilizador <strong>{userToDelete?.username}</strong>?
            <br />
            <span class="text-error">Esta ação não pode ser desfeita.</span>
        </p>
        <div class="modal-action">
            <button type="button" class="btn btn-ghost" onclick={closeDeleteModal} disabled={isSubmittingDelete}>
                Cancelar
            </button>
            <button
                type="button"
                class="btn btn-error"
                onclick={handleDeleteUser}
                disabled={isSubmittingDelete}
            >
                {#if isSubmittingDelete}
                    <span class="loading loading-spinner loading-sm"></span>
                    A Eliminar...
                {:else}
                    Sim, Eliminar
                {/if}
            </button>
        </div>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button onclick={closeDeleteModal} disabled={isSubmittingDelete}>close</button>
    </form>
</dialog>
