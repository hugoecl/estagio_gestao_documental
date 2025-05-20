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
    import API_BASE_URL from "@api/base-url";

    // Define an extended type for displayed pages
    type DisplayedPage = CustomPage & {
        typeIcon: string;
        type: string;
    };

    let allPages = $state<CustomPage[]>([]);
    let displayedPages = $state<Record<string, DisplayedPage>>({});
    let isLoading = $state(true);
    let error = $state<string | null>(null);
    let currentPath = $state<string | null>(null);
    let navigationHistory = $state<{name: string, path: string | null}[]>([]);

    const columns: TableColumn[] = [
        { header: "ID", field: "id" },
        { 
            header: "Nome", 
            field: "name",
            cellRenderer: (value, row) => {
                const icon = row.is_group ? 'fa-folder' : 'fa-file-alt';
                const iconColor = row.is_group ? 'text-accent' : 'text-blue-500';
                
                // Always use the folder/file icon for the name column
                return `<div class="flex items-center">
                    <i class="fa-solid ${icon} ${iconColor} mr-2"></i>
                    <span>${value}</span>
                </div>`;
            }
        },
        { 
            header: "Tipo", 
            field: "type",
            cellRenderer: (value, row) => {
                const badgeColor = row.is_group ? 'badge-accent' : 'badge-info';
                return `<div class="badge ${badgeColor}">${value}</div>`;
            }
        }, 
        { header: "Caminho", field: "path" },
        { header: "Descrição", field: "description" },
        { 
            header: "Ícone", 
            field: "icon",
            cellRenderer: (value, row) => {
                if (row.icon_type === 'image' && row.icon_image_path) {
                    return `<div class="flex items-center">
                        <img src="${API_BASE_URL}/${row.icon_image_path}" alt="Ícone" class="w-5 h-5 object-contain" />
                        <span class="ml-2">Imagem</span>
                    </div>`;
                } else if (value && row.icon_type === 'fontawesome') {
                    return `<i class="fa-solid fa-${value}"></i> ${value}`;
                }
                return '';
            }
        },
    ];

    onMount(async () => {
        try {
            allPages = await getCustomPages();
            filterPagesByParent(null); // Show root level initially
        } catch (e: any) {
            error = `Erro ao carregar páginas: ${e.message}`;
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoading = false;
        }
    });

    function filterPagesByParent(parentPath: string | null) {
        const filtered = allPages.filter(page => page.parent_path === parentPath);
        const pagesRecord: Record<string, DisplayedPage> = {};
        filtered.forEach(page => {
            pagesRecord[page.id.toString()] = {
                ...page,
                type: page.is_group ? 'Grupo' : 'Página',
                typeIcon: page.is_group ? 'fa-folder' : 'fa-file-alt'
            };
        });
        displayedPages = pagesRecord;
        currentPath = parentPath;
    }

    function navigateTo(path: string | null) {
        if (path === null) {
            // Reset to root
            navigationHistory = [];
        } else {
            // Find the index of the path in history
            const index = navigationHistory.findIndex(item => item.path === path);
            if (index >= 0) {
                // Keep only history up to that index
                navigationHistory = navigationHistory.slice(0, index + 1);
            } else {
                // If it's a new path not in history, add it
                const currentPage = allPages.find(p => p.path === path);
                if (currentPage) {
                    navigationHistory = [...navigationHistory, {
                        name: currentPage.name,
                        path: currentPage.path
                    }];
                }
            }
        }
        filterPagesByParent(path);
    }

    function handleRowClick(id: string, row: DisplayedPage) {
        if (row.is_group) {
            // If clicking a group, navigate to its contents
            navigateTo(row.path);
        } else {
            // If clicking a page, go to edit page
        if (typeof window !== "undefined") {
            window.location.href = `/admin/pages/edit/${id}/`;
            }
        }
    }
</script>

<div class="mb-4">
    <!-- Breadcrumb navigation -->
    <div class="text-sm breadcrumbs mb-4">
        <ul>
            <li>
                <a href="#" on:click|preventDefault={() => navigateTo(null)} class="text-primary">
                    <i class="fa-solid fa-home mr-1"></i> Raiz
                </a>
            </li>
            {#each navigationHistory as item}
                <li>
                    <a href="#" on:click|preventDefault={() => navigateTo(item.path)} class="text-primary">
                        <i class="fa-solid fa-folder mr-1"></i> {item.name}
                    </a>
                </li>
            {/each}
        </ul>
    </div>
    
    {#if currentPath !== null}
        <div class="flex items-center mb-4">
            <button class="btn btn-sm btn-outline" on:click={() => navigateTo(navigationHistory.length > 1 ? navigationHistory[navigationHistory.length - 2].path : null)}>
                <i class="fa-solid fa-arrow-left mr-1"></i> Voltar
            </button>
            <div class="ml-4 font-semibold">
                {#if navigationHistory.length > 0}
                    Conteúdo de: {navigationHistory[navigationHistory.length - 1].name}
                {:else}
                    Nível Raiz
                {/if}
            </div>
        </div>
    {/if}
</div>

{#if error}
    <div class="alert alert-error">{error}</div>
{/if}

<Table
    data={displayedPages}
    {columns}
    loading={isLoading}
    emptyMessage={currentPath === null 
        ? "Nenhuma página ou grupo no nível raiz." 
        : `Nenhuma página ou grupo encontrado em "${navigationHistory[navigationHistory.length - 1]?.name || currentPath}".`}
    keyField="id"
    searchFields={["name", "path", "description"]}
    onRowClick={handleRowClick}
    rowClassName={(row) => `hover:bg-base-300 cursor-pointer ${row.is_group ? 'bg-base-200' : ''}`}
/>
