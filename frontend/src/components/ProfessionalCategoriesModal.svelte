<script lang="ts">
  import type { WorkContractCategory } from "@lib/types/work-contracts";

  const {
    categoryId,
    category,
    originalCategoryJson,
    onCategoryUpdated,
    onCategoryDeleted,
  }: {
    categoryId: string;
    category: WorkContractCategory;
    originalCategoryJson: string;
    onCategoryUpdated: (updatedCategory: WorkContractCategory) => void;
    onCategoryDeleted: (deletedId: string) => void;
  } = $props();

  let modal: HTMLDialogElement;
  let confirmModal: HTMLDialogElement;

  let isSubmitting = $state(false);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    isSubmitting = true;
    const [
      { updateWorkContractCategory },
      { AlertPosition, AlertType, showAlert },
    ] = await Promise.all([
      import("@api/utils"),
      import("@components/Alert/Alert"),
    ]);

    const hasCategoryChanged =
      JSON.stringify(category) !== originalCategoryJson;
    if (!hasCategoryChanged) {
      showAlert(
        "Nenhuma alteração detetada",
        AlertType.INFO,
        AlertPosition.TOP
      );
      isSubmitting = false;
      modal.close();
      return;
    }

    const ok = await updateWorkContractCategory(categoryId, category);
    if (ok) {
      category.updatedAt = new Date().toLocaleString("pt-PT");
      onCategoryUpdated(category);
      modal.close();
      showAlert(
        "Categoria atualizada com sucesso",
        AlertType.SUCCESS,
        AlertPosition.TOP
      );
    } else {
      showAlert(
        "Erro ao atualizar categoria",
        AlertType.ERROR,
        AlertPosition.TOP
      );
    }
    isSubmitting = false;
  }

  async function handleDeleteConfirmed() {
    isSubmitting = true;
    const [
      { deleteWorkContractCategory },
      { AlertPosition, AlertType, showAlert },
    ] = await Promise.all([
      import("@api/utils"),
      import("@components/Alert/Alert"),
    ]);

    const ok = await deleteWorkContractCategory(categoryId);
    if (ok) {
      onCategoryDeleted(categoryId);
      confirmModal.close();
      showAlert(
        "Categoria eliminada com sucesso",
        AlertType.SUCCESS,
        AlertPosition.TOP
      );
    } else {
      showAlert(
        "Erro ao eliminar categoria",
        AlertType.ERROR,
        AlertPosition.TOP
      );
    }
    isSubmitting = false;
  }
</script>

<dialog id="category-modal" class="modal" bind:this={modal}>
  <div class="modal-box w-11/12 max-w-5x1">
    <div class="flex justify-between mb-4">
      <h3 class="font-bold text-xl">
        Categoria - {category.name}
      </h3>
      <button
        class="btn btn-ghost btn-sm"
        onclick={() => modal.close()}
        disabled={isSubmitting}
      >
        ✕
      </button>
    </div>
    <form onsubmit={handleSubmit} class="space-y-4">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <fieldset class="fieldset">
          <legend class="fieldset-legend">Nome</legend>
          <input
            type="text"
            class="input input-bordered w-full"
            placeholder="Nome da Categoria"
            bind:value={category.name}
            required
          />
        </fieldset>

        <fieldset class="fieldset">
          <legend class="fieldset-legend">Descrição</legend>
          <textarea
            class="textarea textarea-bordered w-full"
            placeholder="Descrição da Categoria"
            bind:value={category.description}
          ></textarea>
        </fieldset>
      </div>

      <div class="modal-action flex justify-between">
        <button
          type="button"
          class="btn btn-error"
          onclick={() => confirmModal.showModal()}
          disabled={isSubmitting}
        >
          Eliminar Categoria
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
    <button disabled={isSubmitting} onclick={() => modal.close()}>c</button>
  </form>
</dialog>

<dialog class="modal" bind:this={confirmModal}>
  <div class="modal-box">
    <div class="flex justify-between">
      <h3 class="font-bold text-xl">Eliminar Categoria</h3>

      <button
        class="btn btn-ghost btn-sm"
        onclick={() => confirmModal.close()}
        disabled={isSubmitting}
      >
        ✕
      </button>
    </div>

    <p class="py-4">
      <span class="text-error font-bold">ATENÇÃO:</span> Tem a certeza que
      deseja eliminar a categoria
      <span class="font-bold">{category.name}</span>? Esta ação é irreversível.
    </p>

    <div class="modal-action flex justify-between">
      <button
        class="btn"
        onclick={() => confirmModal.close()}
        disabled={isSubmitting}
      >
        Cancelar
      </button>
      <button
        class="btn btn-error"
        onclick={handleDeleteConfirmed}
        disabled={isSubmitting}
      >
        {#if isSubmitting}
          <span class="loading loading-bars loading-md"></span>
        {:else}
          Sim, Eliminar
        {/if}
      </button>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button
      onclick={() => {
        confirmModal.close();
        modal.close();
      }}
      disabled={isSubmitting}>c</button
    >
  </form>
</dialog>
