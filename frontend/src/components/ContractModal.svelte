<script lang="ts">
  import {
    type Contract,
    ContractServices,
    ContractLocations,
    ContractStatus,
    ContractTypes,
  } from "@lib/types/contracts";
  import DatePicker from "@components/DatePicker.svelte";
  import API_BASE_URL from "@api/base-url";

  const {
    contractId,
    contract,
    origianlContractJson,
    isVisible,
    onContractUpdated,
    onContractDeleted,
    onFileDeleted,
  }: {
    // TODO: see passing this as a number instead of a string
    contractId: string;
    contract: Contract;
    origianlContractJson: string;
    isVisible: boolean;
    onContractUpdated: (updatedContract: Contract) => void;
    onContractDeleted: (deletedId: string) => void;
    onFileDeleted: (contractId: string, fileId: string) => void;
  } = $props();

  let modal: HTMLDialogElement;
  let confirmModal: HTMLDialogElement;

  let newFiles = $state<File[]>([]);
  let fileInput = $state<HTMLInputElement | null>(null);
  let isSubmitting = $state(false);

  const enum ConfirmationAction {
    DELETE_CONTRACT,
    DELETE_FILE,
  }
  let confirmationAction = $state<ConfirmationAction | null>(null);
  let fileToDeleteId = $state<string | null>(null);
  let isDeleteSubmitting = $state(false);

  const existingFiles = $derived(
    contract.files
      ? // @ts-ignore
        Object.entries(contract.files).map(([id, file]) => ({
          id,
          // @ts-ignore
          ...file,
          // @ts-ignore
          uploadedAt: file.uploadedAt,
        }))
      : null
  );

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    isSubmitting = true;

    const [
      { updateContract, uploadContractFiles },
      { showAlert, AlertType, AlertPosition },
    ] = await Promise.all([
      import("@api/utils"),
      import("@components/Alert/Alert"),
    ]);

    const files = contract.files;
    const editedContract = {
      ...contract,
      files: undefined,
    } as unknown as Contract;
    const hasContractChanged =
      JSON.stringify(editedContract) !== origianlContractJson;

    const hasNewFiles = newFiles.length > 0;

    let success = true;

    // Scenario 1: Both contract data and files have changed
    if (hasContractChanged && hasNewFiles) {
      const [contractResult, [filesResult, filesBaseId]] = await Promise.all([
        updateContract(contractId, editedContract),
        uploadContractFiles(contractId, newFiles),
      ]);

      success = contractResult && filesResult;

      for (let i = 0, len = newFiles.length; i < len; i++) {
        const file = newFiles[i];
        files[filesBaseId + i] = {
          name: file.name,
          path: `media/contracts/${contractId}/${file.name}`,
          uploadedAt: new Date().toLocaleDateString("pt-PT"),
        };
      }
    }
    // Scenario 2: Only contract data has changed
    else if (hasContractChanged) {
      success = await updateContract(contractId, editedContract);
    }
    // Scenario 3: Only files have changed
    else if (hasNewFiles) {
      const result = await uploadContractFiles(contractId, newFiles);
      success = result[0];
      const filesBaseId = result[1];

      for (let i = 0, len = newFiles.length; i < len; i++) {
        const file = newFiles[i];
        files[filesBaseId + i] = {
          name: file.name,
          path: `media/contracts/${contractId}/${file.name}`,
          uploadedAt: new Date().toLocaleString("pt-PT"),
        };
      }
    }
    // Scenario 4: Nothing has changed
    else {
      closeModal();
      showAlert(
        "Nenhuma alteração detetada",
        AlertType.INFO,
        AlertPosition.TOP
      );
      isSubmitting = false;
      return;
    }

    if (success) {
      closeModal();
      showAlert(
        "Contrato atualizado com sucesso",
        AlertType.SUCCESS,
        AlertPosition.TOP
      );
      onContractUpdated({ ...editedContract, files });
    }
    isSubmitting = false;
  }

  function showDeleteFileConfirmation(fileId: string) {
    fileToDeleteId = fileId;
    confirmationAction = ConfirmationAction.DELETE_FILE;
    confirmModal.showModal();
  }

  function showDeleteContractConfirmation() {
    confirmationAction = ConfirmationAction.DELETE_CONTRACT;
    confirmModal.showModal();
  }

  async function handleDeleteConfirmed() {
    isDeleteSubmitting = true;

    try {
      if (
        confirmationAction === ConfirmationAction.DELETE_FILE &&
        fileToDeleteId
      ) {
        await handleDeleteFile();
      } else if (confirmationAction === ConfirmationAction.DELETE_CONTRACT) {
        await handleDeleteContract();
      }
    } finally {
      isDeleteSubmitting = false;
      closeConfirmationModal();
    }
  }

  async function handleDeleteFile() {
    if (!fileToDeleteId) return;

    const [{ deleteContractFile }, { showAlert, AlertType, AlertPosition }] =
      await Promise.all([
        import("@api/utils"),
        import("@components/Alert/Alert"),
      ]);
    const success = await deleteContractFile(contractId, fileToDeleteId);

    if (success) {
      // @ts-ignore we don't need to convert fileToDeleteId to number here because it is a numeric string and javascript can take that as indexes
      delete contract.files[fileToDeleteId];

      showAlert(
        "Ficheiro eliminado com sucesso",
        AlertType.SUCCESS,
        AlertPosition.TOP
      );
      onFileDeleted(contractId, fileToDeleteId);
    } else {
      showAlert(
        "Erro ao eliminar ficheiro",
        AlertType.ERROR,
        AlertPosition.TOP
      );
    }
  }

  async function handleDeleteContract() {
    const [{ deleteContract }, { showAlert, AlertType, AlertPosition }] =
      await Promise.all([
        import("@api/utils"),
        import("@components/Alert/Alert"),
      ]);
    const success = await deleteContract(contractId);

    if (success) {
      closeModal();
      showAlert(
        "Contrato eliminado com sucesso",
        AlertType.SUCCESS,
        AlertPosition.TOP
      );

      onContractDeleted(contractId);
    } else {
      showAlert(
        "Erro ao eliminar contrato",
        AlertType.ERROR,
        AlertPosition.TOP
      );
    }
  }

  function closeConfirmationModal() {
    confirmModal.close();
    confirmationAction = null;
    fileToDeleteId = null;
  }

  function openFileSelector() {
    fileInput?.click();
  }

  function handleFileSelection(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files) {
      newFiles = [...newFiles, ...Array.from(input.files)];
    }
  }

  function removeNewFile(index: number) {
    newFiles = newFiles.filter((_, i) => i !== index);
  }

  function closeModal() {
    modal.close();
    newFiles = [];
  }
</script>

<dialog id="contract-modal" class="modal" bind:this={modal}>
  <div class="modal-box w-11/12 max-w-5xl">
    {#if isVisible}
      <div class="flex justify-between mb-4">
        <h3 class="font-bold text-xl">
          Contrato #{contract.contractNumber} - {contract.supplier}
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
            <legend class="fieldset-legend"> Número do Contrato </legend>
            <input
              type="number"
              class="input input-bordered w-full"
              bind:value={contract.contractNumber}
              required
            />
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Fornecedor</legend>
            <input
              type="text"
              class="input input-bordered w-full"
              bind:value={contract.supplier}
              required
            />
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Local</legend>
            <select
              class="select select-bordered w-full"
              required
              bind:value={contract.locationValue}
            >
              {#each ContractLocations as location, i}
                <option value={i}>
                  {location}
                </option>
              {/each}
            </select>
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Serviço</legend>
            <select
              class="select select-bordered w-full"
              required
              bind:value={contract.serviceValue}
            >
              {#each ContractServices as service, i}
                <option value={i}>
                  {service}
                </option>
              {/each}
            </select>
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Data de Início - Fim</legend>
            <DatePicker range={false} bind:value={contract.dateString} />
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Data de Início - Fim</legend>
            <DatePicker
              range={true}
              bind:value={
                () => `${contract.dateStartString} - ${contract.dateEndString}`,
                (value) => {
                  const start = value.slice(0, 10);
                  const end = value.slice(13, 23);

                  contract.dateStartString = start;
                  contract.dateEndString = end;
                }
              }
            />
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Tipo</legend>
            <select
              class="select select-bordered w-full"
              required
              bind:value={contract.typeValue}
            >
              {#each ContractTypes as type, i}
                <option value={i}>
                  {type}
                </option>
              {/each}
            </select>
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Estado</legend>
            <select
              class="select select-bordered w-full"
              required
              bind:value={contract.statusValue}
            >
              {#each ContractStatus as status, i}
                <option value={i}>
                  {status}
                </option>
              {/each}
            </select>
          </fieldset>

          <fieldset class="fieldset md:col-span-2">
            <legend class="fieldset-legend">Descrição</legend>
            <textarea
              class="textarea textarea-bordered w-full"
              bind:value={contract.description}
            ></textarea>
          </fieldset>
        </div>

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
              onclick={openFileSelector}
            >
              Adicionar Ficheiros
            </button>
            <input
              type="file"
              bind:this={fileInput}
              onchange={handleFileSelection}
              class="hidden"
              multiple
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
                    class="btn btn-xs btn-ghost"
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
            disabled={isSubmitting}
          >
            Eliminar Contrato
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
    {/if}
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
        {:else if confirmationAction === ConfirmationAction.DELETE_CONTRACT}
          Contrato
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
      {:else if confirmationAction === ConfirmationAction.DELETE_CONTRACT}
        <span class="text-error font-bold">ATENÇÃO:</span> Tem certeza que deseja
        eliminar este contrato? Esta ação não pode ser desfeita e todos os ficheiros
        associados serão eliminados.
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
          <span class="loading loading-bars loading-md"></span>
        {:else}
          Sim, Eliminar
        {/if}
      </button>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button onclick={closeConfirmationModal} disabled={isDeleteSubmitting}
      >c</button
    >
  </form>
</dialog>
