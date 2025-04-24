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
        { header: "Caminho", field: "path" },
        { header: "Pai", field: "parent_path" },
        { header: "Descrição", field: "description" },
        { header: "Ícone", field: "icon" },
        // Add Edit/Delete buttons later
    ];

    onMount(async () => {
        try {
            const pagesArray = await getCustomPages();
            const pagesRecord: Record<string, CustomPage> = {};
            pagesArray.forEach((page) => {
                pagesRecord[page.id.toString()] = page;
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
        // Navigate to edit page (implement later)
        if (typeof window !== "undefined") {
            window.location.href = `/admin/pages/edit/${id}/`; // Define this route later
        }
        console.log("Edit page:", id, row);
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
