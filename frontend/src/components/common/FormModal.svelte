<script lang="ts">
    import { tick, onMount } from "svelte"; // Add $effect
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
    import type { UserPagePermissions } from "@lib/types/custom-page"; // Import UserPagePermissions
    import { validateNIF } from "@utils/nif"; // Import NIF validator

    // --- Email Validation (Simple Regex Example) ---
    function validateEmail(email: string): string | null {
        // ... (validation function remains)
        if (!email) return null;
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        if (!emailRegex.test(email)) {
            return "Formato de e-mail inválido.";
        }
        return null;
    }

    // --- Validation Function Map ---
    const validationFunctions: Record<string, (value: any) => string | null> = {
        // ... (validation map remains) ...
        nif: (value) =>
            validateNIF(String(value ?? "")) ? null : "NIF inválido.",
        email: (value) => validateEmail(String(value ?? "")),
    };

    // --- Props ---
    let {
        formModal = $bindable(),
        title,
        fields,
        recordId = $bindable(null),
        recordData = null, // This prop changing triggers the $effect
        files = null,
        showFiles = true,
        onSubmit,
        onDelete = null,
        onFileDeleted = null,
        showDeleteButton = false,
        deleteButtonText = "Eliminar",
        submitButtonText = "Guardar",
        apiBaseUrl,
        readOnly = false,
        currentUserPermissions = null,
        pageRequiresAcknowledgment = false, // Added prop
    }: {
        formModal: HTMLDialogElement;
        title: string;
        fields: FormField[];
        recordId?: number | null;
        recordData?: Record<string, any> | null; // Watch this prop
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
        readOnly?: boolean;
        currentUserPermissions?: UserPagePermissions | null;
        pageRequiresAcknowledgment?: boolean; // Added prop type
    } = $props();

    // --- Internal State ---
    let modal = $state<HTMLDialogElement | null>(null);
    let confirmModal = $state<HTMLDialogElement | null>(null);
    let formValues = $state<Record<string, any>>({}); // This will be set by the effect
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

    // --- **UPDATED Effect to Initialize/Reset Form Values** ---
    $effect(() => {
        const initialValues: Record<string, any> = {};
        if (fields && Array.isArray(fields)) {
            fields.forEach((field) => {
                let valueToSet: any = null; // Default to null (important for reset)

                if (recordData && recordData.hasOwnProperty(field.id)) {
                    // If editing, try to get the value
                    const rawValue = recordData[field.id];
                    if (
                        field.type === FieldType.DATE &&
                        rawValue &&
                        typeof rawValue === "string"
                    ) {
                        // Format DATE from YYYY-MM-DD to DD/MM/YYYY
                        try {
                            const [y, m, d] = rawValue.split("-");
                            if (y && m && d) valueToSet = `${d}/${m}/${y}`;
                            else valueToSet = null; // Invalid format from backend
                        } catch {
                            valueToSet = null;
                        }
                    } else if (
                        field.type === FieldType.DATE_RANGE &&
                        rawValue &&
                        rawValue.start &&
                        rawValue.end
                    ) {
                        // Format DATE_RANGE from {start: YYYY-MM-DD, end: YYYY-MM-DD} to [DD/MM/YYYY, DD/MM/YYYY]
                        try {
                            const [sy, sm, sd] = rawValue.start.split("-");
                            const [ey, em, ed] = rawValue.end.split("-");
                            if (sy && sm && sd && ey && em && ed) {
                                valueToSet = [
                                    `${sd}/${sm}/${sy}`,
                                    `${ed}/${em}/${ey}`,
                                ];
                            } else {
                                valueToSet = null;
                            } // Invalid format
                        } catch {
                            valueToSet = null;
                        }
                    } else {
                        // For other types, use the raw value (or null if it's undefined/null)
                        valueToSet = rawValue ?? null;
                    }
                } else {
                    // If creating (recordData is null) or field doesn't exist in data, use default
                    valueToSet = getDefaultFieldValue(field);
                }
                initialValues[field.id] = valueToSet;
            });
        }
        formValues = initialValues; // Set the state *once* after processing all fields
        validationErrors = {}; // Reset errors when data changes
        showValidationErrors = false;
    });
    // --- **End UPDATED Effect** ---

    // --- Computed ---
    const existingFilesArray = $derived(
        showFiles && files
            ? Object.entries(files).map(([id, file]) => ({ id, ...file }))
            : [],
    );

    // --- Validation ---
    function validateField(field: FormField, value: any): string | null {
        if (readOnly) return null;
        
        // Special handling for DATE fields
        if (field.type === FieldType.DATE) {
            // Only validate if the field is required
            if (field.required) {
                // Check if the value is empty or null
                if (!value) {
                    return `${field.label} é obrigatório.`;
                }
                
            }
            return null;
        }
        
        // Handle DATE_RANGE fields
        if (field.type === FieldType.DATE_RANGE) {
            if (field.required) {
                // For date ranges, just check if we have an array with two valid date strings
                if (!value) {
                    return `${field.label} é obrigatório.`;
                }
                
                const [startDate, endDate] = value;
                
            }
            return null;
        }
        
        // Regular validation for other field types
        if (
            field.required &&
            (value === null ||
                value === undefined ||
                value === "" ||
                (Array.isArray(value) && value.length === 0))
        ) {
            return `${field.label} é obrigatório.`;
        }
        if (
            field.validation_name &&
            value !== null &&
            value !== undefined &&
            value !== ""
        ) {
            const validator = validationFunctions[field.validation_name];
            if (validator) {
                const errorMsg = validator(value);
                if (errorMsg) {
                    return errorMsg;
                }
            } else {
                console.warn(
                    `Validation function named "${field.validation_name}" not found.`,
                );
            }
        }
        if (field.validate) {
            return field.validate(value);
        }
        return null;
    }

    function validateForm(): boolean {
        // ... (validateForm logic remains the same) ...
        if (readOnly) return true;
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
        // ... (handleFieldBlur logic remains the same) ...
        if (readOnly) return;
        const value = formValues[field.id];
        const error = validateField(field, value);
        validationErrors[field.id] = error || "";
        validationErrors = { ...validationErrors };
    }

    // --- Default Values ---
    function getDefaultFieldValue(field: FormField): any {
        // ... (getDefaultFieldValue remains the same) ...
        switch (field.type) {
            case FieldType.NUMBER:
                return null;
            case FieldType.SELECT:
                return "";
            case FieldType.DATE:
                return null;
            case FieldType.DATE_RANGE:
                return null;
            case FieldType.TEXTAREA:
            case FieldType.TEXT:
            default:
                return "";
        }
    }

    // --- Event Handlers ---
    // ... (handleSubmitInternal, handleFileSelection, removeNewFile, delete confirmations, closeModal remain the same) ...
    async function handleSubmitInternal(e: SubmitEvent) {
        e.preventDefault();
        if (readOnly) return;
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
        if (readOnly || !onFileDeleted) return;
        fileToDeleteId = fileId;
        confirmationAction = "DELETE_FILE";
        confirmModal?.showModal();
    }
    function showDeleteRecordConfirmation() {
        if (readOnly || !onDelete) return;
        confirmationAction = "DELETE_RECORD";
        confirmModal?.showModal();
    }
    async function handleDeleteConfirmed() {
        /* ... remains same ... */ if (isDeleteSubmitting || readOnly) return;
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
        showValidationErrors = false; /* Reset formValues on close? Maybe not necessary if $effect handles it well */ /* formValues = {}; */
    }

    // Function to determine if a field should be disabled based on permissions and content
    function isFieldDisabled(fieldId: string): boolean {
        if (readOnly) return true; // Fully read-only mode
        
        // If user has edit permission, field is always enabled
        if (!currentUserPermissions || currentUserPermissions.can_edit) return false;
        
        // If creating a new record (no recordId) and user has can_add or can_create permission, enable fields
        if (recordId === null && (currentUserPermissions.can_add || currentUserPermissions.can_create)) {
            return false;
        }
        
        // If user has add permission but NOT edit permission, field is disabled only if it has a value
        if (currentUserPermissions.can_add && recordData) {
            const hasValue = recordData[fieldId] !== null && 
                           recordData[fieldId] !== undefined && 
                           recordData[fieldId] !== '';
            return hasValue; // Disable if field has value
        }
        
        // In all other cases, field is disabled
        return true;
    }
</script>

<!-- Template -->
<dialog class="modal z-[9999]" bind:this={modal} bind:this={formModal}>
    <div class="modal-box w-11/12 max-w-5xl">
        <div class="flex justify-between items-center mb-4">
            <h3 class="font-bold text-xl">{title}</h3>
            <button
                class="btn btn-sm btn-ghost absolute right-2 top-2"
                onclick={closeModal}
                disabled={isSubmitting}>✕</button
            >
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
                                    <span class="label-text"
                                        >{field.label}{field.required &&
                                        !readOnly
                                            ? "*"
                                            : ""}</span
                                    >
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
                                        disabled={isFieldDisabled(field.id)}
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
                                        disabled={isFieldDisabled(field.id)}
                                    />
                                {:else if field.type === FieldType.SELECT}
                                    <select
                                        class="select select-bordered w-full"
                                        class:select-error={!readOnly &&
                                            validationErrors[field.id]}
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        onblur={() => handleFieldBlur(field)}
                                        disabled={isFieldDisabled(field.id)}
                                    >
                                        <option value="">Selecione...</option>
                                        {#each field.options || [] as option}
                                            <option value={option.value}>{option.label}</option>
                                        {/each}
                                    </select>
                                {:else if field.type === FieldType.DATE}
                                    <DatePicker
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        inputClass={!readOnly && validationErrors[field.id] ? "input-error" : ""}
                                        onblur={() => handleFieldBlur(field)}
                                        onchange={() => handleFieldBlur(field)}
                                        disabled={isFieldDisabled(field.id)}
                                        formName={field.id}
                                    />
                                {:else if field.type === FieldType.DATE_RANGE}
                                    <DatePicker
                                        range={true}
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        inputClass={!readOnly && validationErrors[field.id] ? "input-error" : ""}
                                        onblur={() => handleFieldBlur(field)}
                                        onchange={() => handleFieldBlur(field)}
                                        disabled={isFieldDisabled(field.id)}
                                        formName={field.id}
                                    />
                                {:else if field.type === FieldType.TEXTAREA}
                                    <textarea
                                        class="textarea textarea-bordered w-full h-32"
                                        class:textarea-error={!readOnly &&
                                            validationErrors[field.id]}
                                        placeholder={field.placeholder || ""}
                                        bind:value={formValues[field.id]}
                                        required={field.required}
                                        onblur={() => handleFieldBlur(field)}
                                        disabled={isFieldDisabled(field.id)}
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

            <!-- Files Section -->
            {#if showFiles}
                <div class="divider mt-6 mb-2">Ficheiros</div>
                {#if existingFilesArray.length > 0}
                    <div class="overflow-x-auto max-h-60 border rounded-md">
                        <table class="table table-pin-rows table-xs w-full">
                            <thead>
                                <tr>
                                    <th>Nome</th> <th>Data Upload</th>
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

            <!-- Actions -->
            <div
                class="modal-action flex flex-col-reverse sm:flex-row justify-between items-center pt-4 mt-4 border-t"
            >
                <div class="flex gap-2">
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

                    {#if recordId !== null && pageRequiresAcknowledgment && currentUserPermissions && (currentUserPermissions.is_admin || currentUserPermissions.can_view_acknowledgments) && fields.length > 0}
                        <a
                            href={`/admin/records/${recordId}/acknowledgments/`}
                            class="btn btn-accent w-full sm:w-auto"
                        >
                            <i class="fa-solid fa-list-check mr-1"></i> Ver Confirmações
                        </a>
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
                                ></span> A Guardar...
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
                    <span class="loading loading-spinner loading-sm"></span> A Apagar...
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
