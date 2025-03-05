<script lang="ts">
  import type { Contracts } from "@lib/types/contracts";
  import { onMount } from "svelte";

  let contracts: Contracts = $state({});
  onMount(async () => {
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
      return;
    }
    contracts = contractsOrNull;
  });
</script>

<div
  class="overflow-x-auto rounded-box border border-base-content/5 bg-base-200"
>
  <table class="table">
    <thead>
      <tr>
        <th>ID</th>
        <th>Fornecedor</th>
        <th>Número de Contrato</th>
        <th>Data</th>
        <th>Data Início</th>
        <th>Data Fim</th>
        <th>Tipo</th>
        <th>Status</th>
      </tr>
    </thead>
    <tbody>
      {#each Object.entries(contracts) as [id, contract]}
        <tr class="hover:bg-base-300">
          <th>{id}</th>
          <td>{contract.supplier}</td>
          <td>{contract.contractNumber}</td>
          <td>{contract.date}</td>
          <td>{contract.dateStart}</td>
          <td>{contract.dateEnd}</td>
          <td>{contract.type}</td>
          <td>{contract.status}</td>
        </tr>
      {/each}
    </tbody>
  </table>
  <div class="flex justify-between items-center p-2 bg-base-100">
    <span>A mostrar 1 a 10 de 97 resultados</span>
    <div class="join">
      <button class="join-item btn">1</button>
      <button class="join-item btn">2</button>
      <button class="join-item btn btn-disabled">...</button>
      <button class="join-item btn">99</button>
      <button class="join-item btn">100</button>
    </div>
  </div>
</div>
