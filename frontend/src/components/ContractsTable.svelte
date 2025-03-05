<script lang="ts">
  import type { Contracts } from "@lib/types/contracts";
  import { onMount } from "svelte";

  let contracts: Contracts = $state({});
  let currentPage = $state(1);
  let perPage = $state(10);
  let loading = $state(true);

  const contractEntries = $derived.by(() => {
    return Object.entries(contracts);
  });
  const totalItems = $derived(contractEntries.length);
  const totalPages = $derived(Math.ceil(totalItems / perPage));

  function generatePageNumbers(
    current: number,
    total: number
  ): (number | null)[] {
    if (total <= 7) {
      return Array.from({ length: total }, (_, i) => i + 1);
    }

    if (current < 4) {
      return [1, 2, 3, 4, 5, null, total];
    } else if (current > total - 3) {
      return [1, null, total - 4, total - 3, total - 2, total - 1, total];
    } else {
      return [1, null, current - 1, current, current + 1, null, total];
    }
  }

  const displayedContracts = $derived.by(() => {
    const startIndex = (currentPage - 1) * perPage;
    const endIndex = Math.min(startIndex + perPage, totalItems);
    return contractEntries.slice(startIndex, endIndex);
  });

  function goToPage(page: number) {
    if (page >= 1 && page <= totalPages) {
      currentPage = page;
    }
  }

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
      loading = false;
      return;
    }
    contracts = contractsOrNull;
    loading = false;
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
      {#if loading}
        {#each { length: 5 } as _}
          <tr>
            <th><div class="skeleton h-4 w-8"></div></th>
            <td><div class="skeleton h-4 w-28"></div></td>
            <td><div class="skeleton h-4 w-36"></div></td>
            <td><div class="skeleton h-4 w-20"></div></td>
            <td><div class="skeleton h-4 w-20"></div></td>
            <td><div class="skeleton h-4 w-20"></div></td>
            <td><div class="skeleton h-4 w-16"></div></td>
            <td><div class="skeleton h-4 w-16"></div></td>
          </tr>
        {/each}
      {:else}
        {#each displayedContracts as [id, contract]}
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
      {/if}
    </tbody>
  </table>

  {#if loading}
    <div
      class="flex justify-between items-center p-2 bg-base-100 border border-zinc-200 rounded-box"
    >
      <div class="skeleton h-8 w-40"></div>
      <div class="skeleton h-6 w-64"></div>
      <div class="skeleton h-10 w-80"></div>
    </div>
  {:else}
    <div
      class="flex justify-between items-center p-2 bg-base-100 border border-zinc-200 rounded-box"
    >
      <div class="flex items-center gap-2">
        <span>Mostrar</span>
        <label class="join">
          <input
            type="number"
            min="1"
            max="100"
            class="input input-bordered join-item w-20"
            value={perPage}
            onchange={(e) => (perPage = parseInt(e.currentTarget.value) || 10)}
          />
          <span class="join-item flex items-center px-2">por página</span>
        </label>
      </div>

      <span
        >A mostrar {(currentPage - 1) * perPage + 1} a {Math.min(
          currentPage * perPage,
          totalItems
        )} de {totalItems} resultados</span
      >

      <div class="join">
        <button
          class="join-item btn"
          disabled={currentPage === 1}
          onclick={() => goToPage(1)}
        >
          «
        </button>

        <button
          class="join-item btn"
          disabled={currentPage === 1}
          onclick={() => goToPage(currentPage - 1)}
        >
          ‹
        </button>

        {#each generatePageNumbers(currentPage, totalPages) as page}
          {#if page === null}
            <button class="join-item btn btn-disabled border border-zinc-200"
              >...</button
            >
          {:else}
            <button
              class="join-item btn {page === currentPage ? 'btn-active' : ''}"
              onclick={() => goToPage(page)}
            >
              {page}
            </button>
          {/if}
        {/each}

        <button
          class="join-item btn"
          disabled={currentPage === totalPages}
          onclick={() => goToPage(currentPage + 1)}
        >
          ›
        </button>

        <button
          class="join-item btn"
          disabled={currentPage === totalPages}
          onclick={() => goToPage(totalPages)}
        >
          »
        </button>
      </div>
    </div>
  {/if}
</div>
