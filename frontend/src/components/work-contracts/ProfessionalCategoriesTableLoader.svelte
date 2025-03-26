<script lang="ts">
  import type { WorkContractCategory } from "@lib/types/work-contracts";
  import { onMount } from "svelte";
  import Table from "@components/common/Table.svelte";
  import ProfessionalCategoriesModal from "./ProfessionalCategoriesModal.svelte";
  import { newCategory } from "@stores/work-contract-stores";
  import type { TableColumn } from "@lib/types/table";

  let loading = $state(true);
  let categories = $state({});
  let modal: HTMLDialogElement;
  const columns: TableColumn[] = [
    { header: "ID", field: "ID" },
    { header: "Nome", field: "name" },
    { header: "Descrição", field: "description" },
    {
      header: "Criado em",
      field: "createdAt",
      dateValueField: "__createdAtDate",
    },
    {
      header: "Atualizado em",
      field: "updatedAt",
      dateValueField: "__updatedAtDate",
    },
  ];

  let selectedCategoryId: string | null = $state(null);
  let originalCategoryJson: string | null = $state(null);
  let selectedCategory: WorkContractCategory | null = $state(null);

  function openCategoryModal(id: string, category: WorkContractCategory) {
    selectedCategoryId = id;
    selectedCategory = $state.snapshot(category); // Deep copy to prevent direct mutations
    originalCategoryJson = JSON.stringify(selectedCategory);
    modal.showModal();
  }

  function handleCategoryUpdated(updatedCategory: WorkContractCategory) {
    // @ts-ignore javascript can take string as indexes
    categories[selectedCategoryId!] = updatedCategory;
  }

  function handleCategoryDeleted(deletedId: string) {
    // @ts-ignore javascript can take string as indexes
    delete categories[deletedId];
  }

  onMount(() => {
    (async () => {
      const [
        { getWorkContractCategories },
        { AlertPosition, AlertType, showAlert },
      ] = await Promise.all([
        import("@api/work-contracts-api"),
        import("@components/alert/alert"),
      ]);

      const categoriesOrNull = await getWorkContractCategories();
      if (!categoriesOrNull) {
        showAlert(
          "Erro ao carregar categorias",
          AlertType.ERROR,
          AlertPosition.TOP
        );
        loading = false;
        return;
      }

      categories = categoriesOrNull;
      modal = document.getElementById("category-modal") as HTMLDialogElement;

      loading = false;
    })();

    const unsubscribe = newCategory.subscribe((category) => {
      if (category) {
        // @ts-ignore javascript can take string as indexes
        categories[category.id] = category.category;
        newCategory.set(null);
      }
    });

    return () => {
      unsubscribe();
    };
  });
</script>

<Table
  data={categories}
  {columns}
  {loading}
  emptyMessage="Nenhuma categoria disponível"
  keyField="ID"
  searchFields={[
    "__searchName",
    "__searchDescription",
    "createdAt",
    "updatedAt",
  ]}
  onRowClick={openCategoryModal}
/>

<ProfessionalCategoriesModal
  categoryId={selectedCategoryId!}
  category={selectedCategory ? selectedCategory : ({} as WorkContractCategory)}
  originalCategoryJson={originalCategoryJson!}
  onCategoryUpdated={handleCategoryUpdated}
  onCategoryDeleted={handleCategoryDeleted}
/>
