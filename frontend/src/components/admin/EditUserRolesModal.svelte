<script lang="ts">
    import { tick } from "svelte";
    import type { UserWithRoles } from "@lib/types/user";
    import type { Role } from "@lib/types/roles";
    import { assignRolesToUser } from "@api/user-api";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    let {
        modalRef = $bindable(),
        user,
        allRoles,
        onRolesUpdated,
    }: {
        modalRef: HTMLDialogElement;
        user: UserWithRoles;
        allRoles: Role[];
        onRolesUpdated: (userId: number, updatedRoles: Role[]) => void;
    } = $props();

    let selectedRoleIds = $state<Set<number>>(new Set());
    let isSubmitting = $state(false);

    // Initialize selected roles when user prop changes
    $effect(() => {
        if (user) {
            selectedRoleIds = new Set(user.roles.map((r) => r.id));
        } else {
            selectedRoleIds = new Set();
        }
    });

    function closeModal() {
        modalRef?.close();
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        isSubmitting = true;
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
                // Find the full Role objects for the updated IDs
                const updatedRoles = allRoles.filter((role) =>
                    selectedRoleIds.has(role.id),
                );
                onRolesUpdated(user.id, updatedRoles); // Notify parent
                closeModal();
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
            isSubmitting = false;
        }
    }

    function handleCheckboxChange(roleId: number, checked: boolean) {
        if (checked) {
            selectedRoleIds.add(roleId);
        } else {
            selectedRoleIds.delete(roleId);
        }
        // Trigger reactivity if needed, though Set operations might be reactive in Svelte 5
        selectedRoleIds = new Set(selectedRoleIds);
    }
</script>

<dialog class="modal" bind:this={modalRef}>
    <div class="modal-box">
        <div class="flex justify-between items-center mb-4">
            <h3 class="font-bold text-lg">
                Editar Funções - {user?.username} ({user?.email})
            </h3>
            <button
                class="btn btn-sm btn-ghost"
                onclick={closeModal}
                disabled={isSubmitting}>✕</button
            >
        </div>

        <form onsubmit={handleSubmit} class="space-y-4">
            <p>Selecione as funções para atribuir:</p>
            <div
                class="max-h-60 overflow-y-auto space-y-2 border p-2 rounded-md bg-base-200"
            >
                {#each allRoles as role (role.id)}
                    {@const isChecked = selectedRoleIds.has(role.id)}
                    <label
                        class="label cursor-pointer justify-start gap-3 p-2 hover:bg-base-300 rounded"
                    >
                        <input
                            type="checkbox"
                            class="checkbox checkbox-primary checkbox-sm"
                            checked={isChecked}
                            onchange={(e) =>
                                handleCheckboxChange(
                                    role.id,
                                    (e.target as HTMLInputElement).checked,
                                )}
                            disabled={isSubmitting}
                        />
                        <span class="label-text"
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
                    disabled={isSubmitting}
                >
                    {#if isSubmitting}
                        <span class="loading loading-spinner loading-sm"></span>
                        Guardando...
                    {:else}
                        Guardar Alterações
                    {/if}
                </button>
            </div>
        </form>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button disabled={isSubmitting} onclick={closeModal}>close</button>
    </form>
</dialog>
