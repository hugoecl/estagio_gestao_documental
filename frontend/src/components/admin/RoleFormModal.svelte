<script lang="ts">
    import { tick } from "svelte"; // Use explicit $effect import
    import type {
        Role,
        CreateRoleRequest,
        UpdateRoleRequest,
    } from "@lib/types/roles";
    import { createRole, updateRole, deleteRole } from "@api/roles-api";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    let {
        modalRef = $bindable(),
        role = null, // One-way prop now
        onClose, // Callback to signal closure
        onRoleCreated,
        onRoleUpdated,
        onRoleDeleted,
    }: {
        modalRef: HTMLDialogElement;
        role?: Role | null;
        onClose: () => void; // Required callback
        onRoleCreated: (newRole: Role) => void;
        onRoleUpdated: (updatedRole: Role) => void;
        onRoleDeleted: (deletedId: number) => void;
    } = $props();

    // --- State ---
    let formData = $state<CreateRoleRequest | UpdateRoleRequest>({
        name: "",
        description: null,
        is_admin: false,
    });
    let originalDataJson = $state(""); // For checking changes
    let isSubmitting = $state(false);
    let isDeleting = $state(false);
    let errors = $state<Record<string, string>>({});
    let confirmModalRef: HTMLDialogElement;

    const isEditMode = $derived(role !== null && role?.id !== undefined);
    const modalTitle = $derived(
        isEditMode ? `Editar Função: ${role?.name ?? ""}` : "Criar Nova Função",
    );
    const submitButtonText = $derived(
        isEditMode ? "Guardar Alterações" : "Criar Função",
    );

    // --- Form Setup ---
    function setupForm(currentRole: Role | null) {
        if (currentRole) {
            formData = structuredClone({
                name: currentRole.name,
                description: currentRole.description,
                is_admin: currentRole.is_admin,
            });
            originalDataJson = JSON.stringify(formData);
        } else {
            formData = { name: "", description: null, is_admin: false };
            originalDataJson = JSON.stringify(formData);
        }
        errors = {};
        isSubmitting = false;
        isDeleting = false;
    }

    // --- Effect to setup form ONLY when role identity changes ---
    let previousRoleRef = $state<Role | null>(undefined); // Track previous role reference

    $effect(() => {
        const currentRole = role; // Capture current prop value
        // Check if role identity actually changed (null -> object, object -> null, or different object)
        if (previousRoleRef !== currentRole) {
            setupForm(currentRole);
            previousRoleRef = currentRole; // Update the reference tracker
        }
        // Add cleanup if needed when component unmounts or role becomes undefined again
        // return () => { console.log('Cleanup effect for role:', currentRole?.id); };
    });

    // --- Actions ---
    function closeModalAndNotify() {
        modalRef?.close();
        onClose(); // Call the parent's close handler
    }

    function validateForm(): boolean {
        // ... validation logic (no changes needed here) ...
        errors = {};
        if (!formData.name?.trim()) {
            errors.name = "Nome da função é obrigatório.";
        }
        if (isEditMode && role?.id === 1 && !formData.is_admin) {
            errors.is_admin =
                "A função Admin principal deve permanecer como administrador.";
        }
        return Object.keys(errors).length === 0;
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        // ... (handleSubmit logic remains largely the same) ...
        if (!validateForm()) {
            showAlert(
                "Existem erros no formulário.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }

        if (isEditMode && JSON.stringify(formData) === originalDataJson) {
            showAlert(
                "Nenhuma alteração detetada.",
                AlertType.INFO,
                AlertPosition.TOP,
            );
            closeModalAndNotify(); // Use new close function
            return;
        }

        isSubmitting = true;
        try {
            if (isEditMode && role) {
                const success = await updateRole(
                    role.id,
                    formData as UpdateRoleRequest,
                );
                if (success) {
                    showAlert(
                        "Função atualizada com sucesso!",
                        AlertType.SUCCESS,
                        AlertPosition.TOP,
                    );
                    onRoleUpdated({ ...role, ...formData });
                    closeModalAndNotify(); // Use new close function
                } else {
                    throw new Error("Falha ao atualizar função no backend.");
                }
            } else {
                const result = await createRole(formData as CreateRoleRequest);
                if (result.success && result.roleId) {
                    showAlert(
                        "Função criada com sucesso!",
                        AlertType.SUCCESS,
                        AlertPosition.TOP,
                    );
                    onRoleCreated({
                        id: result.roleId,
                        ...formData,
                        created_at: new Date().toISOString(),
                        updated_at: new Date().toISOString(),
                    } as Role);
                    closeModalAndNotify(); // Use new close function
                } else {
                    throw new Error("Falha ao criar função no backend.");
                }
            }
        } catch (e: any) {
            showAlert(`Erro: ${e.message}`, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isSubmitting = false;
        }
    }

    function handleDeleteClick() {
        // ... (logic remains the same) ...
        if (!isEditMode || !role || role.id === 1) return;
        confirmModalRef?.showModal();
    }

    async function handleDeleteConfirm() {
        // ... (logic remains largely the same) ...
        if (!isEditMode || !role || role.id === 1) return;
        isDeleting = true;
        try {
            const success = await deleteRole(role.id);
            if (success) {
                showAlert(
                    "Função eliminada com sucesso!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                onRoleDeleted(role.id);
                closeModalAndNotify(); // Use new close function
            } else {
                throw new Error("Falha ao eliminar função no backend.");
            }
        } catch (e: any) {
            showAlert(
                `Erro ao eliminar: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isDeleting = false;
            confirmModalRef?.close(); // Close only confirmation modal here
        }
    }
</script>

<!-- Main Form Modal -->
<dialog class="modal" bind:this={modalRef}>
    <div class="modal-box">
        <div class="flex justify-between items-center mb-4">
            <h3 class="font-bold text-lg">{modalTitle}</h3>
            <button
                class="btn btn-sm btn-ghost absolute right-2 top-2"
                onclick={closeModalAndNotify}
                disabled={isSubmitting}>✕</button
            >
        </div>

        <form onsubmit={handleSubmit} class="space-y-4">
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Nome da Função*</span>
                </div>
                <input
                    type="text"
                    placeholder="Ex: Recursos Humanos"
                    class="input input-bordered w-full"
                    bind:value={formData.name}
                    required
                    disabled={isSubmitting || (isEditMode && role?.id === 1)}
                    class:input-error={errors.name}
                />
                {#if errors.name}<span class="text-error text-xs mt-1"
                        >{errors.name}</span
                    >{/if}
                {#if isEditMode && role?.id === 1}
                    <div class="label">
                        <span class="label-text-alt text-warning text-xs"
                            >O nome da função Admin não pode ser alterado.</span
                        >
                    </div>
                {/if}
            </label>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Descrição (Opcional)</span>
                </div>
                <textarea
                    placeholder="Breve descrição da função"
                    class="textarea textarea-bordered w-full"
                    rows="3"
                    bind:value={formData.description}
                    disabled={isSubmitting}
                ></textarea>
            </label>

            <div class="form-control mt-3">
                <label class="label cursor-pointer justify-start gap-2">
                    <input
                        type="checkbox"
                        class="toggle toggle-primary"
                        bind:checked={formData.is_admin}
                        disabled={isSubmitting ||
                            (isEditMode && role?.id === 1)}
                    />
                    <span class="label-text font-medium"
                        >Permissões de Administrador?</span
                    >
                </label>
                {#if isEditMode && role?.id === 1}
                    <div class="label">
                        <span class="label-text-alt text-warning text-xs"
                            >A função Admin deve manter permissões de
                            administrador.</span
                        >
                    </div>
                {/if}
                {#if errors.is_admin}<span class="text-error text-xs mt-1"
                        >{errors.is_admin}</span
                    >{/if}
            </div>

            <div
                class="modal-action flex flex-col-reverse sm:flex-row justify-between pt-4 mt-4 border-t"
            >
                <div>
                    {#if isEditMode && role?.id !== 1}
                        <button
                            type="button"
                            class="btn btn-error w-full sm:w-auto"
                            onclick={handleDeleteClick}
                            disabled={isSubmitting || isDeleting}
                            >Eliminar</button
                        >
                    {/if}
                </div>
                <div
                    class="flex flex-col-reverse sm:flex-row gap-2 w-full sm:w-auto"
                >
                    <button
                        type="button"
                        class="btn btn-ghost w-full sm:w-auto"
                        onclick={closeModalAndNotify}
                        disabled={isSubmitting}>Cancelar</button
                    >
                    <button
                        type="submit"
                        class="btn btn-primary w-full sm:w-auto"
                        disabled={isSubmitting}
                    >
                        {#if isSubmitting}
                            <span class="loading loading-spinner loading-sm"
                            ></span> A Guardar...
                        {:else}
                            {submitButtonText}
                        {/if}
                    </button>
                </div>
            </div>
        </form>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button
            disabled={isSubmitting || isDeleting}
            onclick={closeModalAndNotify}>close</button
        >
    </form>
</dialog>

<!-- Delete Confirmation Modal -->
<dialog class="modal z-[10000]" bind:this={confirmModalRef}>
    <div class="modal-box">
        <h3 class="font-bold text-lg mb-4">Confirmar Eliminação</h3>
        <p class="py-4">
            Tem a certeza que deseja eliminar a função <span
                class="font-semibold">{role?.name}</span
            >? Esta ação não pode ser desfeita.
        </p>
        <div class="modal-action flex justify-end gap-2">
            <button
                class="btn btn-ghost"
                onclick={() => confirmModalRef?.close()}
                disabled={isDeleting}>Cancelar</button
            >
            <button
                class="btn btn-error"
                onclick={handleDeleteConfirm}
                disabled={isDeleting}
            >
                {#if isDeleting}
                    <span class="loading loading-spinner loading-sm"></span> A Apagar...
                {:else}
                    Sim, Eliminar
                {/if}
            </button>
        </div>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button onclick={() => confirmModalRef?.close()} disabled={isDeleting}
            >close</button
        >
    </form>
</dialog>
