<script lang="ts">
  import {
    type Contract,
    ContractServices,
    ContractLocations,
    ContractStatus,
    ContractTypes,
  } from "@lib/types/contracts";
  import DatePicker from "./DatePicker.svelte";
  import { onMount } from "svelte";

  const {
    contractId,
    contract,
    isVisible,
  }: {
    contractId: string;
    contract: Contract;
    isVisible: boolean;
  } = $props();

  // Local state
  let editedContract: Contract = $state({} as Contract);
  let newFiles = $state<File[]>([]);
  let fileInput = $state<HTMLInputElement | null>(null);
  let isSubmitting = $state(false);

  // Confirmation modal state
  let confirmationAction = $state<"deleteContract" | "deleteFile" | null>(null);
  let fileToDeleteId = $state<string | null>(null);
  let isDeleteSubmitting = $state(false);

  $effect(() => {
    if (contract) {
      console.log("contract", contract);
      editedContract = {
        ...contract,
      };
    }
  });

  // Get existing files as array for easier rendering
  const existingFiles = $derived(
    editedContract.files
      ? // @ts-ignore
        Object.entries(editedContract.files).map(([id, file]) => ({
          id,
          // @ts-ignore
          ...file,
          // @ts-ignore
          uploadedAt: new Date(file.uploadedAt),
        }))
      : null
  );

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();

    try {
      const { updateContract, uploadContractFiles } = await import(
        "@api/utils"
      );
      const success = await updateContract(contractId, editedContract);

      // Upload new files if contract was saved successfully
      if (success && newFiles.length > 0) {
        // TODO: maybe do something with ok here
        await uploadContractFiles(contractId, newFiles);

        // Clear new files list
        newFiles = [];
      }

      if (success) {
        closeModal();
      }
    } catch (error) {
      console.error("Error saving contract:", error);
    } finally {
      isSubmitting = false;
    }
  }

  function showDeleteFileConfirmation(fileId: string) {
    fileToDeleteId = fileId;
    confirmationAction = "deleteFile";
    const confirmModal = document.getElementById(
      "confirm-modal"
    ) as HTMLDialogElement;
    if (confirmModal) confirmModal.showModal();
  }

  function showDeleteContractConfirmation() {
    confirmationAction = "deleteContract";
    const confirmModal = document.getElementById(
      "confirm-modal"
    ) as HTMLDialogElement;
    if (confirmModal) confirmModal.showModal();
  }

  async function handleDeleteConfirmed() {
    isDeleteSubmitting = true;

    try {
      if (confirmationAction === "deleteFile" && fileToDeleteId) {
        await handleDeleteFile();
      } else if (confirmationAction === "deleteContract") {
        await handleDeleteContract();
      }
    } finally {
      isDeleteSubmitting = false;
      closeConfirmationModal();
    }
  }

  async function handleDeleteFile() {
    if (!fileToDeleteId) return;

    try {
      const { deleteContractFile } = await import("@api/utils");
      const success = await deleteContractFile(contractId, fileToDeleteId);

      if (success) {
        // Remove file from editedContract
        const updatedFiles = { ...editedContract.files };
        // @ts-ignore we don't need to convert fileToDeleteId to number here because it is a numeric string and javascript can take that as indexes
        delete updatedFiles[fileToDeleteId];
        editedContract.files = updatedFiles;
      }
    } catch (error) {
      console.error("Error deleting file:", error);
    }
  }

  async function handleDeleteContract() {
    try {
      const { deleteContract } = await import("@api/utils");
      const success = await deleteContract(contractId);

      if (success) {
        closeModal();
        // Refresh the contracts list (you might want to handle this via a callback)
        // TODO: check this
        window.location.reload();
      }
    } catch (error) {
      console.error("Error deleting contract:", error);
    }
  }

  function closeConfirmationModal() {
    const confirmModal = document.getElementById(
      "confirm-modal"
    ) as HTMLDialogElement;
    if (confirmModal) confirmModal.close();
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
    const modal = document.getElementById(
      "contract-modal"
    ) as HTMLDialogElement;
    if (modal) modal.close();
  }
</script>

<dialog id="contract-modal" class="modal">
  <div class="modal-box w-11/12 max-w-5xl">
    {#if isVisible}
      <div class="flex justify-between items-center mb-4">
        <h3 class="font-bold text-xl">
          Contrato #{editedContract.contractNumber} - {editedContract.supplier}
        </h3>

        <button type="button" class="btn btn-ghost btn-sm" onclick={closeModal}>
          ✕
        </button>
      </div>
      <form onsubmit={handleSubmit} class="space-y-4">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <fieldset class="fieldset">
            <legend class="fieldset-legend"> Número do Contrato </legend>
            <input
              type="number"
              name="contractNumber"
              class="input input-bordered w-full"
              value={editedContract.contractNumber}
              required
            />
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Fornecedor</legend>
            <input
              type="text"
              name="supplier"
              class="input input-bordered w-full"
              value={editedContract.supplier}
              required
            />
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Local</legend>
            <select
              name="location"
              class="select select-bordered w-full"
              required
            >
              {#each ContractLocations as location, i}
                <option
                  value={i}
                  selected={editedContract.location === location}
                >
                  {location}
                </option>
              {/each}
            </select>
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Serviço</legend>
            <select
              name="service"
              class="select select-bordered w-full"
              required
            >
              {#each ContractServices as service, i}
                <option value={i} selected={editedContract.service === service}>
                  {service}
                </option>
              {/each}
            </select>
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Data de Início - Fim</legend>
            <DatePicker
              formName="date"
              range={false}
              value={editedContract.dateString}
            />
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Data de Início - Fim</legend>
            <DatePicker
              formName="date-range"
              range={true}
              value={`${editedContract.dateStartString} - ${editedContract.dateEndString}`}
            />
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Tipo</legend>
            <select name="type" class="select select-bordered w-full" required>
              {#each ContractTypes as type, i}
                <option value={i} selected={editedContract.type === type}>
                  {type}
                </option>
              {/each}
            </select>
          </fieldset>

          <fieldset class="fieldset">
            <legend class="fieldset-legend">Estado</legend>
            <select
              name="status"
              class="select select-bordered w-full"
              required
            >
              {#each ContractStatus as status, i}
                <option value={i} selected={editedContract.status === status}>
                  {status}
                </option>
              {/each}
            </select>
          </fieldset>

          <fieldset class="fieldset md:col-span-2">
            <legend class="fieldset-legend">Descrição</legend>
            <textarea
              name="description"
              class="textarea textarea-bordered w-full"
              >{editedContract.description}</textarea
            >
          </fieldset>
        </div>

        <!-- Files section -->
        <div class="divider">Ficheiros</div>

        <!-- Existing files -->
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
                    <td>{file.uploadedAt.toLocaleString("pt-PT")}</td>
                    <td>
                      <div class="flex justify-end space-x-2">
                        <a
                          href={file.path}
                          target="_blank"
                          class="btn btn-xs btn-outline"
                        >
                          Ver
                        </a>
                        <button
                          type="button"
                          class="btn btn-xs btn-error"
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

        <!-- New files section -->
        <div>
          <!-- New files UI remains the same... -->
          <div class="flex items-center justify-between">
            <h4 class="font-semibold">Novos Ficheiros</h4>
            <button
              type="button"
              class="btn btn-sm btn-secondary"
              onclick={openFileSelector}
            >
              Adicionar Ficheiros
            </button>
            <input
              type="file"
              name="newFiles"
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
          <button type="submit" class="btn btn-primary" disabled={isSubmitting}>
            {isSubmitting ? "A guardar..." : "Guardar alterações"}
          </button>
        </div>
      </form>
    {/if}
  </div>
  <form method="dialog" class="modal-backdrop">
    <button>close</button>
  </form>
</dialog>

<!-- Confirmation Modal -->
<dialog id="confirm-modal" class="modal">
  <div class="modal-box">
    <h3 class="font-bold text-lg">
      {#if confirmationAction === "deleteFile"}
        Eliminar Ficheiro
      {:else if confirmationAction === "deleteContract"}
        Eliminar Contrato
      {/if}
    </h3>

    <p class="py-4">
      {#if confirmationAction === "deleteFile"}
        Tem certeza que deseja eliminar este ficheiro? Esta ação não pode ser
        desfeita.
      {:else if confirmationAction === "deleteContract"}
        <span class="text-error font-bold">ATENÇÃO:</span> Tem certeza que deseja
        eliminar este contrato? Esta ação não pode ser desfeita e todos os ficheiros
        associados serão eliminados.
      {/if}
    </p>

    <div class="modal-action">
      <button
        class="btn"
        onclick={closeConfirmationModal}
        disabled={isDeleteSubmitting}
      >
        Cancelar
      </button>
      <button
        class="btn btn-error"
        onclick={handleDeleteConfirmed}
        disabled={isDeleteSubmitting}
      >
        {isDeleteSubmitting ? "Eliminando..." : "Sim, Eliminar"}
      </button>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button onclick={closeConfirmationModal}>close</button>
  </form>
</dialog>
