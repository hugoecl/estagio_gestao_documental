<script lang="ts">
    import FormModal from "@components/common/FormModal.svelte";
    import Table from "@components/common/Table.svelte";
    import {
        FieldType,
        SubmitResult,
        type FormField,
        type SubmitResponse,
    } from "@lib/types/form-modal";
    import type { Model, Models } from "@lib/types/model";
    import type { TableColumn } from "@lib/types/table";
    import { currentModal } from "@stores/modal-store";
    import { onMount } from "svelte";

    let loading = $state(true);
    let models: Models = $state({});
    let modal: HTMLDialogElement;

    const columns: TableColumn[] = [
        { header: "ID", field: "ID" },
        { header: "Nome", field: "name" },
        { header: "Versão", field: "version" },
        { header: "Modelo", field: "model" },
        { header: "Descrição", field: "description" },
        {
            header: "Criado Em",
            field: "createdAt",
            dateValueField: "createdAtDate",
        },
        {
            header: "Atualizado Em",
            field: "updatedAt",
            dateValueField: "updatedAtDate",
        },
    ];

    let selectedModelId: string | null = $state(null);
    let originalModelJson: string | null = $state(null);
    let selectedModel: Model | null = $state(null);

    const fields: FormField[] = $derived([
        {
            id: "name",
            type: FieldType.TEXT,
            label: "Nome",
            placeholder: "Digite o nome do modelo",
            value: selectedModel ? (selectedModel as Model).name : "",
            searchField: "__searchName",
        },
        {
            id: "version",
            type: FieldType.TEXT,
            label: "Versão",
            placeholder: "Digite a versão do modelo",
            value: selectedModel ? (selectedModel as Model).version : "",
        },
        {
            id: "model",
            type: FieldType.TEXT,
            label: "Modelo",
            placeholder: "Digite o modelo",
            value: selectedModel ? (selectedModel as Model).model : "",
        },
        {
            id: "description",
            type: FieldType.TEXTAREA,
            label: "Descrição",
            placeholder: "Digite a descrição do modelo",
            value: selectedModel ? (selectedModel as Model).description : "",
            searchField: "__searchDescription",
            required: false,
        },
    ]);

    function openModal(id: string, model: Model) {
        selectedModelId = id;
        selectedModel = $state.snapshot(model);
        originalModelJson = JSON.stringify({
            ...selectedModel,
            files: undefined,
        });
        modal.showModal();

        const modalBox = modal.children[0] as HTMLDivElement;
        currentModal.set(modalBox);
    }

    function openNewModelModal() {
        selectedModelId = null;
        selectedModel = null;
        originalModelJson = JSON.stringify({
            name: "",
            version: "",
            model: "",
            description: "",
        });
        modal.showModal();
        const modalBox = modal.children[0] as HTMLDivElement;
        currentModal.set(modalBox);
    }

    async function handleUpdateModel(
        data: Record<string, any>,
        files: File[],
    ): Promise<SubmitResponse> {
        const { updateModel, uploadModelFiles } = await import(
            "@api/models-api"
        );

        const editedModel = {
            ...selectedModel!,
            files: undefined,
            ...data,
        } as unknown as Model;

        const hasChanged = JSON.stringify(editedModel) !== originalModelJson;

        const hasNewFiles = files.length > 0;
        let success = true;

        const now = new Date();
        const nowString = now.toLocaleString("pt-PT");

        if (hasChanged && hasNewFiles) {
            const [modelResult, [filesResult, filesBaseId]] = await Promise.all(
                [
                    updateModel(selectedModelId!, editedModel),
                    uploadModelFiles(selectedModelId!, files),
                ],
            );

            success = modelResult && filesResult;

            for (let i = 0, len = files.length; i < len; i++) {
                const file = files[i];
                selectedModel!.files[filesBaseId + i] = {
                    name: file.name,
                    path: `media/quality/models/${selectedModelId}/${file.name}`,
                    uploadedAt: nowString,
                };
            }
        } else if (hasChanged) {
            success = await updateModel(selectedModelId!, editedModel);
        } else if (hasNewFiles) {
            const [result, filesBaseId] = await uploadModelFiles(
                selectedModelId!,
                files,
            );
            success = result;

            for (let i = 0, len = files.length; i < len; i++) {
                const file = files[i];
                selectedModel!.files[filesBaseId + i] = {
                    name: file.name,
                    path: `media/quality/models/${selectedModelId}/${file.name}`,
                    uploadedAt: nowString,
                };
            }
        } else {
            return [SubmitResult.UNCHANGED, null];
        }

        if (success) {
            models[selectedModelId!] = {
                ...selectedModel!,
                ...data,
                updatedAt: nowString,
                updatedAtDate: now,
            };

            return [SubmitResult.SUCCESS, models[selectedModelId!]];
        }
        return [SubmitResult.ERROR, null];
    }

    async function handleCreateModel(
        data: Record<string, any>,
        files: File[],
    ): Promise<SubmitResponse> {
        const formData = new FormData();
        const entries = Object.entries(data);

        for (let i = 0, len = entries.length; i < len; i++) {
            const [key, value] = entries[i];
            formData.append(key, value);
        }
        for (let i = 0, len = files.length; i < len; i++) {
            const file = files[i];
            formData.append("files", file, `${file.name}_${file.size}`);
        }

        const { uploadModel } = await import("@api/models-api");

        const [ok, modelId, fileBaseId] = await uploadModel(formData);

        if (ok) {
            const now = new Date();
            const nowString = now.toLocaleString("pt-PT");

            models[modelId] = {
                ...data,
                updatedAt: nowString,
                updatedAtDate: now,
                createdAt: nowString,
                createdAtDate: now,
                files: {},
            } as unknown as Model;

            const model = models[modelId];

            for (let i = 0, len = files.length; i < len; i++) {
                const file = files[i];
                model.files[fileBaseId + i] = {
                    name: file.name,
                    path: `media/quality/models/${selectedModelId}`,
                    uploadedAt: nowString,
                };
            }
            return [SubmitResult.SUCCESS, model];
        }

        return [SubmitResult.ERROR, null];
    }

    async function handleSubmit(
        data: Record<string, any>,
        files: File[],
    ): Promise<SubmitResponse> {
        if (selectedModelId) {
            return await handleUpdateModel(data, files);
        } else {
            return await handleCreateModel(data, files);
        }
    }

    onMount(async () => {
        const [{ getModels }, { AlertPosition, AlertType, showAlert }] =
            await Promise.all([
                import("@api/models-api"),
                import("@components/alert/alert"),
            ]);

        const modelsOrNull = await getModels();
        loading = false;
        if (!modelsOrNull) {
            showAlert(
                "Erro ao carregar modelos",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }
        models = modelsOrNull;
    });
</script>

<div class="mb-4 flex justify-between">
    <h1 class="text-2xl font-bold">Modelos</h1>
    <button class="btn btn-primary" onclick={openNewModelModal}>
        Adicionar Novo Modelo
    </button>
</div>

<FormModal
    bind:formModal={modal}
    title={selectedModelId ? "Editar Modelo" : "Novo Modelo"}
    {fields}
    recordId={selectedModelId || ""}
    showFiles={true}
    files={selectedModel ? (selectedModel as Model).files : {}}
    onSubmit={handleSubmit}
    showDeleteButton={false}
    submitButtonText={selectedModelId ? "Atualizar" : "Criar"}
/>

<Table
    data={models}
    {columns}
    {loading}
    emptyMessage="Nenhum modelo disponível"
    keyField="ID"
    searchFields={[
        "__searchName",
        "__searchDescription",
        "updatedAt",
        "createdAt",
        "version",
        "model",
    ]}
    onRowClick={openModal}
/>
