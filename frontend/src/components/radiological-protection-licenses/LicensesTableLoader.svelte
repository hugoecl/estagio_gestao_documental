<script lang="ts">
    import FormModal from "@components/common/FormModal.svelte";
    import Table from "@components/common/Table.svelte";
    import {
        FieldType,
        SubmitResult,
        type FormField,
        type SubmitResponse,
    } from "@lib/types/form-modal";
    import { LocationsObject } from "@lib/types/locations";
    import type {
        License,
        Licenses,
    } from "@lib/types/radiological-protection-licenses";
    import type { TableColumn } from "@lib/types/table";
    import { currentModal } from "@stores/modal-store";
    import { onMount } from "svelte";

    let loading = $state(true);
    let licenses: Licenses = $state({});
    let modal: HTMLDialogElement;

    const columns: TableColumn[] = [
        { header: "ID", field: "ID" },
        { header: "Âmbito", field: "scope" },
        { header: "Número da Licença", field: "licenseNumber" },
        { header: "Local", field: "location" },
        { header: "Descrição", field: "description" },
        {
            header: "Data Início",
            field: "dateStartString",
            dateValueField: "dateStart",
        },
        {
            header: "Data Fim",
            field: "dateEndString",
            dateValueField: "dateEnd",
        },
        {
            header: "Criado em",
            field: "createdAtString",
            dateValueField: "createdAt",
        },
        {
            header: "Atualizado em",
            field: "updatedAtString",
            dateValueField: "updatedAt",
        },
    ];

    let selectedLicenseId: string | null = $state(null);
    let originalLicenseJson: string | null = $state(null);
    let selectedLicense: License | null = $state(null);

    // TODO: See about making value of FormField nullable instead of using empty string

    const fields: FormField[] = $derived([
        {
            id: "scope",
            type: FieldType.TEXT,
            label: "Âmbito",
            placeholder: "Âmbito",
            value: selectedLicense ? (selectedLicense as License).scope : "",
            searchField: "__searchScope",
        },
        {
            id: "licenseNumber",
            type: FieldType.NUMBER,
            label: "Número da Licença",
            placeholder: "Número da Licença",
            value: selectedLicense
                ? (selectedLicense as License).licenseNumber
                : "",
            searchField: "__searchLicenseNumber",
        },
        {
            id: "dateRange",
            type: FieldType.DATE_RANGE,
            label: "Data Início e Fim",
            value: selectedLicense
                ? [
                      (selectedLicense as License).dateStartString,
                      (selectedLicense as License).dateEndString,
                  ]
                : [],
            searchField: "dateStartString,dateEndString,dateStart,dateEnd",
        },
        {
            id: "locationValue",
            type: FieldType.SELECT,
            label: "Local",
            options: LocationsObject,
            value: selectedLicense
                ? (selectedLicense as License).locationValue
                : "",
            searchField: "__searchLocation",
        },
        {
            id: "description",
            type: FieldType.TEXTAREA,
            label: "Descrição (Opcional)",
            placeholder: "Descrição",
            required: false,
            value: selectedLicense
                ? (selectedLicense as License).description
                : "",
            colSpan: 2,
            searchField: "__searchDescription",
        },
    ]);

    function openModal(id: string, license: License) {
        selectedLicenseId = id;
        selectedLicense = $state.snapshot(license);
        originalLicenseJson = JSON.stringify({
            ...selectedLicense,
            files: undefined,
        });
        modal.showModal();
        // get the first child of the modal
        const modalBox = modal.children[0] as HTMLDivElement;
        currentModal.set(modalBox);
    }

    async function handleSubmit(
        data: Record<string, any>,
        files: File[],
    ): Promise<SubmitResponse> {
        const { updateLicense, uploadLicenseFiles } = await import(
            "@api/radiological-protection-licenses-api"
        );

        const formData = new FormData();
        const entries = Object.entries(data);

        for (let i = 0; i < entries.length; i++) {
            const [key, value] = entries[i];
            formData.append(key, value);
        }
        for (let i = 0, len = files.length; i < len; i++) {
            const file = files[i];
            formData.append("files", file, `${file.name}_${file.size}`);
        }

        const editedLicense = {
            ...selectedLicense!,
            files: undefined,
            ...data,
        } as unknown as License;

        const hasChanged =
            JSON.stringify(editedLicense) !== originalLicenseJson;
        const hasNewFiles = files.length > 0;

        let success = true;

        const now = new Date();
        const nowString = now.toLocaleString("pt-PT");

        // Scenario 1: Both license data nad files have changed
        if (hasChanged && hasNewFiles) {
            const [licenseResult, [filesResult, filesBaseId]] =
                await Promise.all([
                    updateLicense(selectedLicenseId!, editedLicense),
                    uploadLicenseFiles(selectedLicenseId!, files),
                ]);
            success = licenseResult && filesResult;

            for (let i = 0, len = files.length; i < len; i++) {
                const file = files[i];
                selectedLicense!.files[filesBaseId + i] = {
                    name: file.name,
                    path: `media/radiological-protection/${file.name}`,
                    uploadedAt: nowString,
                };
            }
        }
        // Scenario 2: Only license data has changed
        else if (hasChanged) {
            success = await updateLicense(selectedLicenseId!, editedLicense);
        }
        // Scenario 3: Only files have Changed
        else if (hasNewFiles) {
            const [result, filesBaseId] = await uploadLicenseFiles(
                selectedLicenseId!,
                files,
            );
            success = result;

            for (let i = 0, len = files.length; i < len; i++) {
                const file = files[i];
                selectedLicense!.files[filesBaseId + i] = {
                    name: file.name,
                    path: `media/radiological-protection/${file.name}`,
                    uploadedAt: nowString,
                };
            }
        }
        // Scenario 4: No changes
        else {
            return [SubmitResult.UNCHANGED, null];
        }

        if (success) {
            // @ts-ignore javascript can take string as indexes
            licenses[selectedLicenseId!] = {
                ...selectedLicense!,
                ...data,
                updatedAt: now,
                updatedAtString: nowString,
            };
            // @ts-ignore javascript can take string as indexes
            return [SubmitResult.SUCCESS, licenses[selectedLicenseId!]];
        }

        return [SubmitResult.ERROR, null];
    }

    async function handleDeleted(): Promise<boolean> {
        const { deleteLicense } = await import(
            "@api/radiological-protection-licenses-api"
        );

        const success = await deleteLicense(selectedLicenseId!);

        if (!success) {
            return false;
        }

        // @ts-ignore javascript can take string as indexes
        delete licenses[selectedLicenseId];
        return true;
    }

    async function handleFileDeleted(
        licenseId: string,
        fileId: string,
    ): Promise<boolean> {
        const { deleteLicenseFile } = await import(
            "@api/radiological-protection-licenses-api"
        );

        const success = await deleteLicenseFile(licenseId, fileId);

        if (!success) {
            return false;
        }

        // @ts-ignore javascript can take string as indexes
        delete licenses[licenseId].files[fileId];
        return true;
    }

    onMount(() => {
        (async () => {
            const [{ getLicenses }, { AlertPosition, AlertType, showAlert }] =
                await Promise.all([
                    import("@api/radiological-protection-licenses-api"),
                    import("@components/alert/alert"),
                ]);

            const licensesOrNull = await getLicenses();
            loading = false;
            if (!licensesOrNull) {
                showAlert(
                    "Erro ao carregar licenças",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
                return;
            }
            licenses = licensesOrNull;
        })();
    });
</script>

<FormModal
    bind:formModal={modal}
    title={selectedLicenseId ? "Editar Licença" : "Nova Licença"}
    {fields}
    recordId={selectedLicenseId || ""}
    showFiles={true}
    files={selectedLicense ? (selectedLicense as License).files : {}}
    onSubmit={handleSubmit}
    onDelete={handleDeleted}
    onFileDeleted={handleFileDeleted}
    showDeleteButton={!!selectedLicenseId}
    submitButtonText={selectedLicenseId ? "Atualizar" : "Criar"}
/>

<Table
    data={licenses}
    {columns}
    {loading}
    emptyMessage="Nenhuma licença disponível"
    keyField="ID"
    searchFields={[
        "__searchScope",
        "__searchLocation",
        "__searchLicenseNumber",
        "__searchDescription",
        "dateStartString",
        "dateEndString",
        "createdAtString",
        "updatedAtString",
    ]}
    onRowClick={openModal}
/>
