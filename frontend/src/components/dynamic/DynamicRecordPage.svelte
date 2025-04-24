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
    // searchQuery is now primarily controlled *by* the Table component via binding
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

    // --- Search Fields (remains the same, will be passed to Table) ---
    const searchFields = $derived.by(() => {
        if (!pageDefinition?.fields) return [];
        return (
            pageDefinition.fields
                .filter((field) => field.is_searchable)
                // Ensure we target the searchable fields created in processRecordData
                .map((field) => `processedData.${field.name}_search`)
        );
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

    // --- Data Fetching (Remove search query parameter) ---
    async function fetchRecords() {
        // Removed query parameter
        if (!pageDefinition?.page?.id) {
            console.warn("fetchRecords called before pageDefinition is ready.");
            isLoading = false;
            return;
        }
        isLoading = true;
        error = null;
        try {
            // Fetch ALL records, no search query passed to API
            const rawRecords = await getPageRecords(pageDefinition.page.id);
            const processed: Record<string, PageRecord> = {};
            for (const record of rawRecords) {
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

    // --- Data Processing (remains the same) ---
    function processRecordData(
        data: Record<string, any>,
        fields: PageField[],
    ): Record<string, any> {
        // ... (implementation remains the same) ...
        if (!fields) return {};
        const processed: Record<string, any> = {};

        for (const field of fields) {
            const rawValue = data[field.name];
            let displayValue: any = rawValue ?? "";
            let searchValue: string | undefined;
            let dateValue: Date | undefined; // For sorting

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
                        (opt) => opt.value == rawValue,
                    );
                    displayValue = selectedOption
                        ? selectedOption.label
                        : rawValue;
                    // Ensure search value exists even if option label isn't found, use raw value
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
                            // dateValue is already set by tryFormatDate(rawValue.start)
                            searchValue = displayValue;
                        } else {
                            displayValue = "Datas Inválidas";
                        }
                    } else {
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
                // Store the searchable string
                processed[`${field.name}_search`] = searchValue;
            }
            if (dateValue !== undefined) {
                processed[`${field.name}_date`] = dateValue;
            }
        }
        return processed;
    }

    // --- Mappers (remain the same) ---
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
        if (optionsData && Array.isArray(optionsData.items)) {
            if (
                optionsData.items.length > 0 &&
                typeof optionsData.items[0] === "object" &&
                optionsData.items[0].hasOwnProperty("value") &&
                optionsData.items[0].hasOwnProperty("label")
            ) {
                return optionsData.items as SelectOption[];
            }
        } else if (Array.isArray(optionsData)) {
            if (optionsData.length > 0 && typeof optionsData[0] === "string") {
                return optionsData.map((opt) => ({ value: opt, label: opt }));
            }
        }
        console.warn("Could not parse options data:", optionsData);
        return undefined;
    }

    // --- Event Handlers (handleRowClick, handleCreateClick remain the same) ---
    async function handleRowClick(id: string, row: PageRecord) {
        // ... (implementation remains the same) ...
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
        // ... (implementation remains the same) ...
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

    // --- Form Submission (remains the same) ---
    async function handleFormSubmit(
        formData: Record<string, any>,
        newFiles: File[],
    ): Promise<SubmitResponse> {
        // ... (implementation remains the same) ...
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
                    const intendedType = field.field_type_name;

                    if (intendedType === "NUMBER") {
                        if (typeof value === "string" && value !== "") {
                            value = parseFloat(value);
                            if (isNaN(value)) value = null;
                        } else if (value === "") {
                            value = null;
                        }
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
                        value = null;
                    }
                    payloadData[field.name] = value;
                }
            });

            if (selectedRecordId !== null) {
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
                await fetchRecords(); // Refetch list (no query needed here)
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
        // ... (implementation remains the same) ...
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
        // ... (implementation remains the same) ...
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

    // Remove the $effect for debounced fetching as search is now client-side
    // let debounceTimer: number;
    // $effect(() => {
    //     const currentQuery = searchQuery;
    //     clearTimeout(debounceTimer);
    //     debounceTimer = setTimeout(() => {
    //         if (pageDefinition?.page?.id) {
    //             fetchRecords(currentQuery); // No longer passing query
    //         }
    //     }, 300);
    //     return () => clearTimeout(debounceTimer);
    // });
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
    <!-- ... error alert ... -->
{/if}

<!-- Table component now handles search internally -->
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
    <!-- ... FormModal ... -->
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
