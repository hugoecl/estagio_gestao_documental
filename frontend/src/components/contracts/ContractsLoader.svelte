<script lang="ts">
    import {
        ContractServicesObject,
        ContractStatusObject,
        ContractTypesObject,
        type Contract,
        type Contracts,
    } from "@lib/types/contracts";
    import { onMount } from "svelte";
    import Table from "@components/common/Table.svelte";
    import type { TableColumn } from "@lib/types/table";
    import {
        FieldType,
        SubmitResult,
        type FormField,
        type SubmitResponse,
    } from "@lib/types/form-modal";
    import { LocationsObject } from "@lib/types/locations";
    import { currentModal } from "@stores/modal-store";
    import FormModal from "@components/common/FormModal.svelte";
    import { DMYToDate } from "@utils/date-utils";

    let loading = $state(true);
    let contracts: Contracts = $state({});
    let modal: HTMLDialogElement;
    // TODO: Make columns responsive
    const columns: TableColumn[] = [
        { header: "ID", field: "ID" },
        { header: "Fornecedor", field: "supplier" },
        { header: "Serviço", field: "service" },
        { header: "Local", field: "location" },
        {
            header: "Número de Contrato",
            field: "contractNumber",
            responsive: "",
        },
        { header: "Data", field: "dateString", dateValueField: "date" },
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
        { header: "Tipo", field: "type" },
        { header: "Estado", field: "status" },
    ];

    let selectedContractId: string | null = $state(null);
    let originalContractJson: string | null = $state(null);
    let selectedContract: Contract | null = $state(null);

    const fields: FormField[] = $derived([
        {
            id: "supplier",
            type: FieldType.TEXT,
            label: "Fornecedor",
            placeholder: "Digite o nome do fornecedor",
            value: selectedContract
                ? (selectedContract as Contract).supplier
                : "",
            searchField: "__searchSupplier",
        },
        {
            id: "contractNumber",
            type: FieldType.NUMBER,
            label: "Número de Contrato",
            placeholder: "Digite o número do contrato",
            value: selectedContract
                ? (selectedContract as Contract).contractNumber
                : "",
            searchField: "__searchContractNumber",
        },
        {
            id: "locationValue",
            type: FieldType.SELECT,
            label: "Local",
            options: LocationsObject,
            value: selectedContract
                ? (selectedContract as Contract).locationValue
                : "",
            searchField: "__searchLocation",
        },
        {
            id: "serviceValue",
            type: FieldType.SELECT,
            label: "Serviço",
            options: ContractServicesObject,
            value: selectedContract
                ? (selectedContract as Contract).serviceValue
                : "",
            searchField: "__searchService",
        },

        {
            id: "date",
            type: FieldType.DATE,
            label: "Data",
            value: selectedContract
                ? (selectedContract as Contract).dateString
                : null,
            searchField: "dateString",
        },
        {
            id: "dateRange",
            type: FieldType.DATE_RANGE,
            label: "Data Início e Fim",
            value: selectedContract
                ? [
                      (selectedContract as Contract).dateStartString,
                      (selectedContract as Contract).dateEndString,
                  ]
                : [],
            searchField: "dateStartString,dateEndString,dateStart,dateEnd",
        },
        {
            id: "typeValue",
            type: FieldType.SELECT,
            label: "Tipo",
            options: ContractTypesObject,
            value: selectedContract
                ? (selectedContract as Contract).typeValue
                : "",
            searchField: "__searchType",
        },
        {
            id: "statusValue",
            type: FieldType.SELECT,
            label: "Estado",
            options: ContractStatusObject,
            value: selectedContract
                ? (selectedContract as Contract).statusValue
                : "",
            searchField: "__searchStatus",
        },
        {
            id: "description",
            type: FieldType.TEXTAREA,
            label: "Descrição",
            placeholder: "Digite uma descrição",
            value: selectedContract
                ? (selectedContract as Contract).description
                : null,
            colSpan: 2,
            searchField: "__searchDescription",
        },
    ]);

    function openContractModal(id: string, contract: Contract) {
        selectedContractId = id;
        selectedContract = $state.snapshot(contract); // Deep copy to prevent direct mutations
        originalContractJson = JSON.stringify({
            ...selectedContract,
            files: undefined,
        });

        modal.showModal();
        const modalBox = modal.children[0] as HTMLDivElement;
        currentModal.set(modalBox);
    }

    function openNewContractModal() {
        selectedContractId = null;
        selectedContract = null;
        originalContractJson = JSON.stringify({
            id: null,
            description: "",
            files: [],
        });
        modal.showModal();
        const modalBox = modal.children[0] as HTMLDivElement;
        currentModal.set(modalBox);
    }

    async function handleUpdateContract(
        data: Record<string, any>,
        files: File[],
    ): Promise<SubmitResponse> {
        const { updateContract, uploadContractFiles } = await import(
            "@api/contracts-api"
        );

        const editedContract = {
            ...selectedContract,
            files: undefined,
            ...data,
            // for some reason the date here is returned as a string while the dateStart and dateEnd are returned as Date objects
            date: DMYToDate(data!.date as unknown as string),
            dateRange: undefined,
        } as unknown as Contract;

        const hasChanged =
            JSON.stringify(editedContract) !== originalContractJson;

        const hasNewFiles = files.length > 0;

        let success = true;

        const now = new Date();
        const nowString = now.toLocaleString("pt-PT");

        if (hasChanged && hasNewFiles) {
            const [contractResult, [filesResult, filesBaseId]] =
                await Promise.all([
                    updateContract(selectedContractId!, editedContract),
                    uploadContractFiles(selectedContractId!, files),
                ]);
            success = contractResult && filesResult;

            for (let i = 0, len = files.length; i < len; i++) {
                const file = files[i];
                selectedContract!.files[filesBaseId + i] = {
                    name: file.name,
                    path: `media/contracts/${selectedContractId}/${file.name}`,
                    uploadedAt: nowString,
                };
            }
        } else if (hasChanged) {
            success = await updateContract(selectedContractId!, editedContract);
        } else if (hasNewFiles) {
            const [result, filesBaseId] = await uploadContractFiles(
                selectedContractId!,
                files,
            );
            success = result;

            for (let i = 0, len = files.length; i < len; i++) {
                const file = files[i];
                selectedContract!.files[filesBaseId + i] = {
                    name: file.name,
                    path: `media/contracts/${selectedContractId}/${file.name}`,
                    uploadedAt: nowString,
                };
            }
        } else {
            return [SubmitResult.UNCHANGED, null];
        }

        if (success) {
            // @ts-ignore javascript can take string as indexes
            contracts[selectedContractId!] = {
                ...selectedContract!,
                ...data,
            };

            // @ts-ignore javascript can take string as indexes
            return [SubmitResult.SUCCESS, contracts[selectedContractId!]];
        }
        return [SubmitResult.ERROR, null];
    }

    async function handleCreateContract(
        data: Record<string, any>,
        files: File[],
    ): Promise<SubmitResponse> {
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

        const { uploadContract } = await import("@api/contracts-api");

        const [ok, contractId, fileBaseId] = await uploadContract(formData);

        if (ok) {
            const nowString = new Date().toLocaleString("pt-PT");
            contracts[contractId] = {
                ...data,
                files: {},
            } as unknown as Contract;

            const contract = contracts[contractId];

            for (let i = 0, len = files.length; i < len; i++) {
                const file = files[i];
                contract.files[fileBaseId + i] = {
                    name: file.name,
                    path: `media/contracts/${contractId}/${file.name}`,
                    uploadedAt: nowString,
                };
            }
            return [SubmitResult.SUCCESS, contract];
        }

        return [SubmitResult.ERROR, null];
    }

    async function handleSubmit(
        data: Record<string, any>,
        files: File[],
    ): Promise<SubmitResponse> {
        if (selectedContractId) {
            return await handleUpdateContract(data, files);
        } else {
            return await handleCreateContract(data, files);
        }
    }

    async function handleDeleted(): Promise<boolean> {
        const { deleteContract } = await import("@api/contracts-api");
        const success = await deleteContract(selectedContractId!);

        if (!success) {
            return false;
        }

        // @ts-ignore
        delete contracts[selectedContractId];
        return true;
    }

    async function handleFileDeleted(
        contractId: string,
        fileId: string,
    ): Promise<boolean> {
        const { deleteContractFile } = await import("@api/contracts-api");

        const success = await deleteContractFile(contractId, fileId);
        if (!success) {
            return false;
        }

        // @ts-ignore
        delete contracts[contractId];
        return true;
    }

    onMount(async () => {
        const [{ getContracts }, { AlertPosition, AlertType, showAlert }] =
            await Promise.all([
                import("@api/contracts-api"),
                import("@components/alert/alert"),
            ]);
        const contractsOrNull = await getContracts();
        if (!contractsOrNull) {
            showAlert(
                "Erro ao carregar contratos",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            loading = false;
            return;
        }
        contracts = contractsOrNull;
        loading = false;
    });
</script>

<div class="mb-4 flex justify-between">
    <h1 class="text-2xl font-bold">Contratos</h1>
    <button class="btn btn-primary" onclick={openNewContractModal}
        >Novo Contrato</button
    >
</div>

<FormModal
    bind:formModal={modal}
    title={selectedContractId
        ? `Contrato #${selectedContract?.contractNumber} - ${selectedContract?.supplier}`
        : "Novo Contrato"}
    {fields}
    recordId={selectedContractId || ""}
    showFiles={true}
    files={selectedContractId ? (selectedContract as Contract).files : {}}
    onSubmit={handleSubmit}
    onDelete={handleDeleted}
    onFileDeleted={handleFileDeleted}
    showDeleteButton={!!selectedContractId}
    submitButtonText={selectedContractId ? "Atualizar" : "Criar"}
/>

<Table
    data={contracts}
    {columns}
    {loading}
    emptyMessage="Nenhum contrato disponível"
    keyField="ID"
    searchFields={[
        "__searchSupplier",
        "__searchLocation",
        "__searchService",
        "__searchContractNumber",
        "dateString",
        "dateStartString",
        "dateEndString",
        "dateEndString",
        "__searchType",
        "__searchStatus",
    ]}
    onRowClick={openContractModal}
/>
