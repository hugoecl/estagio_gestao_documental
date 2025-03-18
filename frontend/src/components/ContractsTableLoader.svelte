<script lang="ts">
  import type { Contract } from "@lib/types/contracts";
  import { onMount } from "svelte";
  import Table from "@components/Table.svelte";
  import type { TableColumn } from "@lib/types/table";
  import ContractModal from "@components/ContractModal.svelte";
  import { newContract } from "@stores/contracts-stores";

  let loading = $state(true);
  let contracts = $state({});
  let modal: HTMLDialogElement;
  const columns: TableColumn[] = [
    { header: "ID", field: "ID" },
    { header: "Fornecedor", field: "supplier" },
    { header: "Serviço", field: "service" },
    { header: "Número de Contrato", field: "contractNumber", responsive: "" },
    { header: "Data", field: "dateString", dateValueField: "date" },
    {
      header: "Data Início",
      field: "dateStartString",
      dateValueField: "dateStart",
    },
    { header: "Data Fim", field: "dateEndString", dateValueField: "dateEnd" },
    { header: "Tipo", field: "type" },
    { header: "Estado", field: "status" },
  ];

  let selectedContractId: string | null = $state(null);
  let originalContractJson: string | null = $state(null);
  let selectedContract: Contract | null = $state(null);

  function openContractModal(id: string, contract: Contract) {
    selectedContractId = id;
    selectedContract = $state.snapshot(contract); // Deep copy to prevent direct mutations
    originalContractJson = JSON.stringify({
      ...selectedContract,
      files: undefined,
    });

    modal.showModal();
  }

  function handleContractUpdated(updatedContract: Contract) {
    // @ts-ignore javascript can take string as indexes
    contracts[selectedContractId!] = updatedContract;
  }

  function handleContractDeleted(deletedId: string) {
    // @ts-ignore javascript can take string as indexes
    delete contracts[deletedId];
  }

  function handleFileDeleted(contractId: string, fileId: string) {
    // @ts-ignore javascript can take string as indexes
    delete contracts[contractId].files[fileId];
  }

  onMount(() => {
    (async () => {
      const [{ getContracts }, { AlertPosition, AlertType, showAlert }] =
        await Promise.all([
          import("@api/utils"),
          import("@components/Alert/Alert"),
        ]);
      const contractsOrNull = await getContracts();
      if (!contractsOrNull) {
        showAlert(
          "Erro ao carregar contratos",
          AlertType.ERROR,
          AlertPosition.TOP
        );
        loading = false;
        return;
      }
      contracts = contractsOrNull;
      modal = document.getElementById("contract-modal") as HTMLDialogElement;

      loading = false;
    })();

    const unsubscribe = newContract.subscribe((contract) => {
      if (contract) {
        // @ts-ignore javascript can take string as indexes
        contracts[contract.id] = contract.contract;
        newContract.set(null);
      }
    });

    return () => {
      unsubscribe();
    };
  });
</script>

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

<ContractModal
  contractId={selectedContractId!}
  contract={selectedContract ? selectedContract : ({} as Contract)}
  origianlContractJson={originalContractJson!}
  onContractUpdated={handleContractUpdated}
  onContractDeleted={handleContractDeleted}
  onFileDeleted={handleFileDeleted}
/>
