<script lang="ts">
    import { onMount } from "svelte";
    import Table from "@components/common/Table.svelte";
    import type { CustomPage } from "@lib/types/custom-page";
    import type { TableColumn } from "@lib/types/table";
    import { getCustomPages } from "@api/custom-pages-api";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    let pages = $state<Record<string, CustomPage>>({});
    let isLoading = $state(true);
    let error = $state<string | null>(null);

    const columns: TableColumn[] = [
        { header: "ID", field: "id" },
        { header: "Nome", field: "name" },
        { header: "Tipo", field: "is_group" }, // Add column for type
        { header: "Caminho", field: "path" },
        { header: "Pai", field: "parent_path" },
        { header: "Descrição", field: "description" },
        { header: "Ícone", field: "icon" },
    ];

    onMount(async () => {
        try {
            const pagesArray = await getCustomPages();
            const pagesRecord: Record<string, CustomPage> = {};
            pagesArray.forEach((page) => {
                // Add the is_group property if it's missing from the type (should be included by backend now)
                pagesRecord[page.id.toString()] = {
                    ...page,
                    is_group: page.is_group ?? false,
                };
            });
            pages = pagesRecord;
        } catch (e: any) {
            error = `Erro ao carregar páginas: ${e.message}`;
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoading = false;
        }
    });

    function handleRowClick(id: string, row: CustomPage) {
        if (typeof window !== "undefined") {
            window.location.href = `/admin/pages/edit/${id}/`;
        }
        // console.log("Edit page:", id, row); // Keep for debugging if needed
    }
</script>

{#if error}
    <div class="alert alert-error">{error}</div>
{/if}

<Table
    data={pages}
    {columns}
    loading={isLoading}
    emptyMessage="Nenhuma página customizada encontrada."
    keyField="id"
    searchFields={["name", "path", "description"]}
    onRowClick={handleRowClick}
    rowClassName="hover:bg-base-300 cursor-pointer"
/>
