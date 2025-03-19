<script lang="ts">
  import type { TableColumn } from "@lib/types/table";

  const {
    data,
    columns,
    keyField,
    searchFields,
    loading = false,
    emptyMessage,
    searchEmptyMessage = "Nenhum resultado encontrado",
    rowClassName = "hover:bg-base-300",
    onRowClick,
  }: {
    data: Record<string, any> | [string, Record<string, any>][];
    columns: TableColumn[];
    keyField: string;
    searchFields: string[];
    loading: boolean;
    emptyMessage: string;
    searchEmptyMessage?: string;
    rowClassName?: string;
    onRowClick: (id: string, row: any) => void;
  } = $props();

  const enum SortDirection {
    NONE,
    ASC,
    DESC,
  }

  let currentPage = $state(1);
  let perPage = $state(10);
  let searchQuery = $state("");
  let sortColumn = $state<string | null>(null);
  let sortDirection = $state<SortDirection>(SortDirection.NONE);

  let entries = $state<[string, any][]>([]);

  $effect(() => {
    entries = Object.entries(data);
  });

  function getSortIndicator(columnId: string): string {
    if (sortColumn !== columnId) return "";
    return sortDirection === SortDirection.ASC ? "↑" : "↓";
  }

  function toggleSort(column: TableColumn): void {
    if (sortColumn === column.header) {
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
      sortColumn = column.header;
      sortDirection = SortDirection.ASC;
    }

    currentPage = 1;
  }

  const filteredEntries = $derived.by(() => {
    let query = searchQuery.trim();
    if (!query) return entries;

    query = query
      .normalize("NFKD")
      .replace(/[\u0300-\u036f]/g, "")
      .toLowerCase();
    const result = [];

    for (let i = 0, len = entries.length; i < len; i++) {
      const entry = entries[i];
      const [id, row] = entry;

      let matches = false;
      if (id.includes(query)) {
        matches = true;
      } else {
        for (const field of searchFields) {
          const value = row[field];
          if (value.includes(query)) {
            matches = true;
            break;
          }
        }
      }

      if (matches) {
        result.push(entry);
      }
    }

    return result;
  });

  // Reset to first page when search changes
  $effect(() => {
    if (searchQuery) {
      currentPage = 1;
    }
  });

  const sortedEntries = $derived.by(() => {
    if (sortColumn === null || sortDirection === SortDirection.NONE) {
      return filteredEntries;
    }

    const column = columns.find(
      (col: TableColumn) => col.header === sortColumn
    );
    if (!column) return filteredEntries;

    return [...filteredEntries].sort((a, b) => {
      const [idA, rowA] = a;
      const [idB, rowB] = b;

      // Special case for keyField column (usually "id")
      if (column.field === keyField) {
        // sort by number
        return sortDirection === SortDirection.ASC
          ? // @ts-ignore we don't need to convert to number javascript does the math anyways with less overhead
            idA - idB
          : // @ts-ignore we don't need to convert to number javascript does the math anyways with less overhead
            idB - idA;
      }

      const valueA = rowA[column.field];
      const valueB = rowB[column.field];

      if (typeof valueA === "number" && typeof valueB === "number") {
        return sortDirection === SortDirection.ASC
          ? valueA - valueB
          : valueB - valueA;
      }

      if (column.dateValueField) {
        const dateA = rowA[column.dateValueField];
        const dateB = rowB[column.dateValueField];
        return sortDirection === SortDirection.ASC
          ? dateA.getTime() - dateB.getTime()
          : dateB.getTime() - dateA.getTime();
      }

      return sortDirection === SortDirection.ASC
        ? valueA.localeCompare(valueB, "pt-PT")
        : valueB.localeCompare(valueA, "pt-PT");
    });
  });

  const totalItems = $derived(sortedEntries.length);
  const totalPages = $derived(Math.max(1, Math.ceil(totalItems / perPage)));

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

  const displayedEntries = $derived.by(() => {
    const startIndex = (currentPage - 1) * perPage;
    const endIndex = Math.min(startIndex + perPage, totalItems);
    return sortedEntries.slice(startIndex, endIndex);
  });

  function goToPage(page: number) {
    if (page >= 1 && page <= totalPages) {
      currentPage = page;
    }
  }
</script>

<div
  class="overflow-x-auto rounded-box border border-base-content/5 bg-base-200"
>
  <!-- Search bar -->
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
        placeholder="Pesquisar..."
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
        {#each columns as column}
          <th
            class="{column.responsive || ''} {'cursor-pointer select-none'}"
            onclick={() => toggleSort(column)}
          >
            {column.header}
            {getSortIndicator(column.header)}
          </th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#if loading}
        {#each { length: 5 } as _}
          <tr>
            {#each columns as column}
              <td class={column.responsive || ""}>
                <div class="skeleton h-4 w-16"></div>
              </td>
            {/each}
          </tr>
        {/each}
      {:else if totalItems === 0}
        <tr>
          <td
            colspan={columns.length}
            class="text-center py-8 text-base-content/70"
          >
            {searchQuery ? searchEmptyMessage : emptyMessage}
          </td>
        </tr>
      {:else}
        {#each displayedEntries as [id, row]}
          <tr class={rowClassName} onclick={() => onRowClick(id, row)}>
            <td class={columns[0].responsive || ""}>
              {id}
            </td>
            {#each columns.slice(1) as column}
              <td class={column.responsive || ""}>
                {row[column.field]}
              </td>
            {/each}
          </tr>
        {/each}
      {/if}
    </tbody>
  </table>

  <!-- Pagination -->
  {#if !loading}
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
            onchange={(e) =>
              (perPage = parseInt(e.currentTarget.value, 10) || 10)}
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

      <div class="join max-lg:w-full justify-center">
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
