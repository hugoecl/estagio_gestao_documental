<script lang="ts">
    import { onMount, tick } from "svelte";
    import { getRoles } from "@api/roles-api";
    import { broadcastNotification } from "@api/notification-api.ts"; // Ensure .ts is there if needed by your setup
    import type { Role } from "@lib/types/roles";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    let allRoles = $state<Role[]>([]);
    let selectedRoleIds = $state<Set<number>>(new Set());
    let message = $state("");
    let isLoading = $state(true);
    let isSubmitting = $state(false);
    let errors = $state<Record<string, string>>({});

    onMount(async () => {
        try {
            allRoles = await getRoles();
        } catch (e: any) {
            showAlert(
                `Erro ao carregar funções: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isLoading = false;
        }
    });

    function validateForm(): boolean {
        errors = {};
        if (selectedRoleIds.size === 0) {
            errors.roles = "Selecione pelo menos uma função.";
        }
        if (!message.trim()) {
            errors.message = "A mensagem não pode estar vazia.";
        }
        return Object.keys(errors).length === 0;
    }

    async function handleSubmit(e: Event) {
        e.preventDefault();
        if (!validateForm()) {
            showAlert(
                "Por favor, corrija os erros no formulário.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }

        isSubmitting = true;
        try {
            const result = await broadcastNotification(
                Array.from(selectedRoleIds),
                message,
            );

            if (result.success) {
                showAlert(
                    result.message || "Mensagem enviada com sucesso!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                // Reset form
                selectedRoleIds = new Set();
                message = "";
                // Manually clear checkboxes if necessary (Svelte's reactivity might handle it with Set change)
                // This might be needed if checkboxes don't visually clear
                await tick();
                const checkboxes = document.querySelectorAll<HTMLInputElement>(
                    'input[name="broadcast-role-checkbox"]',
                );
                checkboxes.forEach((cb) => (cb.checked = false));
            } else {
                showAlert(
                    result.message || "Falha ao enviar mensagem.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
        } catch (e: any) {
            showAlert(
                `Erro ao enviar mensagem: ${e.message}`,
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
        selectedRoleIds = new Set(selectedRoleIds); // Trigger reactivity for Svelte 5
        if (errors.roles && selectedRoleIds.size > 0) {
            delete errors.roles;
            errors = { ...errors };
        }
    }
</script>

<form onsubmit={handleSubmit} class="space-y-6">
    <div>
        <label
            for="roles-select"
            class="block text-sm font-medium text-base-content"
            >Enviar Para Funções*</label
        >
        {#if isLoading}
            <div class="mt-1 flex justify-center">
                <span class="loading loading-dots loading-md"></span>
            </div>
        {:else if allRoles.length === 0}
            <p class="mt-1 text-sm text-warning">Nenhuma função disponível.</p>
        {:else}
            <div
                class="mt-2 max-h-60 overflow-y-auto space-y-2 rounded-md border border-base-content/20 bg-base-200 p-3"
            >
                {#each allRoles as role (role.id)}
                    <label
                        class="flex items-center space-x-3 p-2 rounded hover:bg-base-300 cursor-pointer"
                    >
                        <input
                            type="checkbox"
                            name="broadcast-role-checkbox"
                            value={role.id}
                            class="checkbox checkbox-primary checkbox-sm"
                            onchange={(e) =>
                                handleCheckboxChange(
                                    role.id,
                                    (e.target as HTMLInputElement).checked,
                                )}
                            disabled={isSubmitting}
                        />
                        <span class="text-sm text-base-content"
                            >{role.name}
                            {#if role.is_admin}<span
                                    class="text-xs opacity-70 ml-1"
                                    >(Admin)</span
                                >{/if}
                        </span>
                        {#if role.description}
                            <span
                                class="text-xs text-base-content/60 ml-auto truncate"
                                title={role.description}
                                >- {role.description}</span
                            >
                        {/if}
                    </label>
                {/each}
            </div>
        {/if}
        {#if errors.roles}
            <p class="mt-1 text-xs text-error">{errors.roles}</p>
        {/if}
    </div>

    <div>
        <label
            for="broadcast-message"
            class="block text-sm font-medium text-base-content">Mensagem*</label
        >
        <textarea
            id="broadcast-message"
            rows="4"
            class="textarea textarea-bordered w-full mt-1"
            class:textarea-error={errors.message}
            placeholder="Digite a sua mensagem aqui..."
            bind:value={message}
            disabled={isSubmitting}
        ></textarea>
        {#if errors.message}
            <p class="mt-1 text-xs text-error">{errors.message}</p>
        {/if}
    </div>

    <div class="flex justify-end pt-2">
        <button
            type="submit"
            class="btn btn-primary"
            disabled={isSubmitting || isLoading || allRoles.length === 0}
        >
            {#if isSubmitting}
                <span class="loading loading-spinner loading-sm"></span>
                Enviando...
            {:else}
                <i class="fa-solid fa-paper-plane mr-2"></i>
                Enviar Mensagem
            {/if}
        </button>
    </div>
</form>
