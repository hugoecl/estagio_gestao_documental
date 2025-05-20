<script lang="ts">
    import { onMount, tick } from "svelte";
    import Table from "@components/common/Table.svelte";
    import FormModal from "@components/common/FormModal.svelte";
    import AcknowledgmentModal from "@components/common/AcknowledgmentModal.svelte"; // Import AcknowledgmentModal
    import {
        acknowledgeRecord,
        checkIfRecordAcknowledged,
    } from "@api/acknowledgment-api"; // Import acknowledgment API functions
    import type {
        CustomPageWithFields,
        UserPagePermissions,
    } from "@lib/types/custom-page";
    import type { PageField } from "@lib/types/fields";
    import type {
        PageRecord,
        CreatePageRecordRequest,
        UpdatePageRecordRequest,
        PageRecordWithFiles,
        PageRecordFile,
    } from "@lib/types/page-record";
    import type { TableColumn } from "@lib/types/table";
    import {
        getPageRecords,
        createRecord,
        updateRecord,
        deleteRecord,
        uploadRecordFiles,
        deleteRecordFile,
        getRecordById,
    } from "@api/records-api";
    import {
        FieldType as FormModalFieldType,
        SubmitResult,
        type SubmitResponse,
        type SelectOption,
        type FormField, // Import FormField type
    } from "@lib/types/form-modal";
    import { currentModal } from "@stores/modal-store";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";
    import { toSearchString } from "@utils/search-utils";
    import { DMYToDate } from "@utils/date-utils";
    import API_BASE_URL from "@api/base-url";

    const { pageDefinition }: { pageDefinition: CustomPageWithFields } =
        $props();

    let records = $state<Record<string, PageRecord>>({});
    let isLoading = $state(true);
    let searchQuery = $state("");
    let error = $state<string | null>(null);

    let formModalRef: HTMLDialogElement;
    let selectedRecordId = $state<number | null>(null);
    let selectedRecordWithFiles = $state<PageRecordWithFiles | null>(null);
    let originalRecordJson = $state<string | null>(null);
    let isModalReadOnly = $state(false); // State to control modal read-only status

    // State for Acknowledgment Modal
    let isAcknowledgmentModalOpen = $state(false);
    let recordIdToAcknowledge = $state<number | null>(null);
    let recordNameToAcknowledge = $state<string>("este registo");

    // --- Permissions ---
    const permissions = $derived<UserPagePermissions>(
        pageDefinition?.currentUserPermissions || {
            can_view: false, // Default to false if not loaded
            can_create: false,
            can_edit: false,
            can_delete: false,
            can_manage_fields: false,
            is_admin: false,
        },
    );

    // --- Table Columns ---
    const tableColumns = $derived.by(() => {
        if (!pageDefinition?.fields) return [{ header: "ID", field: "id" }];
        const cols: TableColumn[] = [{ header: "ID", field: "id" }];
        pageDefinition.fields
            .filter((field) => field.is_displayed_in_table)
            .sort((a, b) => a.order_index - b.order_index)
            .forEach((field) => {
                cols.push({
                    header: field.display_name,
                    field: `processedData.${field.name}`,
                    dateValueField:
                        field.field_type_name === "DATE" ||
                        field.field_type_name === "DATE_RANGE"
                            ? `processedData.${field.name}_date`
                            : undefined,
                });
            });
        return cols;
    });

    // --- Search Fields ---
    const searchFields = $derived.by(() => {
        if (!pageDefinition?.fields) return [];
        return pageDefinition.fields
            .filter((field) => field.is_searchable)
            .map((field) => `processedData.${field.name}_search`);
    });

    // --- Form Fields ---
    const formFields = $derived.by((): FormField[] => {
        // Explicitly type return
        if (!pageDefinition?.fields || !Array.isArray(pageDefinition.fields)) {
            return [];
        }
        return [...pageDefinition.fields]
            .sort((a, b) => a.order_index - b.order_index)
            .map(
                (pf: PageField): FormField => ({
                    // Map PageField to FormField
                    id: pf.name,
                    label: pf.display_name,
                    type: mapFieldType(pf.field_type_name),
                    required: pf.required,
                    options: pf.options ? mapOptions(pf.options) : undefined,
                    value: null, // Initial value is null, will be populated by FormModal
                    placeholder: `Insira ${pf.display_name.toLowerCase()}`,
                    validation_name: pf.validation_name, // Pass validation name
                    colSpan: pf.field_type_name === "TEXTAREA" ? 2 : 1, // Example: Make TEXTAREA span 2 cols
                }),
            );
    });

    // --- Data Fetching ---
    async function fetchRecords() {
        if (!pageDefinition?.page?.id) {
            console.warn("fetchRecords called before pageDefinition is ready.");
            isLoading = false;
            return;
        }
        isLoading = true;
        error = null;
        try {
            const rawRecords = await getPageRecords(pageDefinition.page.id);
            const processed: Record<string, PageRecord> = {};
            for (const record of rawRecords) {
                record.processedData = processRecordData(
                    record.data,
                    pageDefinition.fields,
                );
                processed[record.id.toString()] = { ...record, id: record.id };
            }
            records = processed;
        } catch (e: any) {
            console.error("Error fetching records:", e);
            error = `Erro ao carregar registos: ${e.message}`;
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoading = false;
        }
    }

    // --- Data Processing ---
    function processRecordData(
        data: Record<string, any>,
        fields: PageField[],
    ): Record<string, any> {
        if (!fields) return {};
        const processed: Record<string, any> = {};

        for (const field of fields) {
            const rawValue = data[field.name];
            let displayValue: any = rawValue ?? "";
            let searchValue: string | undefined;
            let dateValue: Date | undefined;

            const tryFormatDate = (dateString: string): string | null => {
                if (
                    typeof dateString !== "string" ||
                    !/^\d{4}-\d{2}-\d{2}/.test(dateString)
                ) {
                    return null;
                }
                try {
                    const [y, m, d] = dateString.substring(0, 10).split("-");
                    dateValue = new Date(
                        Date.UTC(parseInt(y), parseInt(m) - 1, parseInt(d)),
                    );
                    if (isNaN(dateValue.getTime())) {
                        dateValue = undefined;
                        return "Data Inválida";
                    }
                    return `${d}/${m}/${y}`;
                } catch (e) {
                    dateValue = undefined;
                    return "Data Inválida";
                }
            };

            switch (field.field_type_name) {
                case "SELECT":
                    const options = mapOptions(field.options);
                    const selectedOption = options?.find(
                        (opt) => opt.value == rawValue, // Use == for potential type coercion if needed
                    );
                    displayValue = selectedOption
                        ? selectedOption.label
                        : rawValue;
                    searchValue = selectedOption
                        ? toSearchString(selectedOption.label)
                        : rawValue
                          ? toSearchString(rawValue.toString())
                          : undefined;
                    break;

                case "DATE":
                    const formattedDate = tryFormatDate(rawValue);
                    if (formattedDate !== null) {
                        displayValue = formattedDate;
                        searchValue = displayValue;
                    } else {
                        displayValue = rawValue ?? "";
                        searchValue =
                            typeof rawValue === "number"
                                ? rawValue.toString()
                                : undefined;
                    }
                    break;

                case "DATE_RANGE":
                    if (
                        rawValue &&
                        typeof rawValue === "object" &&
                        rawValue.start &&
                        rawValue.end
                    ) {
                        const formattedStart = tryFormatDate(rawValue.start);
                        const formattedEnd = tryFormatDate(rawValue.end);
                        if (formattedStart && formattedEnd) {
                            displayValue = `${formattedStart} - ${formattedEnd}`;
                            // dateValue is set by tryFormatDate(rawValue.start)
                            searchValue = displayValue;
                        } else {
                            displayValue = "Datas Inválidas";
                        }
                    } else {
                        // Handle cases where it might be stored as a single date string
                        const formattedSingleDate = tryFormatDate(rawValue);
                        if (formattedSingleDate !== null) {
                            displayValue = formattedSingleDate;
                            searchValue = displayValue;
                        } else {
                            displayValue = rawValue ?? "";
                            searchValue = rawValue
                                ? toSearchString(rawValue.toString())
                                : undefined;
                        }
                    }
                    break;

                case "NUMBER":
                    displayValue = rawValue ?? "";
                    searchValue =
                        typeof rawValue === "number"
                            ? rawValue.toString()
                            : undefined;
                    // Example specific handling (adjust as needed)
                    if (
                        field.name === "contact_email" &&
                        typeof rawValue === "string"
                    ) {
                        searchValue = toSearchString(rawValue);
                    }
                    break;

                case "TEXT":
                case "TEXTAREA":
                default:
                    displayValue = rawValue ?? "";
                    searchValue = rawValue
                        ? toSearchString(rawValue.toString())
                        : undefined;
                    break;
            }

            processed[field.name] = displayValue;
            if (searchValue !== undefined) {
                processed[`${field.name}_search`] = searchValue;
            }
            if (dateValue !== undefined) {
                processed[`${field.name}_date`] = dateValue;
            }
        }
        return processed;
    }

    // --- Mappers ---
    function mapFieldType(backendType: string): FormModalFieldType {
        const FieldTypeMap: Record<string, FormModalFieldType> = {
            TEXT: FormModalFieldType.TEXT,
            NUMBER: FormModalFieldType.NUMBER,
            SELECT: FormModalFieldType.SELECT,
            DATE: FormModalFieldType.DATE,
            DATE_RANGE: FormModalFieldType.DATE_RANGE,
            TEXTAREA: FormModalFieldType.TEXTAREA,
        };
        return FieldTypeMap[backendType] ?? FormModalFieldType.TEXT;
    }

    function mapOptions(optionsData: any): SelectOption[] | undefined {
        // Handle options stored as ["Opt1", "Opt2"]
        if (Array.isArray(optionsData)) {
            if (optionsData.length > 0 && typeof optionsData[0] === "string") {
                return optionsData.map((opt) => ({ value: opt, label: opt }));
            }
        }
        // Handle options stored as { items: [{ value: v, label: l }, ...] } (if needed)
        else if (optionsData && Array.isArray(optionsData.items)) {
            if (
                optionsData.items.length > 0 &&
                typeof optionsData.items[0] === "object" &&
                optionsData.items[0].hasOwnProperty("value") &&
                optionsData.items[0].hasOwnProperty("label")
            ) {
                return optionsData.items as SelectOption[];
            }
        }
        console.warn("Could not parse options data:", optionsData);
        return undefined;
    }

    // --- Event Handlers ---
    async function proceedToOpenFormModal(recordIdNum: number, rowData?: PageRecord) {
        // rowData is optional now
        selectedRecordId = recordIdNum;
        
        // Set modal to read-only if user has neither edit nor add permissions
        isModalReadOnly = !permissions.can_edit && !permissions.can_add;
        
        isLoading = true;

        try {
            selectedRecordWithFiles = await getRecordById(recordIdNum);
            if (selectedRecordWithFiles) {
                originalRecordJson = JSON.stringify(
                    selectedRecordWithFiles.record.data,
                );
                await tick();
                formModalRef?.showModal();
                currentModal.set(formModalRef); // formModalRef is the dialog element
            } else {
                showAlert(
                    "Erro ao carregar detalhes do registo.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
                selectedRecordId = null;
            }
        } catch (e) {
            showAlert(
                "Erro ao carregar detalhes do registo.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            selectedRecordId = null;
        } finally {
            isLoading = false;
        }
    }

    async function handleRowClick(id: string, row: PageRecord) {
        if (!permissions.can_view && !permissions.can_edit) {
            showAlert(
                "Não tem permissão para ver detalhes deste registo.",
                AlertType.WARNING,
                AlertPosition.TOP,
            );
            return;
        }

        const recordIdNum = parseInt(id, 10);

        if (
            pageDefinition?.page?.requires_acknowledgment &&
            !permissions.is_admin
        ) {
            isLoading = true;
            try {
                const alreadyAcknowledged =
                    await checkIfRecordAcknowledged(recordIdNum);
                if (!alreadyAcknowledged) {
                    recordIdToAcknowledge = recordIdNum;
                    const primaryDisplayField = pageDefinition.fields.find(
                        (f) => f.order_index === 0 && f.is_displayed_in_table,
                    );
                    recordNameToAcknowledge =
                        primaryDisplayField &&
                        row.processedData &&
                        row.processedData[primaryDisplayField.name]
                            ? `"${row.processedData[primaryDisplayField.name]}" (ID: ${id})`
                            : `o registo #${id}`;
                    isAcknowledgmentModalOpen = true;
                    return;
                }
            } catch (e) {
                showAlert(
                    "Erro ao verificar confirmação de leitura.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            } finally {
                isLoading = false;
            }
        }
        await proceedToOpenFormModal(recordIdNum, row);
    }

    async function handleAcknowledgmentConfirm() {
        if (recordIdToAcknowledge === null) return;
        isLoading = true;
        try {
            const success = await acknowledgeRecord(recordIdToAcknowledge);
            if (success) {
                // Find the original row data to pass to proceedToOpenFormModal
                const recordData = records[recordIdToAcknowledge.toString()];
                if (recordData) {
                    await proceedToOpenFormModal(
                        recordIdToAcknowledge,
                        recordData,
                    );
                } else {
                    // Fallback if recordData not found in local state, just open by ID
                    await proceedToOpenFormModal(recordIdToAcknowledge);
                }
            } else {
                showAlert(
                    "Falha ao registar confirmação de leitura.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
        } catch (e: any) {
            showAlert(
                `Erro ao confirmar leitura: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isLoading = false;
            isAcknowledgmentModalOpen = false;
            recordIdToAcknowledge = null;
        }
    }

    async function handleCreateClick() {
        if (!permissions.can_create && !permissions.can_add) {
            showAlert(
                "Não tem permissão para criar registos nesta página.",
                AlertType.WARNING,
                AlertPosition.TOP,
            );
            return;
        }

        selectedRecordId = null;
        selectedRecordWithFiles = null;
        originalRecordJson = null;
        isModalReadOnly = false; // Always allow editing in create mode
        
        await tick();
        formModalRef?.showModal();
        currentModal.set(formModalRef);
    }

    // --- Form Submission ---
    async function handleFormSubmit(
        formData: Record<string, any>,
        newFiles: File[],
    ): Promise<SubmitResponse> {
        // If modal is read-only, prevent submission (should be disabled, but double-check)
        if (isModalReadOnly) {
            console.warn("Attempted to submit a read-only form.");
            return [SubmitResult.ERROR, null];
        }

        if (!pageDefinition?.fields) {
            showAlert(
                "Configuração de campos inválida.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return [SubmitResult.ERROR, null];
        }
        try {
            let result: boolean;
            let recordIdToUpdate: number | undefined;

            // Prepare payload, converting types as needed
            const payloadData: Record<string, any> = {};
            
            // If editing with only can_add permission (not can_edit), merge with original data
            // to avoid overwriting existing values
            const originalData = selectedRecordWithFiles?.record.data || {};
            
            for (const field of pageDefinition.fields) {
                const fieldName = field.name;
                const formValue = formData[fieldName];
                
                // Special handling for users with can_add but not can_edit
                if (permissions.can_add && !permissions.can_edit && 
                    selectedRecordId !== null && originalData[fieldName]) {
                    // Skip this field if it already has a value and user can't edit
                    payloadData[fieldName] = originalData[fieldName];
                    continue;
                }
                
                // Process the field value as before
                if (
                    field.field_type_name === "DATE" &&
                    formValue &&
                    typeof formValue === "string"
                ) {
                    try {
                        const dateObj = DMYToDate(formValue);
                        if (dateObj) {
                            const y = dateObj.getFullYear();
                            const m = String(dateObj.getMonth() + 1).padStart(2, "0");
                            const d = String(dateObj.getDate()).padStart(2, "0");
                            payloadData[fieldName] = `${y}-${m}-${d}`;
                        } else {
                            payloadData[fieldName] = null;
                        }
                    } catch (e) {
                        payloadData[fieldName] = null;
                    }
                } else if (
                    field.field_type_name === "DATE_RANGE" &&
                    formValue &&
                    typeof formValue === "object"
                ) {
                    // ... existing date range processing
                    if (formValue.start && formValue.end) {
                        try {
                            const startDateObj = DMYToDate(formValue.start);
                            const endDateObj = DMYToDate(formValue.end);
                            if (startDateObj && endDateObj) {
                                const startY = startDateObj.getFullYear();
                                const startM = String(startDateObj.getMonth() + 1).padStart(2, "0");
                                const startD = String(startDateObj.getDate()).padStart(2, "0");
                                const endY = endDateObj.getFullYear();
                                const endM = String(endDateObj.getMonth() + 1).padStart(2, "0");
                                const endD = String(endDateObj.getDate()).padStart(2, "0");
                                payloadData[fieldName] = {
                                    start: `${startY}-${startM}-${startD}`,
                                    end: `${endY}-${endM}-${endD}`,
                                };
                            } else {
                                payloadData[fieldName] = null;
                            }
                        } catch (e) {
                            payloadData[fieldName] = null;
                        }
                    } else {
                        payloadData[fieldName] = null;
                    }
                } else {
                    // Regular fields (texts, numbers, etc.)
                    payloadData[fieldName] = formValue;
                }
            }

            if (selectedRecordId !== null) {
                // Update
                const payload: UpdatePageRecordRequest = { data: payloadData };
                // Check if data or files changed
                if (
                    originalRecordJson !== null &&
                    JSON.stringify(payload.data) === originalRecordJson &&
                    newFiles.length === 0
                ) {
                    return [SubmitResult.UNCHANGED, null];
                }
                result = await updateRecord(selectedRecordId, payload);
                recordIdToUpdate = selectedRecordId;
            } else {
                // Create
                const payload: CreatePageRecordRequest = { data: payloadData };
                const createResult = await createRecord(
                    pageDefinition.page.id,
                    payload,
                );
                result = createResult.success;
                recordIdToUpdate = createResult.recordId;
            }

            // Handle result and file upload
            if (result && recordIdToUpdate !== undefined) {
                if (newFiles.length > 0) {
                    const fileUploadResult = await uploadRecordFiles(
                        recordIdToUpdate,
                        newFiles,
                    );
                    if (!fileUploadResult.success) {
                        showAlert(
                            "Registo salvo, mas ocorreu um erro ao enviar os ficheiros.",
                            AlertType.WARNING,
                            AlertPosition.TOP,
                        );
                        // Proceed considering the main operation successful
                    }
                }
                await fetchRecords(); // Refetch list data
                // Find the potentially updated/created record in the new list
                const finalRecord = records[recordIdToUpdate.toString()];
                return [SubmitResult.SUCCESS, finalRecord || {}]; // Return the updated record data
            } else {
                showAlert(
                    "Erro ao guardar o registo.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
                return [SubmitResult.ERROR, null];
            }
        } catch (e: any) {
            console.error("Error submitting form:", e);
            showAlert(
                `Erro ao guardar: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return [SubmitResult.ERROR, null];
        }
    }

    // --- Delete Handlers ---
    async function handleDeleteRecordSubmit(): Promise<boolean> {
        if (selectedRecordId === null || !permissions.can_delete) return false;
        try {
            const success = await deleteRecord(selectedRecordId);
            if (success) {
                // Optimistically remove from local state
                const updatedRecords = { ...records };
                delete updatedRecords[selectedRecordId.toString()];
                records = updatedRecords;

                selectedRecordId = null; // Reset selection
                selectedRecordWithFiles = null;
                return true;
            }
            return false;
        } catch (e: any) {
            console.error("Error deleting record:", e);
            showAlert(
                `Erro ao eliminar: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return false;
        }
    }

    async function handleFileDeleteSubmit(
        recordIdStr: string,
        fileIdStr: string,
    ): Promise<boolean> {
        // Allow file deletion only if user can edit the record
        if (!permissions.can_edit) return false;
        const recordId = parseInt(recordIdStr, 10);
        const fileId = parseInt(fileIdStr, 10);
        try {
            const success = await deleteRecordFile(recordId, fileId);
            if (success) {
                // Optimistically update local state if modal is open for this record
                if (
                    selectedRecordWithFiles &&
                    selectedRecordWithFiles.record.id === recordId
                ) {
                    const updatedFiles = selectedRecordWithFiles.files.filter(
                        (f) => f.id !== fileId,
                    );
                    selectedRecordWithFiles = {
                        ...selectedRecordWithFiles,
                        files: updatedFiles,
                    };
                }
                return true;
            }
            return false;
        } catch (e: any) {
            console.error("Error deleting file:", e);
            showAlert(
                `Erro ao eliminar ficheiro: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return false;
        }
    }

    // --- Lifecycle ---
    onMount(() => {
        if (pageDefinition?.page?.id) {
            fetchRecords();
        } else {
            console.warn(
                "DynamicRecordPage mounted, but pageDefinition not fully ready.",
            );
            // Consider setting isLoading = false here if definition is unlikely to arrive later
            isLoading = false;
            error = "Configuração da página não encontrada.";
        }
    });
</script>

<!-- Template -->
<div class="mb-4 flex flex-col sm:flex-row justify-between items-center gap-4">
    <h1 class="text-2xl font-bold">
        {pageDefinition?.page?.name || "A Carregar..."}
    </h1>
    <div class="flex gap-2 w-full sm:w-auto">
        {#if permissions.can_create}
            <button
                class="btn btn-primary flex-grow sm:flex-grow-0"
                on:click={handleCreateClick}
                disabled={!pageDefinition || isLoading}
            >
                <i class="fa-solid fa-plus mr-2"></i> Criar Novo
            </button>
        {/if}
        {#if permissions.can_manage_fields || permissions.is_admin}
            <a
                href={`/admin/pages/edit/${pageDefinition?.page?.id}/`}
                class="btn btn-secondary flex-grow sm:flex-grow-0"
                class:btn-disabled={!pageDefinition || isLoading}
            >
                <i class="fa-solid fa-wrench mr-2"></i> Gerir Página
            </a>
        {/if}
    </div>
</div>

{#if error}
    <div class="alert alert-error mb-4">
        <i class="fa-solid fa-circle-exclamation"></i>
        <span>{error}</span>
    </div>
{/if}

{#if permissions.can_create || permissions.can_add}
    <button
        class="btn btn-circle btn-lg btn-primary fixed bottom-12 right-12 shadow-lg z-10"
        on:click={handleCreateClick}
    >
        <i class="fa-solid fa-plus text-2xl"></i>
    </button>
{/if}

<div
    class="bg-base-100 rounded-lg shadow-md border border-base-content/10 overflow-hidden"
>
    <div class="p-1 md:p-5">
        {#if pageDefinition}
            <Table
                data={records}
                columns={tableColumns}
                loading={isLoading}
                emptyMessage="Nenhum registo encontrado."
                searchEmptyMessage="Nenhum registo encontrado para a sua pesquisa."
                keyField="id"
                {searchFields}
                bind:searchQuery
                onRowClick={handleRowClick}
                currentPage={1}
                perPage={10}
            />
        {:else if !error}
            <div class="flex justify-center items-center p-10">
                <span class="loading loading-lg loading-spinner"></span>
                <span class="ml-4">A carregar configuração da página...</span>
            </div>
        {/if}
    </div>
</div>

{#if pageDefinition}
    <FormModal
        bind:formModal={formModalRef}
        title={selectedRecordId
            ? `${isModalReadOnly ? "Ver" : "Editar"} Registo #${selectedRecordId}`
            : `Criar Novo Registo para ${pageDefinition.page.name}`}
        fields={formFields}
        recordId={selectedRecordId}
        recordData={selectedRecordWithFiles?.record.data}
        files={selectedRecordWithFiles?.files?.reduce(
            (acc, file) => {
                acc[file.id.toString()] = file;
                return acc;
            },
            {} as Record<string, PageRecordFile>,
        )}
        showFiles={true}
        onSubmit={handleFormSubmit}
        onDelete={handleDeleteRecordSubmit}
        onFileDeleted={handleFileDeleteSubmit}
        showDeleteButton={permissions.can_delete &&
            !!selectedRecordId &&
            !isModalReadOnly}
        submitButtonText={selectedRecordId ? "Atualizar" : "Criar"}
        apiBaseUrl={API_BASE_URL}
        readOnly={isModalReadOnly}
        currentUserPermissions={pageDefinition?.currentUserPermissions}
        pageRequiresAcknowledgment={pageDefinition?.page?.requires_acknowledgment ?? false}
    />
{/if}

{#if pageDefinition?.page?.requires_acknowledgment}
    <AcknowledgmentModal
        bind:isOpen={isAcknowledgmentModalOpen}
        recordDisplayName={recordNameToAcknowledge}
        onConfirm={handleAcknowledgmentConfirm}
        onCancel={() => {
            isAcknowledgmentModalOpen = false;
            recordIdToAcknowledge = null;
        }}
    />
{/if}
