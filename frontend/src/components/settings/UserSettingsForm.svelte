<script lang="ts">
    import { onMount, tick } from "svelte";
    import {
        getCurrentUserDetails,
        updateUserDetails,
        changePassword,
    } from "@api/user-api";
    import type {
        User,
        UpdateUserDetailsPayload,
        ChangePasswordPayload,
    } from "@lib/types/user";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    // --- State for User Details ---
    let currentUsername = $state("");
    let currentEmail = $state("");
    let newUsername = $state("");
    let newEmail = $state("");
    let currentPasswordForDetails = $state("");
    let detailsErrors = $state<Record<string, string>>({});
    let isSubmittingDetails = $state(false);
    let isLoadingUserDetails = $state(true);

    // --- State for Password Change ---
    let currentPasswordForChange = $state("");
    let newPassword = $state("");
    let confirmNewPassword = $state("");
    let passwordErrors = $state<Record<string, string>>({});
    let isSubmittingPassword = $state(false);

    onMount(async () => {
        try {
            const user = await getCurrentUserDetails();
            if (user) {
                currentUsername = user.username;
                currentEmail = user.email;
                newUsername = user.username; // Pre-fill for easier editing
                newEmail = user.email; // Pre-fill
            } else {
                showAlert(
                    "Não foi possível carregar os seus detalhes.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
                // Optionally redirect or disable form
            }
        } catch (e: any) {
            showAlert(
                `Erro ao carregar detalhes: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isLoadingUserDetails = false;
        }
    });

    function validateDetailsForm(): boolean {
        detailsErrors = {};
        if (!currentPasswordForDetails) {
            detailsErrors.currentPasswordForDetails =
                "Palavra-passe atual é obrigatória para guardar alterações.";
        }
        if (newEmail && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(newEmail)) {
            detailsErrors.newEmail = "Formato de e-mail inválido.";
        }
        if (newUsername && newUsername.trim().length === 0) {
            detailsErrors.newUsername =
                "Nome de utilizador não pode ser vazio.";
        }
        if (newEmail && newEmail.trim().length === 0) {
            detailsErrors.newEmail = "Email não pode ser vazio.";
        }

        // Check if anything actually changed besides password
        if (newUsername === currentUsername && newEmail === currentEmail) {
            if (
                Object.keys(detailsErrors).length === 0 &&
                currentPasswordForDetails
            ) {
                showAlert(
                    "Nenhuma alteração detetada no nome de utilizador ou e-mail.",
                    AlertType.INFO,
                    AlertPosition.TOP,
                );
                return false; // Not an error, but no submission needed for details
            }
        }
        return Object.keys(detailsErrors).length === 0;
    }

    async function handleUpdateDetails(e: Event) {
        e.preventDefault();
        if (!validateDetailsForm()) {
            if (Object.keys(detailsErrors).length > 0) {
                // Only show validation alert if there are actual errors
                showAlert(
                    "Por favor, corrija os erros no formulário de detalhes.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
            return;
        }
        isSubmittingDetails = true;
        const payload: UpdateUserDetailsPayload = {
            current_password: currentPasswordForDetails,
        };
        if (newUsername !== currentUsername) {
            payload.username = newUsername;
        }
        if (newEmail !== currentEmail) {
            payload.email = newEmail;
        }

        try {
            const result = await updateUserDetails(payload);
            if (result.success) {
                showAlert(
                    result.message || "Detalhes atualizados com sucesso!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                // Update current known values
                if (payload.username) currentUsername = payload.username;
                if (payload.email) currentEmail = payload.email;
                currentPasswordForDetails = ""; // Clear password field
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

    function validatePasswordForm(): boolean {
        passwordErrors = {};
        if (!currentPasswordForChange) {
            passwordErrors.currentPasswordForChange =
                "Palavra-passe atual é obrigatória.";
        }
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

    async function handleChangePassword(e: Event) {
        e.preventDefault();
        if (!validatePasswordForm()) {
            showAlert(
                "Por favor, corrija os erros no formulário de palavra-passe.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }
        isSubmittingPassword = true;
        const payload: ChangePasswordPayload = {
            current_password: currentPasswordForChange,
            new_password: newPassword,
        };

        try {
            const result = await changePassword(payload);
            if (result.success) {
                showAlert(
                    result.message || "Palavra-passe alterada com sucesso!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                // Clear password fields
                currentPasswordForChange = "";
                newPassword = "";
                confirmNewPassword = "";
            } else {
                showAlert(
                    result.message || "Falha ao alterar palavra-passe.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
        } catch (e: any) {
            showAlert(
                `Erro ao alterar palavra-passe: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmittingPassword = false;
        }
    }
</script>

{#if isLoadingUserDetails}
    <div class="flex justify-center items-center p-10">
        <span class="loading loading-lg loading-spinner text-primary"></span>
    </div>
{:else}
    <div class="space-y-10">
        <!-- User Details Form -->
        <form
            onsubmit={handleUpdateDetails}
            class="space-y-6 p-6 bg-base-200 rounded-lg shadow"
        >
            <h2
                class="text-xl font-semibold text-primary border-b border-base-content/10 pb-2 mb-6"
            >
                Alterar Detalhes da Conta
            </h2>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Nome de Utilizador Atual</span>
                </div>
                <input
                    type="text"
                    class="input input-bordered w-full bg-base-300"
                    value={currentUsername}
                    readonly
                    disabled
                />
            </label>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text"
                        >Novo Nome de Utilizador (Opcional)</span
                    >
                </div>
                <input
                    type="text"
                    placeholder="Deixe em branco para não alterar"
                    class="input input-bordered w-full"
                    bind:value={newUsername}
                    disabled={isSubmittingDetails}
                    class:input-error={detailsErrors.newUsername}
                />
                {#if detailsErrors.newUsername}
                    <span class="text-error text-xs mt-1"
                        >{detailsErrors.newUsername}</span
                    >
                {/if}
            </label>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">E-mail Atual</span>
                </div>
                <input
                    type="email"
                    class="input input-bordered w-full bg-base-300"
                    value={currentEmail}
                    readonly
                    disabled
                />
            </label>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Novo E-mail (Opcional)</span>
                </div>
                <input
                    type="email"
                    placeholder="Deixe em branco para não alterar"
                    class="input input-bordered w-full"
                    bind:value={newEmail}
                    disabled={isSubmittingDetails}
                    class:input-error={detailsErrors.newEmail}
                />
                {#if detailsErrors.newEmail}
                    <span class="text-error text-xs mt-1"
                        >{detailsErrors.newEmail}</span
                    >
                {/if}
            </label>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text"
                        >Palavra-passe Atual (para confirmar alterações)</span
                    >
                </div>
                <input
                    type="password"
                    placeholder="Digite a sua palavra-passe atual"
                    class="input input-bordered w-full"
                    bind:value={currentPasswordForDetails}
                    required
                    disabled={isSubmittingDetails}
                    class:input-error={detailsErrors.currentPasswordForDetails}
                />
                {#if detailsErrors.currentPasswordForDetails}
                    <span class="text-error text-xs mt-1"
                        >{detailsErrors.currentPasswordForDetails}</span
                    >
                {/if}
            </label>

            <div class="flex justify-end">
                <button
                    type="submit"
                    class="btn btn-primary mt-3"
                    disabled={isSubmittingDetails}
                >
                    {#if isSubmittingDetails}
                        <span class="loading loading-spinner loading-sm"></span>
                        Guardando...
                    {:else}
                        <i class="fa-solid fa-save mr-2"></i>
                        Guardar Detalhes
                    {/if}
                </button>
            </div>
        </form>

        <!-- Change Password Form -->
        <form
            onsubmit={handleChangePassword}
            class="space-y-6 p-6 bg-base-200 rounded-lg shadow"
        >
            <h2
                class="text-xl font-semibold text-primary border-b border-base-content/10 pb-2 mb-6"
            >
                Alterar Palavra-passe
            </h2>
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Palavra-passe Atual</span>
                </div>
                <input
                    type="password"
                    placeholder="Digite a sua palavra-passe atual"
                    class="input input-bordered w-full"
                    bind:value={currentPasswordForChange}
                    required
                    disabled={isSubmittingPassword}
                    class:input-error={passwordErrors.currentPasswordForChange}
                />
                {#if passwordErrors.currentPasswordForChange}
                    <span class="text-error text-xs mt-1"
                        >{passwordErrors.currentPasswordForChange}</span
                    >
                {/if}
            </label>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Nova Palavra-passe</span>
                </div>
                <input
                    type="password"
                    placeholder="Pelo menos 3 caracteres"
                    class="input input-bordered w-full"
                    bind:value={newPassword}
                    required
                    disabled={isSubmittingPassword}
                    class:input-error={passwordErrors.newPassword}
                />
                {#if passwordErrors.newPassword}
                    <span class="text-error text-xs mt-1"
                        >{passwordErrors.newPassword}</span
                    >
                {/if}
            </label>

            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Confirmar Nova Palavra-passe</span>
                </div>
                <input
                    type="password"
                    placeholder="Repita a nova palavra-passe"
                    class="input input-bordered w-full"
                    bind:value={confirmNewPassword}
                    required
                    disabled={isSubmittingPassword}
                    class:input-error={passwordErrors.confirmNewPassword}
                />
                {#if passwordErrors.confirmNewPassword}
                    <span class="text-error text-xs mt-1"
                        >{passwordErrors.confirmNewPassword}</span
                    >
                {/if}
            </label>

            <div class="flex justify-end">
                <button
                    type="submit"
                    class="btn btn-secondary mt-3"
                    disabled={isSubmittingPassword}
                >
                    {#if isSubmittingPassword}
                        <span class="loading loading-spinner loading-sm"></span>
                        Alterando...
                    {:else}
                        <i class="fa-solid fa-key mr-2"></i>
                        Alterar Palavra-passe
                    {/if}
                </button>
            </div>
        </form>
    </div>
{/if}
