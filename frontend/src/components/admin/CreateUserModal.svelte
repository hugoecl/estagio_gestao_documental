<script lang="ts">
    import { onMount, tick } from "svelte";
    import type { Role } from "@lib/types/roles";
    import type { CreateUserRequest, UserWithRoles } from "@lib/types/user";
    import { getRoles } from "@api/roles-api";
    import { createUser, assignRolesToUser } from "@api/user-api"; // Assuming createUser exists
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    let modalRef: HTMLDialogElement;
    let allRoles = $state<Role[]>([]);
    let selectedRoleIds = $state<Set<number>>(new Set());
    let formData = $state<CreateUserRequest>({
        username: "",
        email: "",
        password: "",
    });
    let isLoadingRoles = $state(true);
    let isSubmitting = $state(false);
    let errors = $state<Record<string, string>>({});

    onMount(async () => {
        try {
            allRoles = await getRoles();
            // Pre-select the default "Colaborador" role if it exists
            const defaultRole = allRoles.find((r) => r.name === "Colaborador");
            if (defaultRole) {
                selectedRoleIds.add(defaultRole.id);
                selectedRoleIds = new Set(selectedRoleIds); // Trigger reactivity
            }
        } catch (e) {
            showAlert(
                "Erro ao carregar funções.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            console.error("Failed to load roles:", e);
        } finally {
            isLoadingRoles = false;
        }
    });

    // Expose openModal globally
    if (typeof window !== "undefined") {
        (window as any).openCreateUserModal = openModal;
    }

    function openModal() {
        resetForm();
        modalRef?.showModal();
    }

    function closeModal() {
        modalRef?.close();
    }

    function resetForm() {
        formData = { username: "", email: "", password: "" };
        // Reset selected roles, keeping default if applicable
        const defaultRole = allRoles.find((r) => r.name === "Colaborador");
        selectedRoleIds = defaultRole ? new Set([defaultRole.id]) : new Set();
        errors = {};
        isSubmitting = false;
    }

    function validateForm(): boolean {
        errors = {};
        if (!formData.username.trim())
            errors.username = "Nome de utilizador é obrigatório.";
        if (!formData.email.trim()) errors.email = "Email é obrigatório.";
        else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email))
            errors.email = "Formato de e-mail inválido.";
        if (!formData.password)
            errors.password = "Palavra-passe é obrigatória.";
        // Optionally add password strength validation
        if (selectedRoleIds.size === 0)
            errors.roles = "Selecione pelo menos uma função.";

        return Object.keys(errors).length === 0;
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        if (!validateForm()) {
            showAlert(
                "Existem erros no formulário.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }

        isSubmitting = true;
        try {
            // 1. Create the user
            const createResult = await createUser(formData);

            if (!createResult.success || createResult.userId === undefined) {
                throw new Error(
                    createResult.error || "Falha ao criar utilizador.",
                );
            }

            const newUserId = createResult.userId;

            // 2. Assign selected roles (if different from default)
            const defaultRole = allRoles.find((r) => r.name === "Colaborador");
            const isDefaultOnly =
                selectedRoleIds.size === 1 &&
                defaultRole &&
                selectedRoleIds.has(defaultRole.id);

            // Only call assignRoles if non-default roles are selected or if default wasn't pre-selected
            if (!isDefaultOnly || !defaultRole) {
                const assignSuccess = await assignRolesToUser({
                    user_id: newUserId,
                    role_ids: Array.from(selectedRoleIds),
                });

                if (!assignSuccess) {
                    // User created, but roles failed. Show warning.
                    showAlert(
                        `Utilizador ${formData.username} criado, mas falha ao atribuir funções. Edite o utilizador para corrigir.`,
                        AlertType.WARNING,
                        AlertPosition.TOP,
                    );
                    // Optionally, notify parent list about the new user anyway?
                    // For simplicity, we'll just close here. Manual edit needed.
                    closeModal();
                    // Refresh the list even if role assignment failed
                    if (window.refreshUserList) window.refreshUserList();
                    return; // Exit function
                }
            }

            showAlert(
                `Utilizador ${formData.username} criado com sucesso!`,
                AlertType.SUCCESS,
                AlertPosition.TOP,
            );

            // Refresh the parent list
            if (window.refreshUserList) {
                window.refreshUserList();
            } else {
                console.warn(
                    "refreshUserList function not found, cannot update table automatically.",
                );
            }

            closeModal();
        } catch (e: any) {
            showAlert(
                `Erro ao criar utilizador: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmitting = false;
        }
    }

    function handleCheckboxChange(roleId: number, checked: boolean) {
        // Prevent unselecting the last role
        if (
            !checked &&
            selectedRoleIds.size === 1 &&
            selectedRoleIds.has(roleId)
        ) {
            showAlert(
                "O utilizador deve ter pelo menos uma função.",
                AlertType.WARNING,
                AlertPosition.TOP,
            );
            // Re-check the checkbox visually (might need tick or direct manipulation)
            tick().then(() => {
                const checkbox = modalRef.querySelector(
                    `input[type="checkbox"][value="${roleId}"]`,
                ) as HTMLInputElement;
                if (checkbox) checkbox.checked = true;
            });
            return;
        }

        if (checked) {
            selectedRoleIds.add(roleId);
        } else {
            selectedRoleIds.delete(roleId);
        }
        selectedRoleIds = new Set(selectedRoleIds); // Trigger reactivity
        // Clear potential role error
        if (errors.roles && selectedRoleIds.size > 0) {
            delete errors.roles;
            errors = { ...errors };
        }
    }
</script>

<dialog class="modal" bind:this={modalRef}>
    <div class="modal-box w-11/12 max-w-lg">
        <div class="flex justify-between items-center mb-4">
            <h3 class="font-bold text-lg">Criar Novo Utilizador</h3>
            <button
                class="btn btn-sm btn-ghost absolute right-2 top-2"
                onclick={closeModal}
                disabled={isSubmitting}>✕</button
            >
        </div>

        <form onsubmit={handleSubmit} class="space-y-4">
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Nome de Utilizador*</span>
                </div>
                <input
                    type="text"
                    placeholder="Ex: joao.silva"
                    class="input input-bordered w-full"
                    bind:value={formData.username}
                    required
                    disabled={isSubmitting}
                    class:input-error={errors.username}
                />
                {#if errors.username}<span class="text-error text-xs mt-1"
                        >{errors.username}</span
                    >{/if}
            </label>

            <label class="form-control w-full">
                <div class="label"><span class="label-text">Email*</span></div>
                <input
                    type="email"
                    placeholder="Ex: joao.silva@jcc.pt"
                    class="input input-bordered w-full"
                    bind:value={formData.email}
                    required
                    disabled={isSubmitting}
                    class:input-error={errors.email}
                />
                {#if errors.email}<span class="text-error text-xs mt-1"
                        >{errors.email}</span
                    >{/if}
            </label>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Palavra-passe*</span>
                </div>
                <input
                    type="password"
                    placeholder="Palavra-passe"
                    class="input input-bordered w-full"
                    bind:value={formData.password}
                    required
                    disabled={isSubmitting}
                    class:input-error={errors.password}
                />
                {#if errors.password}<span class="text-error text-xs mt-1"
                        >{errors.password}</span
                    >{/if}
            </label>

            <div class="form-control w-full">
                <div class="label">
                    <span class="label-text">Funções*</span>
                </div>
                {#if isLoadingRoles}
                    <span class="loading loading-sm"></span>
                {:else if allRoles.length === 0}
                    <p class="text-warning text-xs">
                        Nenhuma função disponível.
                    </p>
                {:else}
                    <div
                        class="max-h-40 overflow-y-auto space-y-1 border p-2 rounded-md bg-base-200"
                    >
                        {#each allRoles as role (role.id)}
                            {@const isChecked = selectedRoleIds.has(role.id)}
                            <label
                                class="label cursor-pointer justify-start gap-3 p-1 hover:bg-base-300 rounded"
                            >
                                <input
                                    type="checkbox"
                                    class="checkbox checkbox-primary checkbox-sm"
                                    value={role.id}
                                    checked={isChecked}
                                    onchange={(e) =>
                                        handleCheckboxChange(
                                            role.id,
                                            (e.target as HTMLInputElement)
                                                .checked,
                                        )}
                                    disabled={isSubmitting}
                                />
                                <span class="label-text text-sm"
                                    >{role.name}
                                    {#if role.is_admin}(Admin){/if}</span
                                >
                                {#if role.description}
                                    <span
                                        class="label-text-alt text-xs opacity-60 ml-auto truncate"
                                        title={role.description}
                                    >
                                        - {role.description}</span
                                    >
                                {/if}
                            </label>
                        {/each}
                    </div>
                {/if}
                {#if errors.roles}<span class="text-error text-xs mt-1"
                        >{errors.roles}</span
                    >{/if}
            </div>

            <div class="modal-action mt-6">
                <button
                    type="button"
                    class="btn btn-ghost"
                    onclick={closeModal}
                    disabled={isSubmitting}>Cancelar</button
                >
                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled={isSubmitting || isLoadingRoles}
                >
                    {#if isSubmitting}
                        <span class="loading loading-spinner loading-sm"></span>
                        A Criar...
                    {:else}
                        Criar Utilizador
                    {/if}
                </button>
            </div>
        </form>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button disabled={isSubmitting} onclick={closeModal}>close</button>
    </form>
</dialog>
