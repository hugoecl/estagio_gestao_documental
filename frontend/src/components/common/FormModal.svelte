<script lang="ts">
    import DatePicker from "@components/common/DatePicker.svelte";
    import API_BASE_URL from "@api/base-url";
    import { currentModal } from "@stores/modal-store";
    import {
        FieldType,
        SubmitResult,
        type FormField,
        type SubmitResponse,
    } from "@lib/types/form-modal";
    import { DMYToDate } from "@utils/date-utils";
    import { toSearchString } from "@utils/search-utils";

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
        ) => Promise<SubmitResponse>;
        onDelete?: () => Promise<boolean>;
        onFileDeleted?: (recordId: string, fileId: string) => Promise<boolean>;
        showDeleteButton?: boolean;
        deleteButtonText?: string;
        submitButtonText?: string;
        apiBaseUrl?: string;
    } = $props();

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

    let validationErrors = $state<Record<string, string>>({});
    let showValidationErrors = $state(false);

    function validateField(field: FormField, value: any): string | null {
        if (!field.required && !value) {
            return null;
        }

        if (field.validate) {
            return field.validate(value);
        }

        return null;
    }

    function validateForm(): boolean {
        const errors: Record<string, string> = {};
        let isValid = true;

        for (let i = 0, len = fields.length; i < len; i++) {
            const field = fields[i];
            const value = formValues[field.id];

            const error = validateField(field, value);
            if (error) {
                errors[field.id] = error;
                isValid = false;
            }
        }

        validationErrors = errors;
        return isValid;
    }

    $effect(() => {
        for (let i = 0, len = fields.length; i < len; i++) {
            const field = fields[i];
            formValues[field.id] = field.value;
        }
    });

    const existingFiles = $derived(
        showFiles && files
            ? Object.entries(files).map(([id, file]) => ({
                  id,
                  ...file,
              }))
            : null,
    );

    async function handleSubmit(e: SubmitEvent) {
        e.preventDefault();

        const { showAlert, AlertType, AlertPosition } = await import(
            "@components/alert/alert"
        );

        validationErrors = {};
        const isValid = validateForm();
        if (!isValid) {
            showValidationErrors = true;
            showAlert(
                "Por favor, corrija os erros no formulário.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }

        if (!showDeleteButton && newFiles.length <= 0) {
            showAlert(
                "Por favor, submeta pelo menos um ficheiro.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }

        isSubmitting = true;

        let dateRange: string | null = null;

        if (!showDeleteButton) {
            const formEntries = new FormData(e.target as HTMLFormElement);
            for (let i = 0, len = fields.length; i < len; i++) {
                const field = fields[i];
                if (field.type === FieldType.DATE_RANGE) {
                    dateRange = formEntries.get(field.id) as string;
                    formValues[field.id] = dateRange;
                }
            }
        }

        const [result, data] = await onSubmit(formValues, newFiles);

        // logic to handle properties of records that are not directly editable e.g search properties/dates/selects
        switch (result) {
            case SubmitResult.SUCCESS:
                closeModal();

                for (let i = 0, len = fields.length; i < len; i++) {
                    const field = fields[i];
                    const value = formValues[field.id];
                    if (field.searchField) {
                        switch (field.type) {
                            case FieldType.DATE:
                                data[field.id] = DMYToDate(value);
                                data[field.searchField] = value;
                                break;

                            case FieldType.DATE_RANGE:
                                let start, end;
                                if (!showDeleteButton) {
                                    start = dateRange!.substring(0, 10);
                                    end = dateRange!.substring(13, 23);
                                } else {
                                    start = value[0];
                                    end = value[1];
                                }
                                const [first, second, firstDate, secondDate] =
                                    field.searchField.split(",");
                                data[first] = start;
                                data[second] = end;
                                data[firstDate] = DMYToDate(start);
                                data[secondDate] = DMYToDate(end);

                                break;

                            case FieldType.TEXTAREA:
                            case FieldType.TEXT:
                                data[field.searchField] = value
                                    ? toSearchString(value)
                                    : null;
                                break;

                            case FieldType.NUMBER:
                                data[field.searchField] = value.toString();
                                break;

                            case FieldType.SELECT:
                                const selectedOption = field.options!.find(
                                    (option) => option.value === value,
                                );
                                const displayField = field.id.substring(
                                    0,
                                    field.id.length - 5,
                                );

                                data[displayField] = selectedOption?.label;

                                data[field.searchField] = toSearchString(
                                    selectedOption!.label,
                                );
                                break;
                        }
                    }
                }

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
        }

        isSubmitting = false;
    }

    function handleFieldBlur(field: FormField) {
        if (!showValidationErrors) return;

        const value = formValues[field.id];
        const error = validateField(field, value);

        if (error) {
            validationErrors[field.id] = error;
        } else {
            delete validationErrors[field.id];
        }
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
        newFiles = [];
        validationErrors = {};
        showValidationErrors = false;
    }
</script>

<!-- TODO: See the z-index of the modal -->
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

                        {#if field.type === FieldType.TEXT}
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                placeholder={field.placeholder || ""}
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                                onblur={() => handleFieldBlur(field)}
                            />
                        {:else if field.type === FieldType.NUMBER}
                            <input
                                type="number"
                                class="input input-bordered w-full"
                                placeholder={field.placeholder || ""}
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                                min="0"
                                onblur={() => handleFieldBlur(field)}
                            />
                        {:else if field.type === FieldType.SELECT}
                            <select
                                class="select select-bordered w-full"
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                            >
                                <option disabled selected hidden value="">
                                    {field.label}
                                </option>
                                {#each field.options! as option}
                                    <option value={option.value}
                                        >{option.label}</option
                                    >
                                {/each}
                            </select>
                        {:else if field.type === FieldType.DATE}
                            <DatePicker
                                range={false}
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                            />
                        {:else if field.type === FieldType.DATE_RANGE}
                            {#if field.value.length > 0}
                                <DatePicker
                                    range={true}
                                    bind:value={
                                        () =>
                                            `${field.value[0]} - ${field.value[1]}`,
                                        (value) => {
                                            const start = value.slice(0, 10);
                                            const end = value.slice(13, 23);
                                            field.value[0] = start;
                                            field.value[1] = end;
                                        }
                                    }
                                />
                            {:else}
                                <DatePicker range={true} formName={field.id} />
                            {/if}
                        {:else if field.type === FieldType.TEXTAREA}
                            <textarea
                                class="textarea textarea-bordered w-full"
                                placeholder={field.placeholder || ""}
                                bind:value={formValues[field.id]}
                                required={field.required !== false}
                            ></textarea>
                        {/if}

                        {#if validationErrors[field.id]}
                            <div class="text-error text-sm mt-1">
                                {validationErrors[field.id]}
                            </div>
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
