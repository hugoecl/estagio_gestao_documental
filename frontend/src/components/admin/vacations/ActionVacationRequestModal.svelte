<script lang="ts">
    import { tick } from "svelte";
    import type { VacationRequestWithUser } from "@lib/types/vacation";
    import { VacationRequestStatus as RequestStatusEnum } from "@lib/types/vacation";
    import { actionVacationRequest } from "@api/admin-vacation-api"; // This API function needs to be created
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    let {
        modalRef = $bindable(),
        request, // Type: VacationRequestWithUser | null
        onActionSuccess, // Callback when approve/reject is successful
        onClose, // Callback when modal is closed by cancel or X
    }: {
        modalRef?: HTMLDialogElement;
        request: VacationRequestWithUser | null;
        onActionSuccess: (actionedRequestId: number) => void;
        onClose: () => void;
    } = $props();

    let adminNotes = $state("");
    let isSubmitting = $state(false);
    let errors = $state<Record<string, string>>({});

    // Reactive effect to reset notes when the request prop changes
    $effect(() => {
        if (request) {
            adminNotes = ""; // Reset notes when a new request is loaded into the modal
            errors = {};
        }
    });

    function closeModal() {
        if (modalRef) modalRef.close();
        onClose(); // Notify parent that modal is closing
    }

    async function handleAction(
        statusToSet: RequestStatusEnum.Approved | RequestStatusEnum.Rejected,
    ) {
        if (!request) return;
        isSubmitting = true;
        errors = {};

        try {
            const result = await actionVacationRequest(request.id, {
                status: statusToSet,
                admin_notes: adminNotes.trim() || null,
            });

            if (result.success) {
                showAlert(
                    result.message ||
                        `Pedido ${statusToSet === RequestStatusEnum.Approved ? "aprovado" : "rejeitado"} com sucesso!`,
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                onActionSuccess(request.id); // Notify parent
                closeModal();
            } else {
                errors.general =
                    result.message ||
                    `Falha ao ${statusToSet === RequestStatusEnum.Approved ? "aprovar" : "rejeitar"} o pedido.`;
                showAlert(errors.general, AlertType.ERROR, AlertPosition.TOP);
            }
        } catch (e: any) {
            console.error(`Error actioning request ${request.id}:`, e);
            errors.general = `Erro ao processar o pedido: ${e.message}`;
            showAlert(errors.general, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isSubmitting = false;
        }
    }

    // Helper to calculate duration from the processed display data if available
    // This assumes your VacationRequestWithUser might already have pre-calculated display fields
    // or you might need to calculate it here from start_date and end_date.
    // For simplicity, using the 'durationDisplay' if passed or calculating.
    function getDurationDisplay(req: VacationRequestWithUser | null): string {
        if (!req) return "N/A";
        if ((req as any).durationDisplay) return (req as any).durationDisplay; // If already processed

        const start = new Date(req.start_date + "T00:00:00Z");
        const end = new Date(req.end_date + "T00:00:00Z");
        if (!isNaN(start.getTime()) && !isNaN(end.getTime()) && end >= start) {
            const duration =
                Math.round(
                    (end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24),
                ) + 1;
            return `${duration} dia${duration !== 1 ? "s" : ""}`;
        }
        return "Inválido";
    }
</script>

<dialog class="modal" bind:this={modalRef}>
    <div class="modal-box w-11/12 max-w-2xl">
        {#if request}
            <div class="flex justify-between items-center mb-1">
                <h3 class="font-bold text-lg">
                    Aprovar/Rejeitar Pedido de Férias #{request.id}
                </h3>
                <button
                    class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                    onclick={closeModal}
                    disabled={isSubmitting}>✕</button
                >
            </div>
            <p class="text-xs text-base-content/60 mb-4">
                Submetido por: {request.username} ({request.email})
            </p>

            <div class="space-y-4">
                <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 text-sm">
                    <div>
                        <span class="font-semibold text-base-content/80 block"
                            >Data de Início:</span
                        >
                        <span
                            >{(request as any).startDateDisplay ||
                                new Date(
                                    request.start_date + "T00:00:00Z",
                                ).toLocaleDateString("pt-PT", {
                                    timeZone: "UTC",
                                })}</span
                        >
                    </div>
                    <div>
                        <span class="font-semibold text-base-content/80 block"
                            >Data de Fim:</span
                        >
                        <span
                            >{(request as any).endDateDisplay ||
                                new Date(
                                    request.end_date + "T00:00:00Z",
                                ).toLocaleDateString("pt-PT", {
                                    timeZone: "UTC",
                                })}</span
                        >
                    </div>
                    <div>
                        <span class="font-semibold text-base-content/80 block"
                            >Duração:</span
                        >
                        <span>{getDurationDisplay(request)}</span>
                    </div>
                </div>

                {#if request.notes}
                    <div class="form-control">
                        <div class="label pt-0">
                            <span class="label-text font-medium"
                                >Notas do Utilizador:</span
                            >
                        </div>
                        <div
                            class="p-2 bg-base-200 rounded-md text-sm whitespace-pre-wrap break-words"
                        >
                            {request.notes}
                        </div>
                    </div>
                {/if}

                <label class="form-control w-full">
                    <div class="label">
                        <span class="label-text"
                            >Notas do Administrador (Opcional):</span
                        >
                    </div>
                    <textarea
                        class="textarea textarea-bordered w-full"
                        rows="3"
                        placeholder="Adicionar notas sobre a aprovação ou rejeição..."
                        bind:value={adminNotes}
                        disabled={isSubmitting}
                    ></textarea>
                </label>

                {#if errors.general}
                    <div class="alert alert-error text-xs p-2 my-2">
                        {errors.general}
                    </div>
                {/if}

                <div
                    class="modal-action mt-6 flex flex-col sm:flex-row justify-between items-center gap-2"
                >
                    <button
                        type="button"
                        class="btn btn-ghost order-3 sm:order-1 w-full sm:w-auto"
                        onclick={closeModal}
                        disabled={isSubmitting}
                    >
                        Cancelar
                    </button>
                    <div class="flex gap-2 order-1 sm:order-2 w-full sm:w-auto">
                        <button
                            type="button"
                            class="btn btn-error flex-1"
                            onclick={() =>
                                handleAction(RequestStatusEnum.Rejected)}
                            disabled={isSubmitting}
                        >
                            {#if isSubmitting && request && request.status === RequestStatusEnum.Rejected}
                                <!-- Assuming status might update -->
                                <span class="loading loading-spinner loading-sm"
                                ></span> A Rejeitar...
                            {:else}
                                <i class="fa-solid fa-times-circle mr-2"></i> Rejeitar
                                Pedido
                            {/if}
                        </button>
                        <button
                            type="button"
                            class="btn btn-success flex-1"
                            onclick={() =>
                                handleAction(RequestStatusEnum.Approved)}
                            disabled={isSubmitting}
                        >
                            {#if isSubmitting && request && request.status === RequestStatusEnum.Approved}
                                <span class="loading loading-spinner loading-sm"
                                ></span> A Aprovar...
                            {:else}
                                <i class="fa-solid fa-check-circle mr-2"></i> Aprovar Pedido
                            {/if}
                        </button>
                    </div>
    
                    <!-- Spacer -->
                </div>
            </div>
        {:else}
            <p>A carregar detalhes do pedido...</p>
            <button
                class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                onclick={closeModal}>✕</button
            >
        {/if}
    </div>
    <form method="dialog" class="modal-backdrop">
        <button onclick={closeModal} disabled={isSubmitting}>close</button>
    </form>
</dialog>
