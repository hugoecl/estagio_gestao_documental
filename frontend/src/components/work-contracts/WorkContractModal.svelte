<script lang="ts">
  import DatePicker from "@components/common/DatePicker.svelte";
  import {
    WorkContractTypes,
    type WorkContract,
  } from "@lib/types/work-contracts";
  import { categories } from "@stores/work-contract-stores";
  import API_BASE_URL from "@api/base-url";

  const {
    workContractId,
    workContract,
    originalWorkContractJson,
    onWorkContractUpdated,
    onWorkContractDeleted,
    onFileDeleted,
  }: {
    workContractId: string;
    workContract: WorkContract;
    originalWorkContractJson: string;
    onWorkContractUpdated: (workContract: WorkContract) => void;
    onWorkContractDeleted: (workContractId: string) => void;
    onFileDeleted: (workContractId: string, fileId: string) => void;
  } = $props();

  let modal: HTMLDialogElement;
  let confirmModal: HTMLDialogElement;

  let newFiles = $state<File[]>([]);
  let fileInput: HTMLInputElement;
  let isSubmitting = $state(false);

  const enum ConfirmationAction {
    DELETE_WORK_CONTRACT,
    DELETE_FILE,
  }
  let confirmationAction = $state<ConfirmationAction | null>(null);
  let fileToDeleteId = $state<string | null>(null);
  let isDeleteSubmitting = $state(false);

  const existingFiles = $derived(
    workContract.files
      ? Object.entries(workContract.files).map(([id, file]) => ({
          id,
          ...file,
          uploadedAt: file.uploadedAt,
        }))
      : null
  );

  function closeModal() {
    modal.close();
    newFiles = [];
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    isSubmitting = true;

    const [
      { updateWorkContract, uploadWorkContractFiles },
      { AlertPosition, AlertType, showAlert },
    ] = await Promise.all([
      import("@api/utils"),
      import("@components/alert/alert"),
    ]);

    const files = workContract.files;
    const editedContract = {
      ...workContract,
      files: undefined,
    } as unknown as WorkContract;
    const hasChanged =
      JSON.stringify(editedContract) !== originalWorkContractJson;

    const hasNewFiles = newFiles.length > 0;

    let success = true;

    // Scenario 1: Both work contract data and files have changed
    if (hasChanged && hasNewFiles) {
      const [updateResult, [filesResult, filesBaseId]] = await Promise.all([
        updateWorkContract(workContractId, editedContract),
        uploadWorkContractFiles(workContractId, newFiles),
      ]);

      success = updateResult && filesResult;

      for (let i = 0, len = newFiles.length; i < len; i++) {
        const file = newFiles[i];
        files[filesBaseId + i] = {
          name: file.name,
          path: `media/work_contracts/${workContractId}/${file.name}`,
          uploadedAt: new Date().toLocaleString(),
        };
      }
    }
    // Scenario 2: Only work contract data has changed
    else if (hasChanged) {
      success = await updateWorkContract(workContractId, editedContract);
    }
    // Scenario 3: Only files have changed
    else if (hasNewFiles) {
      const [filesResult, filesBaseId] = await uploadWorkContractFiles(
        workContractId,
        newFiles
      );

      success = filesResult;

      for (let i = 0, len = newFiles.length; i < len; i++) {
        const file = newFiles[i];
        files[filesBaseId + i] = {
          name: file.name,
          path: `media/work_contracts/${workContractId}/${file.name}`,
          uploadedAt: new Date().toLocaleString(),
        };
      }
    }
    // Scenario 4: Nothing has changed
    else {
      showAlert(
        "Nenhuma alteração detetada",
        AlertType.INFO,
        AlertPosition.TOP
      );
      closeModal();
      isSubmitting = false;
      return;
    }

    if (success) {
      showAlert(
        "Contrato de trabalho atualizado com sucesso",
        AlertType.SUCCESS,
        AlertPosition.TOP
      );
      closeModal();
      onWorkContractUpdated({
        ...workContract,
        files,
      });
    } else {
      showAlert(
        "Erro ao atualizar contrato de trabalho",
        AlertType.ERROR,
        AlertPosition.TOP
      );
    }
    isSubmitting = false;
  }

  function showDeleteFileConfirmation(fileId: string) {
    fileToDeleteId = fileId;
    confirmationAction = ConfirmationAction.DELETE_FILE;
    confirmModal.showModal();
  }

  function showDeleteContractConfirmation() {
    confirmationAction = ConfirmationAction.DELETE_WORK_CONTRACT;
    confirmModal.showModal();
  }

  function closeConfirmationModal() {
    confirmModal.close();
    confirmationAction = null;
    fileToDeleteId = null;
  }

  async function handleDeleteConfirmed() {
    isDeleteSubmitting = true;
    try {
      if (
        confirmationAction === ConfirmationAction.DELETE_FILE &&
        fileToDeleteId
      ) {
        await handleDeleteFile();
      } else if (
        confirmationAction === ConfirmationAction.DELETE_WORK_CONTRACT
      ) {
        await handleDeleteWorkContract();
      }
    } finally {
      isDeleteSubmitting = false;
      closeConfirmationModal();
    }
  }

  async function handleDeleteFile() {
    if (!fileToDeleteId) return;

    const [
      { deleteWorkContractFile },
      { showAlert, AlertType, AlertPosition },
    ] = await Promise.all([
      import("@api/utils"),
      import("@components/alert/alert"),
    ]);

    const success = await deleteWorkContractFile(
      workContractId,
      fileToDeleteId
    );

    if (success) {
      // @ts-ignore we don't need to convert fileToDeleteId to number
      delete workContract.files[fileToDeleteId];

      showAlert(
        "Ficheiro eliminado com sucesso",
        AlertType.SUCCESS,
        AlertPosition.TOP
      );
      onFileDeleted(workContractId, fileToDeleteId);
    } else {
      showAlert(
        "Erro ao eliminar ficheiro",
        AlertType.ERROR,
        AlertPosition.TOP
      );
    }
  }

  async function handleDeleteWorkContract() {
    const [{ deleteWorkContract }, { showAlert, AlertType, AlertPosition }] =
      await Promise.all([
        import("@api/utils"),
        import("@components/alert/alert"),
      ]);

    const success = await deleteWorkContract(workContractId);

    if (success) {
      closeModal();
      showAlert(
        "Contrato de trabalho eliminado com sucesso",
        AlertType.SUCCESS,
        AlertPosition.TOP
      );
      onWorkContractDeleted(workContractId);
    } else {
      showAlert(
        "Erro ao eliminar contrato de trabalho",
        AlertType.ERROR,
        AlertPosition.TOP
      );
    }
  }

  function handleFileSelection(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files) return;

    newFiles = [...newFiles, ...Array.from(input.files)];
  }

  function removeNewFile(index: number) {
    newFiles.splice(index, 1);
  }
</script>

<dialog id="work-contract-modal" class="modal" bind:this={modal}>
  <div class="modal-box w-11/12 max-w-5x1">
    <div class="flex justify-between mb-4">
      <h3 class="font-bold text-xl">
        Contrato de Trabalho - {workContract.employeeName}
      </h3>
      <button
        class="btn btn-ghost btn-sm"
        onclick={closeModal}
        disabled={isSubmitting}
      >
        ✕
      </button>
    </div>
    <form onsubmit={handleSubmit} class="space-y-4">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <fieldset class="fieldset">
          <legend class="fieldset-legend">Nome do Funcionário</legend>
          <input
            type="text"
            class="input input-bordered w-full"
            placeholder="Nome do Funcionário"
            bind:value={workContract.employeeName}
            required
          />
        </fieldset>

        <fieldset class="fieldset">
          <legend class="fieldset-legend">NIF</legend>
          <input
            type="number"
            class="input input-bordered w-full"
            placeholder="Número de Identificação Fiscal"
            bind:value={workContract.nif}
            required
          />
        </fieldset>

        <fieldset class="fieldset">
          <legend class="fieldset-legend">Data de Início</legend>
          <DatePicker range={false} bind:value={workContract.dateStartString} />
        </fieldset>

        <fieldset class="fieldset">
          <legend class="fieldset-legend">Data de Fim</legend>
          <DatePicker
            range={false}
            bind:value={workContract.dateEndString}
            required={false}
          />
        </fieldset>

        <fieldset class="fieldset">
          <legend class="fieldset-legend">Categoria</legend>
          <select
            class="select select-bordered w-full"
            bind:value={workContract.categoryId}
            required
          >
            {#each Object.entries($categories) as [id, category]}
              <option value={parseInt(id, 10)}>{category.name}</option>
            {/each}
          </select>
        </fieldset>

        <fieldset class="fieldset">
          <legend class="fieldset-legend">Tipo</legend>
          <select
            class="select select-bordered w-full"
            bind:value={workContract.typeValue}
            required
          >
            {#each WorkContractTypes as type, i}
              <option value={i}>{type}</option>
            {/each}
          </select>
        </fieldset>
      </div>

      <fieldset class="fieldset col-span-1 md:col-span-2">
        <legend class="fieldset-legend">Descrição (opcional)</legend>
        <textarea
          class="textarea textarea-bordered w-full min-h-[80px]"
          placeholder="Informação adicional sobre este contrato..."
          bind:value={workContract.description}
        ></textarea>
      </fieldset>

      <div class="divider">Ficheiros</div>

      {#if existingFiles !== null}
        <div class="overflow-x-auto">
          <table class="table table-compact w-full">
            <thead>
              <tr>
                <th>Nome</th>
                <th>Data de upload</th>
                <th class="w-24">Ações</th>
              </tr>
            </thead>
            <tbody>
              {#each existingFiles as file}
                <tr>
                  <td>{file.name}</td>
                  <td>{file.uploadedAt}</td>
                  <td>
                    <div class="flex justify-end space-x-2">
                      <a
                        href={`${API_BASE_URL}/${file.path}`}
                        target="_blank"
                        class="btn btn-xs btn-outline"
                      >
                        Ver
                      </a>
                      <button
                        type="button"
                        class="btn btn-xs btn-error"
                        disabled={isSubmitting}
                        onclick={() => showDeleteFileConfirmation(file.id)}
                      >
                        Eliminar
                      </button>
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {:else}
        <div class="text-center py-4 text-base-content/70">
          Nenhum ficheiro associado a este contrato.
        </div>
      {/if}

      <div>
        <div class="flex items-center justify-between">
          <h4 class="font-semibold">Novos Ficheiros</h4>
          <button
            type="button"
            class="btn btn-sm btn-secondary"
            disabled={isSubmitting}
            onclick={() => fileInput?.click()}
          >
            Adicionar Ficheiros
          </button>
          <input
            type="file"
            bind:this={fileInput}
            onchange={handleFileSelection}
            class="hidden"
            multiple
            accept="image/*,.doc,.docx,.xls,.xlsx,.pdf"
          />
        </div>

        {#if newFiles.length > 0}
          <div class="mt-2 space-y-2">
            {#each newFiles as file, i}
              <div
                class="flex items-center justify-between p-2 bg-base-200 rounded"
              >
                <span class="text-sm truncate max-w-[80%]">{file.name}</span>
                <button
                  type="button"
                  class="btn btn-xs btn-error"
                  onclick={() => removeNewFile(i)}
                >
                  ×
                </button>
              </div>
            {/each}
          </div>
        {:else}
          <div class="text-center py-4 text-base-content/70">
            Clique no botão acima para adicionar novos ficheiros.
          </div>
        {/if}
      </div>

      <div class="modal-action flex justify-between">
        <button
          type="button"
          class="btn btn-error"
          onclick={showDeleteContractConfirmation}
        >
          Eliminar Contrato de Trabalho
        </button>
        <button type="submit" class="btn btn-primary">
          {#if isSubmitting}
            <span class="loading loading-bars loading-md"></span>
          {:else}
            Guardar alterações
          {/if}
        </button>
      </div>
    </form>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button disabled={isSubmitting} onclick={closeModal}>c</button>
  </form>
</dialog>

<dialog class="modal" bind:this={confirmModal}>
  <div class="modal-box">
    <div class="flex justify-between">
      <h3 class="font-bold text-lg">
        Eliminar
        {#if confirmationAction === ConfirmationAction.DELETE_FILE}
          Ficheiro
        {:else}
          Contrato de Trabalho
        {/if}
      </h3>
      <button
        class="btn btn-ghost btn-sm"
        onclick={closeConfirmationModal}
        disabled={isDeleteSubmitting}
      >
        ✕
      </button>
    </div>
    <p class="py-4">
      {#if confirmationAction === ConfirmationAction.DELETE_FILE}
        Tem certeza que deseja eliminar este ficheiro? Esta ação não pode ser
        desfeita.
      {:else if confirmationAction === ConfirmationAction.DELETE_WORK_CONTRACT}
        <span class="text-error font-bold">ATENÇÃO:</span> Tem certeza que deseja
        eliminar este contrato de trabalho? Esta ação não pode ser desfeita e todos
        os ficheiros associados serão eliminados.
      {/if}
    </p>

    <div class="modal-action flex justify-between">
      <button
        class="btn"
        onclick={closeConfirmationModal}
        disabled={isDeleteSubmitting}
      >
        Cancelar
      </button>
      <button class="btn btn-error" onclick={handleDeleteConfirmed}>
        {#if isDeleteSubmitting}
          <span class="loading loading-bards loading-md"></span>
        {:else}
          Sim, Eliminar
        {/if}
      </button>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button disabled={isDeleteSubmitting} onclick={closeConfirmationModal}
      >c</button
    >
  </form>
</dialog>
