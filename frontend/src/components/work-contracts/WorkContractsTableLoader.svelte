<script lang="ts">
  import Table from "@components/common/Table.svelte";
  import type { TableColumn } from "@lib/types/table";
  import type { WorkContract } from "@lib/types/work-contracts";
  import { onMount } from "svelte";

  let loading = $state(true);
  let workContracts = $state({});
  let workContractCategories = $state({});
  let modal: HTMLDialogElement;

  const columns: TableColumn[] = [
    { header: "ID", field: "ID" },
    { header: "Funcionário", field: "employeeName" },
    { header: "NIF", field: "nif" },
    { header: "Categoria", field: "category" },
    { header: "Local", field: "location" },
    {
      header: "Data Início",
      field: "dateStartString",
      dateValueField: "dateStart",
    },
    { header: "Data Fim", field: "dateEndString", dateValueField: "dateEnd" },
    { header: "Tipo", field: "type" },
  ];

  let selectedWorkContractId: string | null = $state(null);
  let originalWorkContractJson: string | null = $state(null);
  let selectedWorkcontract: WorkContract | null = $state(null);

  function openWorkContractModal(id: string, workContract: WorkContract) {
    selectedWorkContractId = id;
    selectedWorkcontract = $state.snapshot(workContract); // Deep copy to prevent direct mutations
    originalWorkContractJson = JSON.stringify({
      ...selectedWorkcontract,
      files: undefined,
    });

    modal.showModal();
  }

  function handleWorkContractUpdated(updatedWorkContract: WorkContract) {
    // @ts-ignore javascript can take string as indexes
    workContracts[selectedWorkContractId!] = updatedWorkContract;
  }

  function handleWorkContractDeleted(deletedId: string) {
    // @ts-ignore javascript can take string as indexes
    delete workContracts[deletedId];
  }

  function handleFileDeleted(contractId: string, fileId: string) {
    // @ts-ignore javascript can take string as indexes
    delete workContracts[contractId].files[fileId];
  }

  onMount(() => {
    (async () => {
      const [{ getWorkContracts }, { AlertPosition, AlertType, showAlert }] =
        await Promise.all([
          import("@api/utils"),
          import("@components/alert/alert"),
        ]);

      const result = await getWorkContracts();
      if (!result) {
        showAlert(
          "Error ao carregar contratos de trabalho",
          AlertType.ERROR,
          AlertPosition.TOP
        );
        loading = false;
        return;
      }

      [workContracts, workContractCategories] = result;

      modal = document.getElementById(
        "work-contract-modal"
      ) as HTMLDialogElement;

      loading = false;
    })();
  });
</script>

<Table
  data={workContracts}
  {columns}
  {loading}
  emptyMessage="Nenhum contrato de trabalho disponível"
  keyField="ID"
  searchFields={[
    "__searchEmployeeName",
    "__searchType",
    "__searchLocation",
    "__searchCategory",
    "nif",
    "dateStartString",
    "dateEndString",
  ]}
  onRowClick={openWorkContractModal}
/>
