<script lang="ts">
    import FormModal from "@components/common/FormModal.svelte";
    import Table from "@components/common/Table.svelte";
    import { SubmitResult, type FormField } from "@lib/types/form-modal";
    import type { License } from "@lib/types/radiological-protection-licenses";
    import type { TableColumn } from "@lib/types/table";
    import { currentModal } from "@stores/modal-store";
    import { onMount } from "svelte";

    let loading = $state(true);
    let licenses = $state({});
    let modal: HTMLDialogElement;

    const columns: TableColumn[] = [
        { header: "ID", field: "ID" },
        { header: "Âmbito", field: "scope" },
        { header: "Número da Licença", field: "licenseNumber" },
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

    const fields: FormField[] = $derived([
        {
            id: "scope",
            type: "text",
            label: "Âmbito",
            placeholder: "Âmbito",
            value: selectedLicense ? (selectedLicense as License).scope : "",
        },
        {
            id: "licenseNumber",
            type: "number",
            label: "Número da Licença",
            placeholder: "Número da Licença",
            value: selectedLicense
                ? (selectedLicense as License).licenseNumber
                : "",
        },
        {
            id: "dateStartString",
            type: "date",
            label: "Data Início",
            value: selectedLicense
                ? (selectedLicense as License).dateStartString
                : "",
        },
        {
            id: "dateEndString",
            type: "date",
            label: "Data Fim",
            value: selectedLicense
                ? (selectedLicense as License).dateEndString
                : "",
        },
        {
            id: "description",
            type: "textarea",
            label: "Descrição (Opcional)",
            placeholder: "Descrição",
            required: false,
            value: selectedLicense
                ? (selectedLicense as License).description
                : "",
            colSpan: 2,
        },
    ]);

    function openModal(id: string, license: License) {
        selectedLicenseId = id;
        selectedLicense = $state.snapshot(license);
        originalLicenseJson = JSON.stringify({
            ...selectedLicense,
            files: undefined,
        });
        modal.show();
        // get the first child of the modal
        const modalBox = modal.children[0] as HTMLDivElement;
        currentModal.set(modalBox);
    }

    async function handleSubmit(
        data: Record<string, any>,
        files: File[],
    ): Promise<SubmitResult> {
        const [
            { updateLicense, uploadLicenseFiles },
            { showAlert, AlertType, AlertPosition },
        ] = await Promise.all([
            import("@api/radiological-protection-licenses-api"),
            import("@components/alert/alert"),
        ]);

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
                    uploadedAt: new Date().toLocaleString("pt-PT"),
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
                    uploadedAt: new Date().toLocaleString("pt-PT"),
                };
            }
        }
        // Scenario 4: No changes
        else {
            return SubmitResult.UNCHANGED;
        }

        if (success) {
            // @ts-ignore javascript can take string as indexes
            licenses[selectedLicenseId!] = {
                ...selectedLicense!,
                ...data,
            };
            return SubmitResult.SUCCESS;
        }

        return SubmitResult.ERROR;
    }

    async function handleDeleted(): Promise<boolean> {
        // @ts-ignore javascript can take string as indexes
        delete licenses[selectedLicenseId];
        return true;
    }

    async function handleFileDeleted(
        licenseId: string,
        fileId: string,
    ): Promise<boolean> {
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
