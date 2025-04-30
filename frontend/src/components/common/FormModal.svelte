<script lang="ts">
    import { tick, onMount } from "svelte";
    import DatePicker from "@components/common/DatePicker.svelte";
    import { currentModal } from "@stores/modal-store";
    import {
        FieldType,
        SubmitResult,
        type FormField,
        type SubmitResponse,
        type SelectOption,
    } from "@lib/types/form-modal";
    import type { PageRecordFile } from "@lib/types/page-record";

    // --- Props ---
    let {
        formModal = $bindable(),
        title,
        fields,
        recordId = $bindable(null),
        recordData = null,
        files = null,
        showFiles = true,
        onSubmit,
        onDelete = null,
        onFileDeleted = null,
        showDeleteButton = false,
        deleteButtonText = "Eliminar",
        submitButtonText = "Guardar",
        apiBaseUrl,
        readOnly = false, // Added readOnly prop
    }: {
        formModal: HTMLDialogElement;
        title: string;
        fields: FormField[];
        recordId?: number | null;
        recordData?: Record<string, any> | null;
        files?: Record<string, PageRecordFile> | null;
        showFiles?: boolean;
        onSubmit: (
            formData: Record<string, any>,
            newFiles: File[],
        ) => Promise<SubmitResponse>;
        onDelete?: (() => Promise<boolean>) | null;
        onFileDeleted?:
            | ((recordId: string, fileId: string) => Promise<boolean>)
            | null;
        showDeleteButton?: boolean;
        deleteButtonText?: string;
        submitButtonText?: string;
        apiBaseUrl: string;
        readOnly?: boolean; // Added readOnly prop
    } = $props();

    // --- Internal State ---
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

    // --- Effects ---
    $effect(() => {
        const initialValues: Record<string, any> = {};
        if (fields && Array.isArray(fields)) {
            fields.forEach((field) => {
                if (recordData && recordData.hasOwnProperty(field.id)) {
                    let value = recordData[field.id];
                    if (
                        field.type === FieldType.DATE &&
                        value &&
                        typeof value === "string"
                    ) {
                        try {
                            const [y, m, d] = value.split("-");
                            value = `${d}/${m}/${y}`;
                        } catch {
                            value = null;
                        }
                    } else if (
                        field.type === FieldType.DATE_RANGE &&
                        value &&
                        value.start &&
                        value.end
                    ) {
                        try {
                            const [sy, sm, sd] = value.start.split("-");
                            const [ey, em, ed] = value.end.split("-");
                            value = [`${sd}/${sm}/${sy}`, `${ed}/${em}/${ey}`];
                        } catch {
                            value = [];
                        }
                    }
                    initialValues[field.id] = value;
                } else {
                    initialValues[field.id] = getDefaultFieldValue(field);
                }
            });
        }
        formValues = initialValues;
        validationErrors = {};
        showValidationErrors = false;
    });

    // --- Computed ---
    const existingFilesArray = $derived(
        showFiles && files
            ? Object.entries(files).map(([id, file]) => ({ id, ...file }))
            : [],
    );

    // --- Validation ---
    function validateField(field: FormField, value: any): string | null {
        if (readOnly) return null; // Skip validation if read-only

        if (
            field.required &&
            (value === null ||
                value === undefined ||
                value === "" ||
                (Array.isArray(value) && value.length === 0))
        ) {
            return `${field.label} é obrigatório.`;
        }
        if (field.validate) {
            return field.validate(value);
        }
        return null;
    }

    function validateForm(): boolean {
        if (readOnly) return true; // Always valid if read-only

        const errors: Record<string, string> = {};
        let isValid = true;
        if (fields && Array.isArray(fields)) {
            fields.forEach((field) => {
                const value = formValues[field.id];
                const error = validateField(field, value);
                if (error) {
                    errors[field.id] = error;
                    isValid = false;
                }
            });
        } else {
            isValid = false;
        }
        validationErrors = errors;
        return isValid;
    }

    function handleFieldBlur(field: FormField) {
        if (readOnly) return; // Don't validate on blur if read-only
        if (!showValidationErrors) return;
        const value = formValues[field.id];
        const error = validateField(field, value);
        validationErrors[field.id] = error || "";
        validationErrors = { ...validationErrors };
    }

    // --- Default Values ---
    function getDefaultFieldValue(field: FormField): any {
        switch (field.type) {
            case FieldType.NUMBER:
                return null;
            case FieldType.SELECT:
                return "";
            case FieldType.DATE:
                return null;
            case FieldType.DATE_RANGE:
                return [];
            case FieldType.TEXTAREA:
            case FieldType.TEXT:
            default:
                return "";
        }
    }

    // --- Event Handlers ---
    async function handleSubmitInternal(e: SubmitEvent) {
        e.preventDefault();
        if (readOnly) return; // Prevent submission if read-only

        const { showAlert, AlertType, AlertPosition } = await import(
            "@components/alert/alert"
        );
        const isValid = validateForm();
        showValidationErrors = true;

        if (!isValid) {
            showAlert(
                "Por favor, corrija os erros no formulário.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }

        isSubmitting = true;
        const formDataCopy = { ...formValues };
        const newFilesCopy = [...newFiles];
        const [result, updatedData] = await onSubmit(
            formDataCopy,
            newFilesCopy,
        );

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
        }
        isSubmitting = false;
    }

    function handleFileSelection(e: Event) {
        if (readOnly) return;
        const input = e.target as HTMLInputElement;
        if (input.files) {
            newFiles = [...newFiles, ...Array.from(input.files)];
            input.value = "";
        }
    }

    function removeNewFile(index: number) {
        if (readOnly) return;
        newFiles.splice(index, 1);
        newFiles = [...newFiles];
    }

    function showDeleteFileConfirmation(fileId: string) {
        if (readOnly || !onFileDeleted) return; // Prevent if read-only
        fileToDeleteId = fileId;
        confirmationAction = "DELETE_FILE";
        confirmModal?.showModal();
    }

    function showDeleteRecordConfirmation() {
        if (readOnly || !onDelete) return; // Prevent if read-only
        confirmationAction = "DELETE_RECORD";
        confirmModal?.showModal();
    }

    async function handleDeleteConfirmed() {
        if (isDeleteSubmitting || readOnly) return; // Prevent if read-only
        isDeleteSubmitting = true;
        const { showAlert, AlertType, AlertPosition } = await import(
            "@components/alert/alert"
        );
        let success = false;
        let actionText = "";

        try {
            if (
                confirmationAction === "DELETE_FILE" &&
                fileToDeleteId &&
                onFileDeleted &&
                recordId !== null
            ) {
                actionText = "ficheiro";
                success = await onFileDeleted(
                    recordId.toString(),
                    fileToDeleteId,
                );
            } else if (confirmationAction === "DELETE_RECORD" && onDelete) {
                actionText = "registo";
                success = await onDelete();
                if (success) closeModal();
            }
        } catch (e: any) {
            console.error(`Error deleting ${actionText}:`, e);
            showAlert(
                `Erro ao eliminar ${actionText}: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            success = false;
        } finally {
            if (success) {
                showAlert(
                    `${actionText.charAt(0).toUpperCase() + actionText.slice(1)} eliminado com sucesso`,
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
            } else if (actionText) {
                showAlert(
                    `Erro ao eliminar ${actionText}`,
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
            isDeleteSubmitting = false;
            closeConfirmationModal();
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

    // --- DatePicker Helpers ---
    function getDateRangeValue(fieldId: string): [string, string] | [] {
        const val = formValues[fieldId];
        return Array.isArray(val) && val.length === 2 ? val : [];
    }
    function setDateRangeValue(fieldId: string, value: string) {
        if (readOnly) return;
        const parts = value.split(" - ");
        formValues[fieldId] = parts.length === 2 ? [parts[0], parts[1]] : [];
        formValues = { ...formValues };
        handleFieldBlur(fields.find((f) => f.id === fieldId)!);
    }
    function getDateValue(fieldId: string): string | null {
        const val = formValues[fieldId];
        return typeof val === "string" ? val : null;
    }
    function setDateValue(fieldId: string, value: string | null) {
        if (readOnly) return;
        formValues[fieldId] = value;
        formValues = { ...formValues };
        handleFieldBlur(fields.find((f) => f.id === fieldId)!);
    }
</script>

<dialog class="modal z-[9999]" bind:this={modal} bind:this={formModal}>
    <div class="modal-box w-11/12 max-w-5xl">
        <div class="flex justify-between items-center mb-4">
            <h3 class="font-bold text-xl">{title}</h3>
            <button
                class="btn btn-sm btn-ghost absolute right-2 top-2"
                onclick={closeModal}
                disabled={isSubmitting}
            >
                ✕
            </button>
        </div>

        <form onsubmit={handleSubmitInternal} class="space-y-4">
            {#if !fields || fields.length === 0}
                <p class="text-center text-base-content/70 p-4">
                    Nenhum campo configurado para esta página.
                </p>
            {:else}
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {#each fields as field (field.id)}
                        <div class:md:col-span-2={field.colSpan === 2}>
                            <label class="form-control w-full">
                                <div class="label">
                                    <span class="label-text">
                                        {field.label}{field.required &&
                                        !readOnly
                                            ? "*"
                                            : ""}
                                    </span>
                                </div>

                                {#if field.type === FieldType.TEXT}
                                    <input
                                        type="text"
                                        class="input input-bordered w-full"
                                        class:input-error={!readOnly &&
                                            validationErrors[field.id]}
                                        placeholder={field.placeholder || ""}
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        onblur={() => handleFieldBlur(field)}
                                        disabled={readOnly}
                                    />
                                {:else if field.type === FieldType.NUMBER}
                                    <input
                                        type="number"
                                        step="any"
                                        class="input input-bordered w-full"
                                        class:input-error={!readOnly &&
                                            validationErrors[field.id]}
                                        placeholder={field.placeholder || ""}
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        onblur={() => handleFieldBlur(field)}
                                        disabled={readOnly}
                                    />
                                {:else if field.type === FieldType.SELECT}
                                    <select
                                        class="select select-bordered w-full"
                                        class:select-error={!readOnly &&
                                            validationErrors[field.id]}
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        onblur={() => handleFieldBlur(field)}
                                        onchange={() => handleFieldBlur(field)}
                                        disabled={readOnly}
                                    >
                                        <option disabled value="">
                                            {field.placeholder ||
                                                `Selecione ${field.label}`}
                                        </option>
                                        {#if field.options}
                                            {#each field.options as option}
                                                <option value={option.value}
                                                    >{option.label}</option
                                                >
                                            {/each}
                                        {/if}
                                    </select>
                                {:else if field.type === FieldType.DATE}
                                    <DatePicker
                                        range={false}
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        inputClass={!readOnly &&
                                        validationErrors[field.id]
                                            ? "input-error"
                                            : ""}
                                        onblur={() => handleFieldBlur(field)}
                                        onchange={() => handleFieldBlur(field)}
                                        disabled={readOnly}
                                        formName={field.id}
                                    />
                                {:else if field.type === FieldType.DATE_RANGE}
                                    <DatePicker
                                        range={true}
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        inputClass={!readOnly &&
                                        validationErrors[field.id]
                                            ? "input-error"
                                            : ""}
                                        onblur={() => handleFieldBlur(field)}
                                        onchange={() => handleFieldBlur(field)}
                                        disabled={readOnly}
                                        formName={field.id}
                                    />
                                {:else if field.type === FieldType.TEXTAREA}
                                    <textarea
                                        class="textarea textarea-bordered w-full min-h-24"
                                        class:textarea-error={!readOnly &&
                                            validationErrors[field.id]}
                                        placeholder={field.placeholder || ""}
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        onblur={() => handleFieldBlur(field)}
                                        disabled={readOnly}
                                    ></textarea>
                                {/if}

                                {#if !readOnly && validationErrors[field.id]}
                                    <div class="label">
                                        <span class="label-text-alt text-error"
                                            >{validationErrors[field.id]}</span
                                        >
                                    </div>
                                {/if}
                            </label>
                        </div>
                    {/each}
                </div>
            {/if}

            {#if showFiles}
                <div class="divider mt-6 mb-2">Ficheiros</div>

                {#if existingFilesArray.length > 0}
                    <div class="overflow-x-auto max-h-60 border rounded-md">
                        <table class="table table-pin-rows table-xs w-full">
                            <thead>
                                <tr>
                                    <th>Nome</th>
                                    <th>Data Upload</th>
                                    <th class="w-24 text-right">Ações</th>
                                </tr>
                            </thead>
                            <tbody>
                                {#each existingFilesArray as file (file.id)}
                                    <tr>
                                        <td
                                            class="max-w-xs truncate"
                                            title={file.file_name}
                                            >{file.file_name}</td
                                        >
                                        <td
                                            >{new Date(
                                                file.uploaded_at,
                                            ).toLocaleString("pt-PT")}</td
                                        >
                                        <td>
                                            <div
                                                class="flex justify-end space-x-1"
                                            >
                                                <a
                                                    href={`${apiBaseUrl}/${file.file_path}`}
                                                    target="_blank"
                                                    class="btn btn-xs btn-ghost btn-square"
                                                    title="Ver Ficheiro"
                                                >
                                                    <i class="fa-solid fa-eye"
                                                    ></i>
                                                </a>
                                                {#if onFileDeleted && recordId !== null && !readOnly}
                                                    <button
                                                        type="button"
                                                        class="btn btn-xs btn-ghost btn-square text-error"
                                                        title="Eliminar Ficheiro"
                                                        disabled={isSubmitting ||
                                                            isDeleteSubmitting}
                                                        onclick={() =>
                                                            showDeleteFileConfirmation(
                                                                file.id,
                                                            )}
                                                    >
                                                        <i
                                                            class="fa-solid fa-trash"
                                                        ></i>
                                                    </button>
                                                {/if}
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

                {#if !readOnly}
                    <div class="mt-4">
                        <div class="flex items-center justify-between">
                            <h4 class="font-semibold text-sm">
                                Adicionar Ficheiros
                            </h4>
                            <button
                                type="button"
                                class="btn btn-sm btn-outline btn-secondary"
                                disabled={isSubmitting}
                                onclick={() => fileInput?.click()}
                            >
                                <i class="fa-solid fa-upload mr-1"></i> Selecionar
                            </button>
                            <input
                                type="file"
                                bind:this={fileInput}
                                onchange={handleFileSelection}
                                class="hidden"
                                multiple
                                accept="image/*,.doc,.docx,.xls,.xlsx,.pdf,.txt,.csv,.zip,.rar"
                                disabled={readOnly}
                            />
                        </div>

                        {#if newFiles.length > 0}
                            <div
                                class="mt-2 space-y-1 max-h-40 overflow-y-auto border rounded-md p-2 bg-base-200"
                            >
                                {#each newFiles as file, i (file.name + file.lastModified)}
                                    <div
                                        class="flex items-center justify-between p-1 bg-base-100 rounded text-xs"
                                    >
                                        <span
                                            class="truncate max-w-[85%]"
                                            title={file.name}>{file.name}</span
                                        >
                                        <button
                                            type="button"
                                            class="btn btn-xs btn-ghost btn-square text-error"
                                            title="Remover"
                                            onclick={() => removeNewFile(i)}
                                            disabled={readOnly}
                                        >
                                            ✕
                                        </button>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/if}
            {/if}

            <div
                class="modal-action flex flex-col-reverse sm:flex-row justify-between pt-4 mt-4 border-t"
            >
                <div>
                    {#if showDeleteButton && onDelete && recordId !== null && !readOnly}
                        <button
                            type="button"
                            class="btn btn-error w-full sm:w-auto"
                            onclick={showDeleteRecordConfirmation}
                            disabled={isSubmitting || isDeleteSubmitting}
                        >
                            {deleteButtonText}
                        </button>
                    {/if}
                </div>
                <div
                    class="flex flex-col-reverse sm:flex-row gap-2 w-full sm:w-auto"
                >
                    <button
                        type="button"
                        class="btn btn-ghost w-full sm:w-auto"
                        onclick={closeModal}
                        disabled={isSubmitting}
                    >
                        {readOnly ? "Fechar" : "Cancelar"}
                    </button>
                    {#if !readOnly}
                        <button
                            type="submit"
                            class="btn btn-primary w-full sm:w-auto"
                            disabled={isSubmitting ||
                                !fields ||
                                fields.length === 0}
                        >
                            {#if isSubmitting}
                                <span class="loading loading-spinner loading-sm"
                                ></span> Guardando...
                            {:else}
                                {submitButtonText}
                            {/if}
                        </button>
                    {/if}
                </div>
            </div>
        </form>
    </div>

    <form method="dialog" class="modal-backdrop">
        <button
            disabled={isSubmitting || isDeleteSubmitting}
            onclick={closeModal}>close</button
        >
    </form>
</dialog>

<!-- Confirmation Modal -->
<dialog class="modal z-[10000]" bind:this={confirmModal}>
    <div class="modal-box">
        <h3 class="font-bold text-lg mb-4">
            Confirmar Eliminação de {confirmationAction === "DELETE_FILE"
                ? "Ficheiro"
                : "Registo"}
        </h3>
        <p class="py-4">
            {#if confirmationAction === "DELETE_FILE"}
                Tem a certeza que deseja eliminar este ficheiro? Esta ação não
                pode ser desfeita.
            {:else if confirmationAction === "DELETE_RECORD"}
                <span class="text-error font-bold">ATENÇÃO:</span> Tem a certeza
                que deseja eliminar este registo? Todos os ficheiros associados também
                serão eliminados. Esta ação não pode ser desfeita.
            {/if}
        </p>
        <div class="modal-action flex justify-end gap-2">
            <button
                class="btn btn-ghost"
                onclick={closeConfirmationModal}
                disabled={isDeleteSubmitting}>Cancelar</button
            >
            <button
                class="btn btn-error"
                onclick={handleDeleteConfirmed}
                disabled={isDeleteSubmitting}
            >
                {#if isDeleteSubmitting}
                    <span class="loading loading-spinner loading-sm"></span> Eliminando...
                {:else}
                    Sim, Eliminar
                {/if}
            </button>
        </div>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button onclick={closeConfirmationModal} disabled={isDeleteSubmitting}
            >close</button
        >
    </form>
</dialog>
