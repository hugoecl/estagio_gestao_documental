<script lang="ts">
    import { onMount, tick } from "svelte";
    import Table from "@components/common/Table.svelte";
    import FormModal from "@components/common/FormModal.svelte";
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
    } from "@lib/types/form-modal"; // Added SelectOption
    import { currentModal } from "@stores/modal-store";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";
    import { toSearchString } from "@utils/search-utils";
    import { DMYToDate } from "@utils/date-utils"; // Keep this for potential sorting
    import API_BASE_URL from "@api/base-url";

    // ... (Props and other state variables remain the same) ...
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

    // --- Permissions (remains the same) ---
    const permissions = $derived<UserPagePermissions>(
        pageDefinition?.currentUserPermissions || {
            can_view: true,
            can_create: false,
            can_edit: false,
            can_delete: false,
            can_manage_fields: false,
            is_admin: false,
        },
    );

    // --- Table Columns (remains the same) ---
    const tableColumns = $derived.by(() => {
        if (!pageDefinition?.fields) return [{ header: "ID", field: "id" }];
        const cols: TableColumn[] = [{ header: "ID", field: "id" }];
        pageDefinition.fields
            .filter((field) => field.is_displayed_in_table)
            .sort((a, b) => a.order_index - b.order_index)
            .forEach((field) => {
                cols.push({
                    header: field.display_name,
                    field: `processedData.${field.name}`, // Access processed data
                    dateValueField:
                        field.field_type_name === "DATE" ||
                        field.field_type_name === "DATE_RANGE"
                            ? `processedData.${field.name}_date` // For sorting
                            : undefined,
                });
            });
        return cols;
    });

    // --- Search Fields (remains the same) ---
    const searchFields = $derived.by(() => {
        if (!pageDefinition?.fields) return [];
        return pageDefinition.fields
            .filter((field) => field.is_searchable)
            .map((field) => `processedData.${field.name}_search`);
    });

    // --- Form Fields (remains the same) ---
    const formFields = $derived.by(() => {
        if (!pageDefinition?.fields || !Array.isArray(pageDefinition.fields)) {
            return [];
        }
        return [...pageDefinition.fields]
            .sort((a, b) => a.order_index - b.order_index)
            .map((pf) => ({
                id: pf.name,
                label: pf.display_name,
                type: mapFieldType(pf.field_type_name),
                required: pf.required,
                options: pf.options ? mapOptions(pf.options) : undefined,
                value: null,
                placeholder: `Insira ${pf.display_name.toLowerCase()}`,
            }));
    });

    // --- Data Fetching (remains the same) ---
    async function fetchRecords(query?: string) {
        if (!pageDefinition?.page?.id) {
            console.warn("fetchRecords called before pageDefinition is ready.");
            isLoading = false;
            return;
        }
        isLoading = true;
        error = null;
        try {
            const rawRecords = await getPageRecords(
                pageDefinition.page.id,
                query,
            );
            const processed: Record<string, PageRecord> = {};
            for (const record of rawRecords) {
                // Pass the specific field definition to processRecordData
                record.processedData = processRecordData(
                    record.data,
                    pageDefinition.fields,
                );
                processed[record.id.toString()] = { ...record, id: record.id };
            }
            console.log("DynamicRecordPage: Setting records state:", processed);
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
            let dateValue: Date | undefined; // For sorting

            // --- Robust Date Formatting ---
            const tryFormatDate = (dateString: string): string | null => {
                if (
                    typeof dateString !== "string" ||
                    !/^\d{4}-\d{2}-\d{2}/.test(dateString)
                ) {
                    return null; // Not a valid YYYY-MM-DD string
                }
                try {
                    const [y, m, d] = dateString.substring(0, 10).split("-");
                    // Create Date object for sorting (use UTC to avoid timezone issues)
                    dateValue = new Date(
                        Date.UTC(parseInt(y), parseInt(m) - 1, parseInt(d)),
                    );
                    if (isNaN(dateValue.getTime())) {
                        // Check if date is valid
                        dateValue = undefined;
                        return "Data Inválida";
                    }
                    return `${d}/${m}/${y}`; // Format as DD/MM/YYYY
                } catch (e) {
                    dateValue = undefined;
                    return "Data Inválida";
                }
            };

            switch (field.field_type_name) {
                case "SELECT":
                    const options = mapOptions(field.options); // Use updated mapOptions
                    const selectedOption = options?.find(
                        (opt) => opt.value == rawValue,
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

                case "DATE": // Field definition says DATE
                    const formattedDate = tryFormatDate(rawValue);
                    if (formattedDate !== null) {
                        displayValue = formattedDate;
                        searchValue = displayValue;
                    } else {
                        // If it wasn't a date string, display raw value (handles contract_value case)
                        displayValue = rawValue ?? "";
                        searchValue =
                            typeof rawValue === "number"
                                ? rawValue.toString()
                                : undefined;
                    }
                    break;

                case "DATE_RANGE": // Field definition says DATE_RANGE
                    // Check if data is object {start, end}
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
                            // Use start date for sorting
                            // dateValue is already set by tryFormatDate(rawValue.start)
                            searchValue = displayValue;
                        } else {
                            displayValue = "Datas Inválidas";
                        }
                    }
                    // Check if data is single date string (like contract_date)
                    else {
                        const formattedSingleDate = tryFormatDate(rawValue);
                        if (formattedSingleDate !== null) {
                            displayValue = formattedSingleDate;
                            searchValue = displayValue;
                            // dateValue is set by tryFormatDate
                        } else {
                            // Otherwise display raw
                            displayValue = rawValue ?? "";
                            searchValue = rawValue
                                ? toSearchString(rawValue.toString())
                                : undefined;
                        }
                    }
                    break;

                case "NUMBER": // Field definition says NUMBER
                    // Display raw value, ensure it's treated as number if possible
                    displayValue = rawValue ?? "";
                    searchValue =
                        typeof rawValue === "number"
                            ? rawValue.toString()
                            : undefined;
                    // Handle contact_email case where definition is wrong
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
                processed[`${field.name}_date`] = dateValue; // Store Date object for sorting
            }
        }
        return processed;
    }

    // --- Mappers ---
    function mapFieldType(backendType: string): FormModalFieldType {
        // ... (remains the same) ...
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

    // Updated mapOptions to handle the { items: [...] } structure
    function mapOptions(optionsData: any): SelectOption[] | undefined {
        if (optionsData && Array.isArray(optionsData.items)) {
            // Check if items have label and value properties
            if (
                optionsData.items.length > 0 &&
                typeof optionsData.items[0] === "object" &&
                optionsData.items[0].hasOwnProperty("value") &&
                optionsData.items[0].hasOwnProperty("label")
            ) {
                return optionsData.items as SelectOption[];
            }
        }
        // Fallback for simple string array (if needed in other cases)
        else if (Array.isArray(optionsData)) {
            if (optionsData.length > 0 && typeof optionsData[0] === "string") {
                return optionsData.map((opt) => ({ value: opt, label: opt }));
            }
        }
        console.warn("Could not parse options data:", optionsData);
        return undefined;
    }

    // --- Event Handlers (handleRowClick, handleCreateClick remain the same) ---
    async function handleRowClick(id: string, row: PageRecord) {
        if (!permissions.can_edit) {
            showAlert(
                "Não tem permissão para editar este registo.",
                AlertType.WARNING,
                AlertPosition.TOP,
            );
            return;
        }
        const recordIdNum = parseInt(id, 10);
        selectedRecordId = recordIdNum;
        isLoading = true;
        try {
            selectedRecordWithFiles = await getRecordById(recordIdNum);
            if (selectedRecordWithFiles) {
                originalRecordJson = JSON.stringify(
                    selectedRecordWithFiles.record.data,
                );
                formModalRef?.showModal();
                currentModal.set(formModalRef?.children[0] as HTMLDivElement);
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

    function handleCreateClick() {
        if (!permissions.can_create) {
            showAlert(
                "Não tem permissão para criar registos.",
                AlertType.WARNING,
                AlertPosition.TOP,
            );
            return;
        }
        selectedRecordId = null;
        selectedRecordWithFiles = null;
        originalRecordJson = JSON.stringify({});
        formModalRef?.showModal();
        currentModal.set(formModalRef?.children[0] as HTMLDivElement);
    }

    // --- Form Submission ---
    async function handleFormSubmit(
        formData: Record<string, any>,
        newFiles: File[],
    ): Promise<SubmitResponse> {
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

            const payloadData: Record<string, any> = {};
            pageDefinition.fields.forEach((field) => {
                if (formData.hasOwnProperty(field.name)) {
                    let value = formData[field.name];

                    // --- Convert back to backend format based on INTENDED type ---
                    // Use field_type_name from definition to decide conversion
                    const intendedType = field.field_type_name;

                    if (intendedType === "NUMBER") {
                        if (typeof value === "string" && value !== "") {
                            value = parseFloat(value);
                            if (isNaN(value)) value = null;
                        } else if (value === "") {
                            value = null;
                        }
                        // Ensure it's a number or null
                        value =
                            typeof value === "number" && !isNaN(value)
                                ? value
                                : null;
                    } else if (
                        intendedType === "DATE_RANGE" &&
                        Array.isArray(value)
                    ) {
                        const [startStr, endStr] = value;
                        if (startStr && endStr) {
                            const formatDate = (dmy: string) => {
                                if (!dmy || !/^\d{2}\/\d{2}\/\d{4}$/.test(dmy))
                                    return null;
                                const [d, m, y] = dmy.split("/");
                                return `${y}-${m}-${d}`;
                            };
                            const start = formatDate(startStr);
                            const end = formatDate(endStr);
                            value = start && end ? { start, end } : null;
                        } else {
                            value = null;
                        }
                    } else if (
                        intendedType === "DATE" &&
                        typeof value === "string"
                    ) {
                        if (value && /^\d{2}\/\d{2}\/\d{4}$/.test(value)) {
                            const [d, m, y] = value.split("/");
                            value = `${y}-${m}-${d}`;
                        } else {
                            value = null;
                        }
                    } else if (
                        value === "" &&
                        intendedType !== "TEXT" &&
                        intendedType !== "TEXTAREA"
                    ) {
                        // Treat empty strings as null for non-text fields
                        value = null;
                    }
                    payloadData[field.name] = value;
                }
            });

            if (selectedRecordId !== null) {
                // Update
                const payload: UpdatePageRecordRequest = { data: payloadData };
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
                    }
                }
                await fetchRecords(searchQuery); // Refetch list
                const updatedRecord = records[recordIdToUpdate];
                return [SubmitResult.SUCCESS, updatedRecord || {}];
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

    // --- Delete Handlers (remain the same) ---
    async function handleDeleteRecordSubmit(): Promise<boolean> {
        if (selectedRecordId === null || !permissions.can_delete) return false;
        try {
            const success = await deleteRecord(selectedRecordId);
            if (success) {
                const updatedRecords = { ...records };
                delete updatedRecords[selectedRecordId];
                records = updatedRecords;

                selectedRecordId = null;
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
        if (!permissions.can_edit) return false;
        const recordId = parseInt(recordIdStr, 10);
        const fileId = parseInt(fileIdStr, 10);
        try {
            const success = await deleteRecordFile(recordId, fileId);
            if (success) {
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

    // --- Lifecycle (remains the same) ---
    onMount(() => {
        if (pageDefinition?.page?.id) {
            fetchRecords();
        } else {
            console.warn(
                "DynamicRecordPage mounted, but pageDefinition not fully ready.",
            );
        }
    });

    let debounceTimer: number;
    $effect(() => {
        const currentQuery = searchQuery;
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
            if (pageDefinition?.page?.id) {
                fetchRecords(currentQuery);
            }
        }, 300);
        return () => clearTimeout(debounceTimer);
    });
</script>

<!-- Template (remains the same) -->
<div class="mb-4 flex flex-col sm:flex-row justify-between items-center gap-4">
    <h1 class="text-2xl font-bold">
        {pageDefinition?.page?.name || "A Carregar..."}
    </h1>
    <div class="flex gap-2 w-full sm:w-auto">
        {#if permissions.can_create}
            <button
                class="btn btn-primary flex-grow sm:flex-grow-0"
                onclick={handleCreateClick}
                disabled={!pageDefinition}
            >
                <i class="fa-solid fa-plus mr-2"></i> Criar Novo
            </button>
        {/if}
        {#if permissions.can_manage_fields}
            <a
                href={`/admin/pages/edit/${pageDefinition?.page?.id}/`}
                class="btn btn-secondary flex-grow sm:flex-grow-0"
                class:btn-disabled={!pageDefinition}
            >
                <i class="fa-solid fa-wrench mr-2"></i> Gerir Página
            </a>
        {/if}
    </div>
</div>

{#if error}
    <div class="alert alert-error shadow-lg">
        <div>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="stroke-current flex-shrink-0 h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                ><path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                /></svg
            >
            <span>{error}</span>
        </div>
    </div>
{/if}

<div class="mb-4">
    <label class="input input-bordered flex items-center gap-2">
        <input
            type="text"
            class="grow"
            placeholder="Pesquisar..."
            bind:value={searchQuery}
            disabled={!pageDefinition}
        />
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 16 16"
            fill="currentColor"
            class="w-4 h-4 opacity-70"
            ><path
                fill-rule="evenodd"
                d="M9.965 11.026a5 5 0 1 1 1.06-1.06l2.755 2.754a.75.75 0 1 1-1.06 1.06l-2.755-2.754ZM10.5 7a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0Z"
                clip-rule="evenodd"
            /></svg
        >
    </label>
</div>

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
                {searchQuery}
                onRowClick={handleRowClick}
                currentPage={1}
                perPage={10}
                totalItems={Object.keys(records).length}
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
            ? `Editar Registo #${selectedRecordId}`
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
        showDeleteButton={permissions.can_delete && !!selectedRecordId}
        submitButtonText={selectedRecordId ? "Atualizar" : "Criar"}
        apiBaseUrl={API_BASE_URL}
    />
{/if}
