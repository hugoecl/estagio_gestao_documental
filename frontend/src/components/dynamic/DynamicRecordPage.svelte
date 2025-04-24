<script lang="ts">
    import { onMount, tick } from "svelte";
    import Table from "@components/common/Table.svelte";
    import FormModal from "@components/common/FormModal.svelte";
    import type {
        CustomPageWithFields,
        UserPagePermissions,
    } from "@lib/types/custom-page";
    import type { PageField } from "@lib/types/fields";
    // Explicitly import PageRecordWithFiles and PageRecordFile
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
    // Use the imported type directly
    let selectedRecordWithFiles = $state<PageRecordWithFiles | null>(null);
    let originalRecordJson = $state<string | null>(null);

    // --- Permissions ---
    const permissions = $derived<UserPagePermissions>(
        pageDefinition?.currentUserPermissions || {
            // Add safe navigation for pageDefinition
            can_view: true,
            can_create: false,
            can_edit: false,
            can_delete: false,
            can_manage_fields: false,
            is_admin: false,
        },
    );

    // --- Table Columns ---
    const tableColumns = $derived.by(() => {
        // Add safe navigation for pageDefinition
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
        // Add safe navigation for pageDefinition
        if (!pageDefinition?.fields) return [];
        return pageDefinition.fields
            .filter((field) => field.is_searchable)
            .map((field) => `processedData.${field.name}_search`);
    });

    // --- Form Fields ---
    const formFields = $derived(() => {
        // Ensure pageDefinition and pageDefinition.fields exist and are an array
        if (!pageDefinition?.fields || !Array.isArray(pageDefinition.fields)) {
            return []; // Return empty array if fields are not ready
        }
        return pageDefinition.fields
            .sort((a, b) => a.order_index - b.order_index)
            .map((pf) => ({
                id: pf.name,
                label: pf.display_name,
                type: mapFieldType(pf.field_type_name),
                required: pf.required,
                options: pf.options ? mapOptions(pf.options) : undefined,
                value: null, // Let FormModal handle initial value
                placeholder: `Insira ${pf.display_name.toLowerCase()}`,
            }));
    });

    // --- Data Fetching and Processing ---
    async function fetchRecords(query?: string) {
        // Add safe navigation for pageDefinition
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
                record.processedData = processRecordData(
                    record.data,
                    pageDefinition.fields,
                );
                processed[record.id] = { ...record, id: record.id };
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

    function processRecordData(
        data: Record<string, any>,
        fields: PageField[],
    ): Record<string, any> {
        // Add safe navigation for fields
        if (!fields) return {};
        const processed: Record<string, any> = {};
        for (const field of fields) {
            const rawValue = data[field.name];
            let displayValue: any = rawValue ?? "";
            let searchValue: string | undefined;
            let dateValue: Date | undefined;

            switch (field.field_type_name) {
                case "SELECT":
                    const options = mapOptions(field.options);
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
                case "DATE":
                    if (rawValue) {
                        try {
                            dateValue = new Date(rawValue + "T00:00:00Z");
                            displayValue =
                                dateValue.toLocaleDateString("pt-PT");
                            searchValue = displayValue;
                        } catch (e) {
                            displayValue = "Data Inválida";
                        }
                    } else {
                        displayValue = "";
                    }
                    break;
                case "DATE_RANGE":
                    if (
                        rawValue &&
                        typeof rawValue === "object" &&
                        rawValue.start &&
                        rawValue.end
                    ) {
                        try {
                            const startDate = new Date(
                                rawValue.start + "T00:00:00Z",
                            );
                            const endDate = new Date(
                                rawValue.end + "T00:00:00Z",
                            );
                            displayValue = `${startDate.toLocaleDateString("pt-PT")} - ${endDate.toLocaleDateString("pt-PT")}`;
                            dateValue = startDate;
                            searchValue = displayValue;
                        } catch (e) {
                            displayValue = "Datas Inválidas";
                        }
                    } else {
                        displayValue = "";
                    }
                    break;
                case "NUMBER":
                    displayValue = rawValue;
                    searchValue = rawValue?.toString();
                    break;
                case "TEXT":
                case "TEXTAREA":
                default:
                    displayValue = rawValue;
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

    function mapOptions(
        optionsData: any,
    ): import("@lib/types/form-modal").SelectOption[] | undefined {
        if (Array.isArray(optionsData)) {
            if (
                optionsData.length > 0 &&
                typeof optionsData[0] === "object" &&
                optionsData[0].hasOwnProperty("value") &&
                optionsData[0].hasOwnProperty("label")
            ) {
                return optionsData as import("@lib/types/form-modal").SelectOption[];
            } else if (
                optionsData.length > 0 &&
                typeof optionsData[0] === "string"
            ) {
                return optionsData.map((opt) => ({ value: opt, label: opt }));
            }
        }
        return undefined;
    }

    // --- Event Handlers ---
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
        isLoading = true; // Indicate loading details
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
                selectedRecordId = null; // Reset if fetch failed
            }
        } catch (e) {
            showAlert(
                "Erro ao carregar detalhes do registo.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            selectedRecordId = null; // Reset on error
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
        selectedRecordWithFiles = null; // Clear selected record details
        originalRecordJson = JSON.stringify({}); // Empty object for comparison
        formModalRef?.showModal();
        currentModal.set(formModalRef?.children[0] as HTMLDivElement);
    }

    async function handleFormSubmit(
        formData: Record<string, any>,
        newFiles: File[],
    ): Promise<SubmitResponse> {
        // Add safe navigation for pageDefinition
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
                    if (
                        field.field_type_name === "NUMBER" &&
                        typeof value === "string" &&
                        value !== ""
                    ) {
                        value = parseFloat(value);
                        if (isNaN(value)) value = null;
                    } else if (
                        field.field_type_name === "NUMBER" &&
                        value === ""
                    ) {
                        value = null;
                    } else if (
                        field.field_type_name === "DATE_RANGE" &&
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
                        field.field_type_name === "DATE" &&
                        typeof value === "string"
                    ) {
                        if (value && /^\d{2}\/\d{2}\/\d{4}$/.test(value)) {
                            const [d, m, y] = value.split("/");
                            value = `${y}-${m}-${d}`;
                        } else {
                            value = null;
                        }
                    } else if (value === "") {
                        if (
                            field.field_type_name !== "TEXT" &&
                            field.field_type_name !== "TEXTAREA"
                        ) {
                            value = null;
                        }
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
                await fetchRecords(searchQuery);
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

    async function handleDeleteRecordSubmit(): Promise<boolean> {
        if (selectedRecordId === null || !permissions.can_delete) return false;
        try {
            const success = await deleteRecord(selectedRecordId);
            if (success) {
                const updatedRecords = { ...records };
                delete updatedRecords[selectedRecordId];
                records = updatedRecords;

                selectedRecordId = null;
                selectedRecordWithFiles = null; // Clear selection
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
                // Update the state if the modal for this record is open
                if (
                    selectedRecordWithFiles &&
                    selectedRecordWithFiles.record.id === recordId
                ) {
                    // Create a new array without the deleted file
                    const updatedFiles = selectedRecordWithFiles.files.filter(
                        (f) => f.id !== fileId,
                    );
                    // Create a new object to trigger reactivity
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

    onMount(() => {
        // Fetch records only if pageDefinition is available
        if (pageDefinition?.page?.id) {
            fetchRecords();
        } else {
            // Handle case where pageDefinition might not be ready immediately
            // This might happen if the parent component loads it asynchronously
            console.warn(
                "DynamicRecordPage mounted, but pageDefinition not fully ready.",
            );
            // Consider adding a loading state or error message here if pageDefinition is expected
        }
    });

    let debounceTimer: number;
    $effect(() => {
        const currentQuery = searchQuery;
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
            // Fetch records only if pageDefinition is available
            if (pageDefinition?.page?.id) {
                fetchRecords(currentQuery);
            }
        }, 300);
        return () => clearTimeout(debounceTimer);
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
