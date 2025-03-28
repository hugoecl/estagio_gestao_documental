<script lang="ts">
    // TODO: add optional validation e.g NIF
    import DatePicker from "@components/common/DatePicker.svelte";
    import API_BASE_URL from "@api/base-url";
    import { currentModal } from "@stores/modal-store";
    import { SubmitResult, type FormField } from "@lib/types/form-modal";

    // Props definition
    let {
        formModal = $bindable(),
        title,
        fields,
        recordId,
        showFiles = false,
        files = {},
        onSubmit,
        onDelete,
        onFileDeleted,
        showDeleteButton = false,
        deleteButtonText = "Eliminar",
        submitButtonText = "Guardar",
        apiBaseUrl = API_BASE_URL,
    }: {
        formModal: HTMLDialogElement;
        title: string;
        fields: FormField[];
        recordId: string;
        showFiles?: boolean;
        files?: Record<
            string,
            { name: string; path: string; uploadedAt: string }
        >;
        onSubmit: (
            formData: Record<string, any>,
            newFiles: File[],
        ) => Promise<SubmitResult>;
        onDelete?: () => Promise<boolean>;
        onFileDeleted?: (recordId: string, fileId: string) => Promise<boolean>;
        showDeleteButton?: boolean;
        deleteButtonText?: string;
        submitButtonText?: string;
        apiBaseUrl?: string;
    } = $props();

    // State
    let modal = $state<HTMLDialogElement | null>(null);
    let confirmModal = $state<HTMLDialogElement | null>(null);
    let formValues = $state<Record<string, any>>({});
    let newFiles = $state<File[]>([]);
    let fileInput = $state<HTMLInputElement | null>(null);
    let isSubmitting = $state(false);
    let fileToDeleteId = $state<string | null>(null);
    let confirmationAction = $state<"DELETE_RECORD" | "DELETE_FILE" | null>(
        null,
    );
    let isDeleteSubmitting = $state(false);

    // Initialize form values from fields
    $effect(() => {
        for (let i = 0, len = fields.length; i < len; i++) {
            const field = fields[i];
            formValues[field.id] = field.value;
        }
    });

    // Computed values
    const existingFiles = $derived(
        showFiles && files
            ? Object.entries(files).map(([id, file]) => ({
                  id,
                  ...file,
              }))
            : null,
    );

    // Functions
    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault();
        isSubmitting = true;

        const { showAlert, AlertType, AlertPosition } = await import(
            "@components/alert/alert"
        );

        const result = await onSubmit(formValues, newFiles);

        switch (result) {
            case SubmitResult.SUCCESS:
                closeModal();
                showAlert(
                    "Dados guardados com sucesso",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                break;
            case SubmitResult.ERROR:
                showAlert(
                    "Erro ao guardar dados",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
                break;
            case SubmitResult.UNCHANGED:
                closeModal();
                showAlert(
                    "Nenhum dado alterado",
                    AlertType.INFO,
                    AlertPosition.TOP,
                );
                break;
            case SubmitResult.UNCHANGED:
                showAlert(
                    "Nenhum dado alterado",
                    AlertType.INFO,
                    AlertPosition.TOP,
                );
                break;
        }

        isSubmitting = false;
    }

    function handleFileSelection(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files) {
            newFiles = [...newFiles, ...Array.from(input.files)];
        }
    }

    function removeNewFile(index: number) {
        newFiles.splice(index, 1);
    }

    function showDeleteFileConfirmation(fileId: string) {
        fileToDeleteId = fileId;
        confirmationAction = "DELETE_FILE";
        confirmModal?.showModal();
    }

    function showDeleteRecordConfirmation() {
        confirmationAction = "DELETE_RECORD";
        confirmModal?.showModal();
    }

    async function handleDeleteConfirmed() {
        isDeleteSubmitting = true;

        try {
            if (confirmationAction === "DELETE_FILE" && fileToDeleteId) {
                await handleDeleteFile();
            } else if (confirmationAction === "DELETE_RECORD") {
                await handleDeleteRecord();
            }
        } finally {
            isDeleteSubmitting = false;
            closeConfirmationModal();
        }
    }

    async function handleDeleteFile() {
        if (!fileToDeleteId) return;

        const { showAlert, AlertType, AlertPosition } = await import(
            "@components/alert/alert"
        );

        const success = await onFileDeleted!(recordId, fileToDeleteId);

        if (success) {
            delete files[fileToDeleteId];

            showAlert(
                "Ficheiro eliminado com sucesso",
                AlertType.SUCCESS,
                AlertPosition.TOP,
            );
        } else {
            showAlert(
                "Erro ao eliminar ficheiro",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        }
    }

    async function handleDeleteRecord() {
        if (!onDelete) return;

        const { showAlert, AlertType, AlertPosition } = await import(
            "@components/alert/alert"
        );
        const success = await onDelete();

        if (success) {
            closeModal();
            showAlert(
                "Registo eliminado com sucesso",
                AlertType.SUCCESS,
                AlertPosition.TOP,
            );
        } else {
            showAlert(
                "Erro ao eliminar registo",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        }
    }

    function closeConfirmationModal() {
        confirmModal?.close();
        confirmationAction = null;
        fileToDeleteId = null;
    }

    function closeModal() {
        modal?.close();
        currentModal.set(null);
    }
</script>

<dialog class="modal z-99999999999" bind:this={modal} bind:this={formModal}>
    <div class="modal-box w-11/12 max-w-5xl">
        <div class="flex justify-between mb-4">
            <h3 class="font-bold text-xl">{title}</h3>
            <button
                class="btn btn-ghost btn-sm"
                onclick={closeModal}
                disabled={isSubmitting}>✕</button
            >
        </div>

        <form onsubmit={handleSubmit} class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                {#each fields as field}
                    <fieldset
                        class="fieldset"
                        class:md:col-span-2={field.colSpan === 2}
                    >
                        <legend class="fieldset-legend">{field.label}</legend>

                        {#if field.type === "text"}
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                placeholder={field.placeholder || ""}
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                            />
                        {:else if field.type === "number"}
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                placeholder={field.placeholder || ""}
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                            />
                        {:else if field.type === "select" && field.options}
                            <select
                                class="select select-bordered w-full"
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                            >
                                {#each field.options as option}
                                    <option value={option.value}
                                        >{option.label}</option
                                    >
                                {/each}
                            </select>
                        {:else if field.type === "date"}
                            <DatePicker
                                range={false}
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                            />
                        {:else if field.type === "dateRange"}
                            <DatePicker
                                range={true}
                                bind:value={
                                    () =>
                                        `${formValues[field.id + "Start"]} - ${formValues[field.id + "End"]}`,
                                    (value) => {
                                        const start = value.slice(0, 10);
                                        const end = value.slice(13, 23);
                                        formValues[field.id + "Start"] = start;
                                        formValues[field.id + "End"] = end;
                                    }
                                }
                            />
                        {:else if field.type === "textarea"}
                            <textarea
                                class="textarea textarea-bordered w-full"
                                placeholder={field.placeholder || ""}
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                            ></textarea>
                        {/if}
                    </fieldset>
                {/each}
            </div>

            {#if showFiles}
                <div class="divider">Ficheiros</div>

                {#if existingFiles !== null && existingFiles.length > 0}
                    <div class="overflow-x-auto">
                        <table class="table table-compact w-full">
                            <thead>
                                <tr>
                                    <th>Nome</th>
                                    <th>Data de upload</th>
                                    <th class="w-24">Ações</th>
                                </tr>
                            </thead>
                            <tbody>
                                {#each existingFiles as file}
                                    <tr>
                                        <td>{file.name}</td>
                                        <td>{file.uploadedAt}</td>
                                        <td>
                                            <div
                                                class="flex justify-end space-x-2"
                                            >
                                                <a
                                                    href={`${apiBaseUrl}/${file.path}`}
                                                    target="_blank"
                                                    class="btn btn-xs btn-outline"
                                                >
                                                    Ver
                                                </a>
                                                <button
                                                    type="button"
                                                    class="btn btn-xs btn-error"
                                                    disabled={isSubmitting}
                                                    onclick={() =>
                                                        showDeleteFileConfirmation(
                                                            file.id,
                                                        )}
                                                >
                                                    Eliminar
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                {:else}
                    <div class="text-center py-4 text-base-content/70">
                        Nenhum ficheiro associado a este registo.
                    </div>
                {/if}

                <div>
                    <div class="flex items-center justify-between">
                        <h4 class="font-semibold">Novos Ficheiros</h4>
                        <button
                            type="button"
                            class="btn btn-sm btn-secondary"
                            disabled={isSubmitting}
                            onclick={() => fileInput?.click()}
                        >
                            Adicionar Ficheiros
                        </button>
                        <input
                            type="file"
                            bind:this={fileInput}
                            onchange={handleFileSelection}
                            class="hidden"
                            multiple
                            accept="image/*,.doc,.docx,.xls,.xlsx,.pdf"
                        />
                    </div>

                    {#if newFiles.length > 0}
                        <div class="mt-2 space-y-2">
                            {#each newFiles as file, i}
                                <div
                                    class="flex items-center justify-between p-2 bg-base-200 rounded"
                                >
                                    <span class="text-sm truncate max-w-[80%]"
                                        >{file.name}</span
                                    >
                                    <button
                                        type="button"
                                        class="btn btn-xs btn-ghost"
                                        onclick={() => removeNewFile(i)}
                                    >
                                        ×
                                    </button>
                                </div>
                            {/each}
                        </div>
                    {:else}
                        <div class="text-center py-4 text-base-content/70">
                            Clique no botão acima para adicionar novos
                            ficheiros.
                        </div>
                    {/if}
                </div>
            {/if}

            <div class="modal-action flex justify-between">
                {#if showDeleteButton && onDelete}
                    <button
                        type="button"
                        class="btn btn-error"
                        onclick={showDeleteRecordConfirmation}
                        disabled={isSubmitting}
                    >
                        {deleteButtonText}
                    </button>
                {:else}
                    <div></div>
                {/if}

                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled={isSubmitting}
                >
                    {#if isSubmitting}
                        <span class="loading loading-bars loading-md"></span>
                    {:else}
                        {submitButtonText}
                    {/if}
                </button>
            </div>
        </form>
    </div>

    <form method="dialog" class="modal-backdrop">
        <button disabled={isSubmitting} onclick={closeModal}>close</button>
    </form>
</dialog>

<!-- Confirmation Modal -->
<dialog class="modal" bind:this={confirmModal}>
    <div class="modal-box">
        <div class="flex justify-between">
            <h3 class="font-bold text-lg">
                Eliminar
                {#if confirmationAction === "DELETE_FILE"}
                    Ficheiro
                {:else if confirmationAction === "DELETE_RECORD"}
                    Registo
                {/if}
            </h3>
            <button
                class="btn btn-ghost btn-sm"
                onclick={closeConfirmationModal}
                disabled={isDeleteSubmitting}
            >
                ✕
            </button>
        </div>

        <p class="py-4">
            {#if confirmationAction === "DELETE_FILE"}
                Tem certeza que deseja eliminar este ficheiro? Esta ação não
                pode ser desfeita.
            {:else if confirmationAction === "DELETE_RECORD"}
                <span class="text-error font-bold">ATENÇÃO:</span> Tem certeza que
                deseja eliminar este registo? Esta ação não pode ser desfeita e todos
                os ficheiros associados serão eliminados.
            {/if}
        </p>

        <div class="modal-action flex justify-between">
            <button
                class="btn"
                onclick={closeConfirmationModal}
                disabled={isDeleteSubmitting}
            >
                Cancelar
            </button>
            <button
                class="btn btn-error"
                onclick={handleDeleteConfirmed}
                disabled={isDeleteSubmitting}
            >
                {#if isDeleteSubmitting}
                    <span class="loading loading-bars loading-md"></span>
                {:else}
                    Sim, Eliminar
                {/if}
            </button>
        </div>
    </div>

    <form method="dialog" class="modal-backdrop">
        <button onclick={closeConfirmationModal} disabled={isDeleteSubmitting}
            >c</button
        >
    </form>
</dialog>
