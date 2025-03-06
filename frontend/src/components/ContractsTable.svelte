<script lang="ts">
  import type { Contract, Contracts } from "@lib/types/contracts";
  import { onMount } from "svelte";

  let contracts: Contracts = $state({});
  let currentPage = $state(1);
  let perPage = $state(10);
  let loading = $state(true);
  let searchQuery = $state("");

  const enum SortDirection {
    NONE,
    ASC,
    DESC,
  }

  const enum SortableColumn {
    ID = "id",
    SUPPLIER = "supplier",
    LOCATION = "location",
    SERVICE = "service",
    CONTRACT_NUMBER = "contractNumber",
    DATE = "date",
    DATE_START = "dateStart",
    DATE_END = "dateEnd",
    TYPE = "type",
    STATUS = "status",
  }

  let sortColumn = $state<SortableColumn | null>(null);
  let sortDirection = $state<SortDirection>(SortDirection.NONE);

  function getSortIndicator(column: SortableColumn): string {
    if (sortColumn !== column) return "";
    return sortDirection === SortDirection.ASC ? "↑" : "↓";
  }

  function toggleSort(column: SortableColumn): void {
    if (sortColumn === column) {
      // Cycle through: ASC -> DESC -> NONE
      sortDirection =
        sortDirection === SortDirection.ASC
          ? SortDirection.DESC
          : sortDirection === SortDirection.DESC
            ? SortDirection.NONE
            : SortDirection.ASC;

      if (sortDirection === SortDirection.NONE) {
        sortColumn = null;
      }
    } else {
      sortColumn = column;
      sortDirection = SortDirection.ASC;
    }

    currentPage = 1;
  }

  let contractEntries: [string, Contract][] = $state([]);

  const filteredContractEntries = $derived.by(() => {
    if (!searchQuery.trim()) return contractEntries;

    const query = searchQuery.toLowerCase();
    const result = [];

    for (let i = 0, len = contractEntries.length; i < len; i++) {
      const entry = contractEntries[i];
      const [id, contract] = entry;

      if (
        id.includes(query) ||
        contract.__searchSupplier.includes(query) ||
        contract.__searchLocation.includes(query) ||
        contract.__searchService.includes(query) ||
        contract.__searchContractNumber.includes(query) ||
        contract.dateString.includes(query) ||
        contract.dateStartString.includes(query) ||
        contract.dateEndString.includes(query) ||
        contract.__searchType.includes(query) ||
        contract.__searchStatus.includes(query)
      ) {
        result.push(entry);
      }
    }

    return result;
  });

  const sortedContractEntries = $derived.by(() => {
    if (sortColumn === null || sortDirection === SortDirection.NONE) {
      return filteredContractEntries;
    }

    return [...filteredContractEntries].sort((a, b) => {
      const [idA, contractA] = a;
      const [idB, contractB] = b;

      // Special case for ID column which is the key, not in the contract object
      if (sortColumn === SortableColumn.ID) {
        // @ts-ignore we don't need to convert to number javascript does the math anyways with less overhead
        return sortDirection === SortDirection.ASC ? idA - idB : idB - idA;
      }

      const valueA = contractA[sortColumn as keyof typeof contractA];
      const valueB = contractB[sortColumn as keyof typeof contractB];

      if (sortColumn === SortableColumn.CONTRACT_NUMBER) {
        return sortDirection === SortDirection.ASC
          ? // @ts-ignore we don't need to convert to number javascript does the math anyways with less overhead
            valueA - valueB
          : // @ts-ignore we don't need to convert to number javascript does the math anyways with less overhead
            valueB - valueA;
      }

      // Handle dates
      if (
        sortColumn === SortableColumn.DATE ||
        sortColumn === SortableColumn.DATE_START ||
        sortColumn === SortableColumn.DATE_END
      ) {
        return sortDirection === SortDirection.ASC
          ? // @ts-ignore values are Dates here
            valueA.getTime() - valueB.getTime()
          : // @ts-ignore values are Dates here
            valueB.getTime() - valueA.getTime();
      }

      // Handle strings
      // the values are guaranteed to be strings here
      const strA = (valueA as string).toLowerCase();
      const strB = (valueB as string).toLowerCase();
      return sortDirection === SortDirection.ASC
        ? strA.localeCompare(strB, "pt-PT")
        : strB.localeCompare(strA, "pt-PT");
    });
  });

  const totalItems = $derived(sortedContractEntries.length);
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
    return sortedContractEntries.slice(startIndex, endIndex);
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
    contractEntries = Object.entries(contracts);
    loading = false;
  });
</script>

<div
  class="overflow-x-auto rounded-box border border-base-content/5 bg-base-200"
>
  <div class="p-2 flex justify-center">
    <div
      class="join w-full max-w-md mx-auto border border-zinc-300 rounded-box"
    >
      <div class="join-item flex items-center px-3 bg-base-100">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 20 20"
          fill="currentColor"
          class="w-4 h-4"
        >
          <path
            fill-rule="evenodd"
            d="M9 3.5a5.5 5.5 0 100 11 5.5 5.5 0 000-11zM2 9a7 7 0 1112.452 4.391l3.328 3.329a.75.75 0 11-1.06 1.06l-3.329-3.328A7 7 0 012 9z"
            clip-rule="evenodd"
          />
        </svg>
      </div>
      <input
        type="text"
        placeholder="Pesquisar contratos..."
        class="input input-bordered join-item w-full"
        bind:value={searchQuery}
      />
      <button
        class="btn join-item bg-base-300"
        onclick={() => (searchQuery = "")}
        disabled={!searchQuery}
      >
        ×
      </button>
    </div>
  </div>

  <table class="table table-pin-rows">
    <thead>
      <tr>
        <th
          onclick={() => toggleSort(SortableColumn.ID)}
          class="cursor-pointer"
        >
          ID {getSortIndicator(SortableColumn.ID)}
        </th>
        <th
          onclick={() => toggleSort(SortableColumn.SUPPLIER)}
          class="cursor-pointer"
        >
          Fornecedor {getSortIndicator(SortableColumn.SUPPLIER)}
        </th>
        <th
          onclick={() => toggleSort(SortableColumn.LOCATION)}
          class="hidden sm:table-cell cursor-pointer"
        >
          Local {getSortIndicator(SortableColumn.LOCATION)}
        </th>
        <th
          onclick={() => toggleSort(SortableColumn.SERVICE)}
          class="hidden sm:table-cell cursor-pointer"
        >
          Serviço {getSortIndicator(SortableColumn.SERVICE)}
        </th>
        <th
          onclick={() => toggleSort(SortableColumn.CONTRACT_NUMBER)}
          class="cursor-pointer"
        >
          Número de Contrato {getSortIndicator(SortableColumn.CONTRACT_NUMBER)}
        </th>
        <th
          onclick={() => toggleSort(SortableColumn.DATE)}
          class="hidden md:table-cell cursor-pointer"
        >
          Data {getSortIndicator(SortableColumn.DATE)}
        </th>
        <th
          onclick={() => toggleSort(SortableColumn.DATE_START)}
          class="hidden lg:table-cell cursor-pointer"
        >
          Data Início {getSortIndicator(SortableColumn.DATE_START)}
        </th>
        <th
          onclick={() => toggleSort(SortableColumn.DATE_END)}
          class="hidden lg:table-cell cursor-pointer"
        >
          Data Fim {getSortIndicator(SortableColumn.DATE_END)}
        </th>
        <th
          onclick={() => toggleSort(SortableColumn.TYPE)}
          class="hidden md:table-cell cursor-pointer"
        >
          Tipo {getSortIndicator(SortableColumn.TYPE)}
        </th>
        <th
          onclick={() => toggleSort(SortableColumn.STATUS)}
          class="cursor-pointer"
        >
          Status {getSortIndicator(SortableColumn.STATUS)}
        </th>
      </tr>
    </thead>
    <tbody>
      {#if loading}
        {#each { length: 5 } as _}
          <tr>
            <th>
              <div class="skeleton h-4 w-4 md:w-6"></div>
            </th>
            <td>
              <div class="skeleton h-4 w-10 sm:w-16 md:w-24"></div>
            </td>
            <td class="hidden sm:table-cell">
              <div class="skeleton h-4 w-12 md:w-18"></div>
            </td>
            <td class="hidden sm:table-cell">
              <div class="skeleton h-4 w-12 md:w-18"></div>
            </td>
            <td>
              <div class="skeleton h-4 w-10 sm:w-16 md:w-20"></div>
            </td>
            <td class="hidden md:table-cell">
              <div class="skeleton h-4 w-16"></div>
            </td>
            <td class="hidden lg:table-cell">
              <div class="skeleton h-4 w-16"></div>
            </td>
            <td class="hidden lg:table-cell">
              <div class="skeleton h-4 w-16"></div>
            </td>
            <td class="hidden md:table-cell">
              <div class="skeleton h-4 w-10"></div>
            </td>
            <td>
              <div class="skeleton h-4 w-8 md:w-12"></div>
            </td>
          </tr>
        {/each}
      {:else if totalItems === 0}
        <tr>
          <td colspan="10" class="text-center py-8 text-base-content/70">
            {searchQuery
              ? "Nenhum resultado encontrado"
              : "Nenhum contrato disponível"}
          </td>
        </tr>
      {:else}
        {#each displayedContracts as [id, contract]}
          <tr class="hover:bg-base-300">
            <th>{id}</th>
            <td>{contract.supplier}</td>
            <td class="hidden sm:table-cell">{contract.location}</td>
            <td class="hidden sm:table-cell">{contract.service}</td>
            <td>{contract.contractNumber}</td>
            <td class="hidden md:table-cell">{contract.dateString}</td>
            <td class="hidden lg:table-cell">{contract.dateStartString}</td>
            <td class="hidden lg:table-cell">{contract.dateEndString}</td>
            <td class="hidden md:table-cell">{contract.type}</td>
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
      <div class="skeleton h-8 w-20 sm:w-40"></div>
      <div class="skeleton h-6 w-32 sm:w-64"></div>
      <div class="skeleton h-10 w-40 sm:w-80"></div>
    </div>
  {:else}
    <div
      class="flex flex-wrap justify-between items-center gap-2 p-2 bg-base-100 border border-zinc-200 rounded-box"
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

      <span class="text-sm md:text-base">
        {totalItems === 0
          ? "Sem resultados"
          : `A mostrar ${(currentPage - 1) * perPage + 1} a ${Math.min(
              currentPage * perPage,
              totalItems
            )} de ${totalItems} resultados`}
      </span>

      <div class="join max-sm:w-full justify-center">
        <button
          class="join-item btn btn-sm md:btn-md"
          disabled={currentPage === 1}
          onclick={() => goToPage(1)}
        >
          «
        </button>

        <button
          class="join-item btn btn-sm md:btn-md"
          disabled={currentPage === 1}
          onclick={() => goToPage(currentPage - 1)}
        >
          ‹
        </button>

        {#each generatePageNumbers(currentPage, totalPages) as page}
          {#if page === null}
            <button
              class="join-item btn btn-sm md:btn-md btn-disabled border border-zinc-200"
              >...</button
            >
          {:else}
            <button
              class="join-item btn btn-sm md:btn-md {page === currentPage
                ? 'btn-active'
                : ''}"
              onclick={() => goToPage(page)}
            >
              {page}
            </button>
          {/if}
        {/each}

        <button
          class="join-item btn btn-sm md:btn-md"
          disabled={currentPage === totalPages}
          onclick={() => goToPage(currentPage + 1)}
        >
          ›
        </button>

        <button
          class="join-item btn btn-sm md:btn-md"
          disabled={currentPage === totalPages}
          onclick={() => goToPage(totalPages)}
        >
          »
        </button>
      </div>
    </div>
  {/if}
</div>
