<script lang="ts">
    import type { TableColumn } from "@lib/types/table";
    import { toSearchString } from "@utils/search-utils"; // Import for searching

    // Props
    let {
        data, // Expects Record<string, any>
        columns,
        keyField, // Field name containing the unique ID (e.g., 'id')
        searchFields, // Fields to use for client-side search
        loading = false,
        emptyMessage,
        searchEmptyMessage = "Nenhum resultado encontrado",
        rowClassName = "hover:bg-base-300 cursor-pointer",
        onRowClick,
    }: {
        data: Record<string, any>;
        columns: TableColumn[];
        keyField: string;
        searchFields: string[];
        loading?: boolean;
        emptyMessage: string;
        searchEmptyMessage?: string;
        rowClassName?: string;
        onRowClick: (id: string, row: any) => void;
    } = $props();

    // Internal State for Search, Sort, Pagination
    let searchQuery = $state("");
    let currentPage = $state(1);
    let perPage = $state(10);

    // Sorting State
    const enum SortDirection {
        NONE,
        ASC,
        DESC,
    }
    let sortColumn = $state<string | null>(null);
    let sortDirection = $state<SortDirection>(SortDirection.NONE);

    // --- Client-side Filtering ---
    const filteredEntries = $derived.by(() => {
        const entries = Object.entries(data || {});
        const query = searchQuery.trim(); // Use internal state
        if (!query) {
            return entries; // No filter applied
        }
        const lowerCaseQuery = toSearchString(query);

        return entries.filter(([id, row]) => {
            // Check ID first
            if (id.toLowerCase().includes(lowerCaseQuery)) {
                return true;
            }
            // Check against specified search fields
            for (const fieldPath of searchFields) {
                const value = getCellValue(row, fieldPath); // Use existing helper
                if (
                    value &&
                    toSearchString(value.toString()).includes(lowerCaseQuery)
                ) {
                    return true;
                }
            }
            return false;
        });
    });

    // Reset to first page when search changes
    $effect(() => {
        if (searchQuery !== undefined) {
            currentPage = 1;
        }
    });

    // --- Sorting (Applied to filtered data) ---
    const sortedEntries = $derived.by(() => {
        const entriesToUse = filteredEntries;

        if (sortColumn === null || sortDirection === SortDirection.NONE) {
            return entriesToUse;
        }
        const column = columns.find((col) => col.header === sortColumn);
        if (!column) return entriesToUse;

        const fieldPath = column.field.split(".");

        const sorted = [...entriesToUse].sort(([, rowA], [, rowB]) => {
            const getValue = (row: any, path: string[]) =>
                path.reduce((obj, key) => obj?.[key], row);

            const valueA = getValue(rowA, fieldPath);
            const valueB = getValue(rowB, fieldPath);

            if (column.field === keyField) {
                const numA = parseFloat(rowA[keyField]);
                const numB = parseFloat(rowB[keyField]);
                if (!isNaN(numA) && !isNaN(numB)) {
                    return sortDirection === SortDirection.ASC
                        ? numA - numB
                        : numB - numA;
                }
                const strA = rowA[keyField]?.toString() ?? "";
                const strB = rowB[keyField]?.toString() ?? "";
                return sortDirection === SortDirection.ASC
                    ? strA.localeCompare(strB, "pt-PT")
                    : strB.localeCompare(strA, "pt-PT");
            }

            if (column.dateValueField) {
                const datePath = column.dateValueField.split(".");
                const dateA = getValue(rowA, datePath);
                const dateB = getValue(rowB, datePath);
                if (dateA instanceof Date && dateB instanceof Date) {
                    return sortDirection === SortDirection.ASC
                        ? dateA.getTime() - dateB.getTime()
                        : dateB.getTime() - dateA.getTime();
                }
            }

            if (typeof valueA === "number" && typeof valueB === "number") {
                return sortDirection === SortDirection.ASC
                    ? valueA - valueB
                    : valueB - valueA;
            }

            const strA = valueA?.toString() ?? "";
            const strB = valueB?.toString() ?? "";
            return sortDirection === SortDirection.ASC
                ? strA.localeCompare(strB, "pt-PT")
                : strB.localeCompare(strA, "pt-PT");
        });
        return sorted;
    });

    // --- Pagination (Applied to filtered and sorted data) ---
    const totalItems = $derived(sortedEntries.length);
    const totalPages = $derived(Math.max(1, Math.ceil(totalItems / perPage)));

    $effect(() => {
        // Ensure currentPage is valid after filtering/sorting changes totalPages
        if (currentPage > totalPages) {
            currentPage = totalPages;
        }
    });

    const paginatedEntries = $derived.by(() => {
        const start = (currentPage - 1) * perPage;
        const end = start + perPage;
        return sortedEntries.slice(start, end);
    });

    // --- Other Functions ---
    function getSortIndicator(columnHeader: string): string {
        if (sortColumn !== columnHeader) return "";
        return sortDirection === SortDirection.ASC ? "↑" : "↓";
    }

    function toggleSort(column: TableColumn): void {
        if (sortColumn === column.header) {
            sortDirection = (sortDirection + 1) % 3;
            if (sortDirection === SortDirection.NONE) {
                sortColumn = null;
            }
        } else {
            sortColumn = column.header;
            sortDirection = SortDirection.ASC;
        }
        currentPage = 1;
    }

    function generatePageNumbers(
        current: number,
        total: number,
    ): (number | null)[] {
        if (total <= 7) {
            // Show all pages if 7 or less
            return Array.from({ length: total }, (_, i) => i + 1);
        }
        // Logic for more than 7 pages (ellipsis)
        if (current < 4) {
            return [1, 2, 3, 4, 5, null, total];
        } else if (current > total - 3) {
            return [1, null, total - 4, total - 3, total - 2, total - 1, total];
        } else {
            return [1, null, current - 1, current, current + 1, null, total];
        }
    }

    function goToPage(page: number) {
        if (page >= 1 && page <= totalPages && page !== currentPage) {
            currentPage = page;
        }
    }

    function handlePerPageChange(e: Event) {
        const newPerPage =
            parseInt((e.target as HTMLInputElement).value, 10) || 10;
        if (newPerPage !== perPage) {
            perPage = newPerPage;
            currentPage = 1;
        }
    }

    function getCellValue(row: any, fieldPath: string): any {
        if (typeof row !== "object" || row === null) return "";
        try {
            if (fieldPath === keyField) {
                return row[keyField] ?? "";
            }
            return (
                fieldPath.split(".").reduce((obj, key) => obj?.[key], row) ?? ""
            );
        } catch (e) {
            console.error(
                `Error getting cell value for path "${fieldPath}" in row:`,
                row,
                e,
            );
            return "Error";
        }
    }
</script>

<div
    class="overflow-x-auto rounded-box border border-base-content/10 bg-base-200 shadow"
>
    <!-- Search bar -->
    <div class="p-2 flex justify-center border-b border-base-content/10">
        <div class="join w-full max-w-md mx-auto">
            <div
                class="join-item flex items-center px-3 bg-base-200 rounded-l-lg"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 16 16"
                    fill="currentColor"
                    class="w-4 h-4 opacity-70"
                    ><path
                        fill-rule="evenodd"
                        d="M9.965 11.026a5 5 0 1 1 1.06-1.06l2.755 2.754a.75.75 0 1 1-1.06 1.06l-2.755-2.754ZM10.5 7a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0Z"
                        clip-rule="evenodd"
                    /></svg
                >
            </div>
            <input
                type="text"
                placeholder="Pesquisar..."
                class="input input-bordered join-item w-full focus:outline-none focus:border-primary"
                bind:value={searchQuery}
                disabled={loading}
            />
            {#if searchQuery}
                <button
                    class="btn join-item bg-base-200 rounded-r-lg"
                    onclick={() => {
                        searchQuery = "";
                    }}
                    title="Limpar pesquisa"
                    disabled={loading}
                >
                    ×
                </button>
            {/if}
        </div>
    </div>

    <div class="overflow-x-auto">
        <table class="table table-pin-rows table-sm md:table-md">
            <thead>
                <tr>
                    {#each columns as column (column.header)}
                        <th
                            class="{column.responsive ||
                                ''} cursor-pointer select-none hover:bg-base-200"
                            onclick={() => toggleSort(column)}
                        >
                            {column.header}
                            <span class="ml-1"
                                >{getSortIndicator(column.header)}</span
                            >
                        </th>
                    {/each}
                </tr>
            </thead>
            <tbody>
                {#if loading}
                    {#each { length: Math.min(perPage, 5) } as _}
                        <tr>
                            {#each columns as column}
                                <td class={column.responsive || ""}>
                                    <div class="skeleton h-4 w-16 my-1"></div>
                                </td>
                            {/each}
                        </tr>
                    {/each}
                {:else if filteredEntries.length === 0}
                    <tr>
                        <td
                            colspan={columns.length}
                            class="text-center py-8 text-base-content/70"
                        >
                            {searchQuery ? searchEmptyMessage : emptyMessage}
                        </td>
                    </tr>
                {:else}
                    {#each paginatedEntries as [id, row] (id)}
                        <tr
                            class={rowClassName}
                            onclick={() => onRowClick(id, row)}
                        >
                            {#each columns as column (column.field)}
                                <td class={column.responsive || ""}>
                                    {getCellValue(row, column.field)}
                                </td>
                            {/each}
                        </tr>
                    {/each}
                {/if}
            </tbody>
            <!-- tfoot removed -->
        </table>
    </div>

    <!-- Pagination - Always show if not loading and there are items -->
    {#if !loading && totalItems > 0}
        <div
            class="flex flex-col md:flex-row justify-between items-center gap-2 p-2 bg-base-200 border-t border-base-content/10"
        >
            <div class="flex items-center gap-2 text-sm">
                <span>Mostrar</span>
                <select
                    class="select select-bordered select-xs"
                    bind:value={perPage}
                    onchange={handlePerPageChange}
                >
                    <option value={10}>10</option>
                    <option value={25}>25</option>
                    <option value={50}>50</option>
                    <option value={100}>100</option>
                </select>
                <span>por página</span>
            </div>

            <span class="text-sm text-center md:text-right">
                A mostrar {filteredEntries.length > 0
                    ? (currentPage - 1) * perPage + 1
                    : 0}
                a {Math.min(currentPage * perPage, totalItems)} de {totalItems} resultados
            </span>

            <div class="join mt-2 md:mt-0">
                <button
                    class="join-item btn btn-sm"
                    disabled={currentPage === 1}
                    onclick={() => goToPage(1)}>«</button
                >
                <button
                    class="join-item btn btn-sm"
                    disabled={currentPage === 1}
                    onclick={() => goToPage(currentPage - 1)}>‹</button
                >

                {#each generatePageNumbers(currentPage, totalPages) as page}
                    {#if page === null}
                        <button class="join-item btn btn-sm btn-disabled"
                            >...</button
                        >
                    {:else}
                        <button
                            class="join-item btn btn-sm {page === currentPage
                                ? 'btn-active btn-primary'
                                : ''}"
                            onclick={() => goToPage(page)}>{page}</button
                        >
                    {/if}
                {/each}

                <button
                    class="join-item btn btn-sm"
                    disabled={currentPage === totalPages}
                    onclick={() => goToPage(currentPage + 1)}>›</button
                >
                <button
                    class="join-item btn btn-sm"
                    disabled={currentPage === totalPages}
                    onclick={() => goToPage(totalPages)}>»</button
                >
            </div>
        </div>
    {/if}
</div>
