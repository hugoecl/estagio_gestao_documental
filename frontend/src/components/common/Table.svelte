<script lang="ts">
    import type { TableColumn } from "@lib/types/table";
    // Removed toSearchString import as filtering is now external

    // Props
    let {
        data, // Expects Record<string, any> where key is ID
        columns,
        keyField, // Still needed to identify the ID column
        // searchFields, // No longer needed for client-side filtering
        loading = false,
        emptyMessage,
        searchEmptyMessage = "Nenhum resultado encontrado",
        rowClassName = "hover:bg-base-300 cursor-pointer", // Added cursor-pointer
        onRowClick,
        searchQuery = $bindable(""), // Bindable search query for parent control
        totalItems = 0, // Total items from server for pagination
        currentPage = $bindable(1),
        perPage = $bindable(10),
    }: {
        data: Record<string, any>;
        columns: TableColumn[];
        keyField: string;
        // searchFields: string[]; // Removed
        loading?: boolean;
        emptyMessage: string;
        searchEmptyMessage?: string;
        rowClassName?: string;
        onRowClick: (id: string, row: any) => void;
        searchQuery?: string;
        totalItems?: number;
        currentPage?: number;
        perPage?: number;
    } = $props();

    // Sorting State
    const enum SortDirection {
        NONE,
        ASC,
        DESC,
    }
    let sortColumn = $state<string | null>(null);
    let sortDirection = $state<SortDirection>(SortDirection.NONE);

    // Derived Data (No filtering/sorting here anymore)
    const displayedEntries = $derived(Object.entries(data)); // Just convert data to array

    // --- Sorting ---
    // Sorting logic remains client-side for now, applied *after* server fetches data for the current page
    // This is simpler but less accurate than server-side sorting.
    // For full server-side sorting, you'd need to pass sortColumn/sortDirection to the parent/API call.
    const sortedEntries = $derived(() => {
        if (sortColumn === null || sortDirection === SortDirection.NONE) {
            return displayedEntries;
        }
        const column = columns.find((col) => col.header === sortColumn);
        if (!column) return displayedEntries;

        // Find the actual field name, handling potential nesting like 'processedData.name'
        const fieldPath = column.field.split(".");

        return [...displayedEntries].sort(([, rowA], [, rowB]) => {
            // Helper to get potentially nested value
            const getValue = (row: any, path: string[]) =>
                path.reduce((obj, key) => obj?.[key], row);

            const valueA = getValue(rowA, fieldPath);
            const valueB = getValue(rowB, fieldPath);

            // Handle ID column specifically if needed (often numeric)
            if (column.field === keyField) {
                const idA = parseInt(rowA.id, 10); // Assuming ID is in row.id
                const idB = parseInt(rowB.id, 10);
                return sortDirection === SortDirection.ASC
                    ? idA - idB
                    : idB - idA;
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

            // LocaleCompare for strings, handle null/undefined
            const strA = valueA?.toString() ?? "";
            const strB = valueB?.toString() ?? "";
            return sortDirection === SortDirection.ASC
                ? strA.localeCompare(strB, "pt-PT")
                : strB.localeCompare(strA, "pt-PT");
        });
    });

    function getSortIndicator(columnId: string): string {
        if (sortColumn !== columnId) return "";
        return sortDirection === SortDirection.ASC ? "↑" : "↓";
    }

    function toggleSort(column: TableColumn): void {
        if (sortColumn === column.header) {
            sortDirection = (sortDirection + 1) % 3; // Cycle 0, 1, 2
            if (sortDirection === SortDirection.NONE) {
                sortColumn = null;
            }
        } else {
            sortColumn = column.header;
            sortDirection = SortDirection.ASC;
        }
        // NOTE: If implementing server-side sorting, emit an event here instead of sorting locally.
        // Example: dispatch('sortchange', { column: sortColumn, direction: sortDirection });
    }

    // --- Pagination ---
    const totalPages = $derived(Math.max(1, Math.ceil(totalItems / perPage)));

    function generatePageNumbers(
        current: number,
        total: number,
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

    function goToPage(page: number) {
        if (page >= 1 && page <= totalPages && page !== currentPage) {
            currentPage = page; // Update bound prop, parent should refetch data
        }
    }

    function handlePerPageChange(e: Event) {
        const newPerPage =
            parseInt((e.target as HTMLInputElement).value, 10) || 10;
        if (newPerPage !== perPage) {
            perPage = newPerPage;
            currentPage = 1; // Go to first page when changing items per page
        }
    }

    // Helper to get cell value, handling nested paths
    function getCellValue(row: any, fieldPath: string): any {
        return fieldPath.split(".").reduce((obj, key) => obj?.[key], row) ?? "";
    }
</script>

<div
    class="overflow-x-auto rounded-box border border-base-content/10 bg-base-100 shadow"
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
            />
            {#if searchQuery}
                <button
                    class="btn join-item bg-base-200 rounded-r-lg"
                    onclick={() => (searchQuery = "")}
                    title="Limpar pesquisa"
                >
                    ×
                </button>
            {/if}
        </div>
    </div>

    <div class="overflow-x-auto">
        <table class="table table-pin-rows table-sm md:table-md">
            <!-- -- Adjust size -->
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
                {:else if displayedEntries.length === 0}
                    <tr>
                        <td
                            colspan={columns.length}
                            class="text-center py-8 text-base-content/70"
                        >
                            {searchQuery ? searchEmptyMessage : emptyMessage}
                        </td>
                    </tr>
                {:else}
                    {#each sortedEntries as [id, row] (id)}
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
            {#if !loading && displayedEntries.length > 0}
                <tfoot>
                    <tr>
                        {#each columns as column (column.header)}
                            <th class={column.responsive || ""}
                                >{column.header}</th
                            >
                        {/each}
                    </tr>
                </tfoot>
            {/if}
        </table>
    </div>

    <!-- Pagination -->
    {#if !loading && totalItems > 0}
        <div
            class="flex flex-col md:flex-row justify-between items-center gap-2 p-2 bg-base-200 border-t border-base-content/10"
        >
            <div class="flex items-center gap-2 text-sm">
                <span>Mostrar</span>
                <select
                    class="select select-bordered select-xs"
                    value={perPage}
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
                A mostrar {displayedEntries.length > 0
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
