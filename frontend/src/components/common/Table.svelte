<script lang="ts">
    import type { TableColumn } from "@lib/types/table";

    // Props
    let {
        data, // Expects Record<string, any>
        columns,
        keyField, // Field name containing the unique ID (e.g., 'id')
        loading = false,
        emptyMessage,
        searchEmptyMessage = "Nenhum resultado encontrado",
        rowClassName = "hover:bg-base-300 cursor-pointer",
        onRowClick,
        searchQuery = $bindable(""), // Bindable search query
        totalItems = 0,
        currentPage = $bindable(1),
        perPage = $bindable(10),
    }: {
        data: Record<string, any>;
        columns: TableColumn[];
        keyField: string;
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

    // Derived Data
    // Convert the input data (Record<string, any>) into an array of [id, rowObject] pairs
    const displayedEntries = $derived.by(() => {
        console.log("Table Data Prop:", data); // Log input data
        const entries = Object.entries(data || {}); // Ensure data is not null/undefined
        console.log("Displayed Entries (derived):", entries); // Log the derived array
        return entries;
    });

    // --- Sorting ---
    // Apply sorting to the displayedEntries array
    const sortedEntries = $derived.by(() => {
        // Log the value *before* sorting
        console.log("Entries before sorting:", displayedEntries);

        if (sortColumn === null || sortDirection === SortDirection.NONE) {
            return displayedEntries; // Return the unsorted array
        }
        const column = columns.find((col) => col.header === sortColumn);
        if (!column) return displayedEntries;

        const fieldPath = column.field.split(".");

        // Create a new sorted array
        const sorted = [...displayedEntries].sort(([, rowA], [, rowB]) => {
            const getValue = (row: any, path: string[]) =>
                path.reduce((obj, key) => obj?.[key], row);

            const valueA = getValue(rowA, fieldPath);
            const valueB = getValue(rowB, fieldPath);

            // Use keyField prop for ID sorting
            if (column.field === keyField) {
                // Access the ID using the keyField prop
                const idA = parseInt(rowA[keyField], 10);
                const idB = parseInt(rowB[keyField], 10);
                // Handle potential NaN if keyField value isn't a number string
                if (!isNaN(idA) && !isNaN(idB)) {
                    return sortDirection === SortDirection.ASC
                        ? idA - idB
                        : idB - idA;
                }
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
        console.log("Sorted Entries (derived):", sorted); // Log the sorted array
        return sorted;
    });

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

    // Helper to get cell value
    function getCellValue(row: any, fieldPath: string): any {
        if (typeof row !== "object" || row === null) return "";
        try {
            // Special case for 'id' field - use the keyField prop
            if (fieldPath === keyField) {
                return row[keyField] ?? "";
            }
            // Otherwise, use the reduce method
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
    class="overflow-x-auto rounded-box border border-base-content/10 bg-base-100 shadow"
>
    <!--  Search bar removed - handled externally -->

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
                    <!-- Iterate over the derived sortedEntries array -->
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
        </table>
    </div>

    {#if !loading && totalItems > 0 && totalPages > 1}
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
