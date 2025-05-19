<script lang="ts">
  import { onMount } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { getNavigationMenu, reorderPages } from "@api/custom-pages-api";
  import { showAlert, AlertType, AlertPosition } from "@components/alert/alert";
  import type { NavigationItem } from "@lib/types/custom-page";
  import { toSearchString } from "@utils/search-utils";
  import jccBigLogo from "src/assets/jcc_big.svg?raw"; // Import logo correctly

  let isLoading = $state(true);
  let hasError = $state(false);
  let errorMsg = $state("");
  let menuItems = $state<NavigationItem[]>([]);
  let isAdmin = $state(true); // Will be set from props or from parent component
  let isReordering = $state(false);
  let isSaving = $state(false);
  
  onMount(async () => {
    try {
      const data = await getNavigationMenu();
      menuItems = data;
      isLoading = false;
    } catch (e) {
      hasError = true;
      errorMsg = e instanceof Error ? e.message : String(e);
      isLoading = false;
    }
  });

  function handleDndConsiderRoot(e) {
    const { items } = e.detail;
    menuItems = items;
  }

  function handleDndFinalizeRoot(e) {
    const { items } = e.detail;
    menuItems = [...items];

    // Update display_order for each item at the root level
    for (let i = 0; i < menuItems.length; i++) {
      menuItems[i].display_order = i;
    }
  }

  function handleDndConsiderChildren(e, parentItem) {
    const { items } = e.detail;
    const index = menuItems.findIndex(item => item.id === parentItem.id);
    if (index !== -1) {
      const newMenuItems = [...menuItems];
      newMenuItems[index] = { ...newMenuItems[index], children: items };
      menuItems = newMenuItems;
    }
  }

  function handleDndFinalizeChildren(e, parentItem) {
    const { items } = e.detail;
    const index = menuItems.findIndex(item => item.id === parentItem.id);
    if (index !== -1) {
      const newMenuItems = [...menuItems];
      // Update children with their new display orders
      const updatedChildren = [...items];
      for (let i = 0; i < updatedChildren.length; i++) {
        updatedChildren[i].display_order = i;
      }
      newMenuItems[index] = { ...newMenuItems[index], children: updatedChildren };
      menuItems = newMenuItems;
    }
  }

  function toggleReordering() {
    isReordering = !isReordering;
  }

  async function saveOrderChanges() {
    isSaving = true;
    
    try {
      // Collect all items and their display orders
      const orders = [];
      
      // Process root items
      menuItems.forEach((item) => {
        orders.push({ id: item.id, display_order: item.display_order });
        
        // Process children recursively using helper function
        if (item.children?.length > 0) {
          collectChildrenOrders(item.children, orders);
        }
      });
      
      const success = await reorderPages(orders);
      
      if (success) {
        showAlert('Ordem do menu guardada com sucesso!', AlertType.SUCCESS, AlertPosition.TOP);
        isReordering = false;
      } else {
        showAlert('Falha ao guardar a ordem do menu', AlertType.ERROR, AlertPosition.TOP);
      }
    } catch (e) {
      console.error('Error saving menu order:', e);
      showAlert('Erro ao guardar a ordem do menu: ' + (e instanceof Error ? e.message : String(e)), 
                AlertType.ERROR, AlertPosition.TOP);
    } finally {
      isSaving = false;
    }
  }

  function collectChildrenOrders(children: NavigationItem[], orders: { id: number; display_order: number }[]) {
    children.forEach((child) => {
      orders.push({ id: child.id, display_order: child.display_order });
      
      if (child.children?.length > 0) {
        collectChildrenOrders(child.children, orders);
      }
    });
  }
  
  // Prevent navigation when clicking items in reordering mode
  function handleItemClick(e) {
    if (isReordering) {
      e.preventDefault();
      e.stopPropagation();
      return false;
    }
  }
</script>

<div class="sidebar bg-base-200">
  <a href="/" class="sidebar__header">
    {@html jccBigLogo}
    <h1 class="sidebar__title font-bold">Gestão Documental</h1>
  </a>
  
  <div class="py-2 flex justify-center items-center">
    {#if isAdmin}
      <div class="flex gap-2">
        {#if isReordering}
          <button 
            class="btn btn-xs btn-primary" 
            onclick={saveOrderChanges} 
            disabled={isSaving}
          >
            {#if isSaving}
              <span class="loading loading-spinner loading-xs"></span>
              A guardar...
            {:else}
              <i class="fa-solid fa-save"></i> Guardar
            {/if}
          </button>
          <button 
            class="btn btn-xs btn-outline" 
            onclick={toggleReordering}
            disabled={isSaving}
          >
            Cancelar
          </button>
        {:else}
          <button 
            class="btn btn-xs btn-outline" 
            onclick={toggleReordering}
          >
            <i class="fa-solid fa-sort"></i> Reordenar Menu
          </button>
        {/if}
      </div>
    {/if}
  </div>

  <nav>
    {#if isLoading}
      <div class="flex justify-center p-4">
        <span class="loading loading-spinner loading-md"></span>
      </div>
    {:else if hasError}
      <div class="p-4 text-error">
        <p>Falha ao carregar o menu de navegação</p>
        <p class="text-xs">{errorMsg}</p>
      </div>
    {:else if menuItems.length === 0}
      <div class="p-4 text-base-content/70">
        Não existem itens no menu disponíveis.
      </div>
    {:else}
      {#if isReordering}
        <div class="menu bg-base-100 border border-zinc-200 rounded-box w-56 p-2" >
          <div class="root-container">
            <div use:dndzone={{items: menuItems, flipDurationMs: 200, type: "root"}} 
                onconsider={handleDndConsiderRoot} 
                onfinalize={handleDndFinalizeRoot} 
                class="dnd-container space-y-1">
              {#each menuItems as item (item.id)}
                <div class="dnd-item border-dashed border border-base-content/30 p-1 bg-base" >
                  <div class="handle p-2 bg-base-200 flex items-center">
                    <i class="fa-solid fa-grip-vertical mr-2"></i>
                    <span>{item.title}</span>
                  </div>
                  
                  {#if item.children && item.children.length > 0}
                    <div class="ml-4 mt-2 child-container">
                      <div use:dndzone={{items: item.children, flipDurationMs: 200, type: `children-${item.id}`}} 
                           onconsider={(e) => handleDndConsiderChildren(e, item)} 
                           onfinalize={(e) => handleDndFinalizeChildren(e, item)}
                           class="dnd-container space-y-1">
                        {#each item.children as child (child.id)}
                          <div class="dnd-item border-dashed border border-base-content/30 p-1 bg-base" >
                            <div class="handle p-2 bg-base-200 flex items-center">
                              <i class="fa-solid fa-grip-vertical mr-2"></i>
                              <span>{child.title}</span>
                            </div>
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        </div>
      {:else}
        <ul class="menu bg-base-100 border border-zinc-200 rounded-box w-56">
          {#each menuItems as item (item.id)}
            <li>
              {#if item.children && item.children.length > 0}
                <details open>
                  <summary>
                    {#if item.icon}<i class="fa-solid fa-{item.icon}"></i>{/if}
                    {item.title}
                  </summary>
                  <ul>
                    {#each item.children as child (child.id)}
                      <li>
                        {#if child.children && child.children.length > 0}
                          <details>
                            <summary>
                              {#if child.icon}<i class="fa-solid fa-{child.icon}"></i>{/if}
                              {child.title}
                            </summary>
                            <!-- Handle deeper nesting -->
                          </details>
                        {:else if child.path}
                          <a href={child.path.endsWith("/") ? child.path : child.path + "/"}>
                            {#if child.icon}<i class="fa-solid fa-{child.icon}"></i>{/if}
                            {child.title}
                          </a>
                        {:else}
                          <span>
                            {#if child.icon}<i class="fa-solid fa-{child.icon}"></i>{/if}
                            {child.title} (Grupo sem link direto)
                          </span>
                        {/if}
                      </li>
                    {/each}
                  </ul>
                </details>
              {:else if item.path}
                <a href={item.path.endsWith("/") ? item.path : item.path + "/"}>
                  {#if item.icon}<i class="fa-solid fa-{item.icon}"></i>{/if}
                  {item.title}
                </a>
              {:else}
                <span>
                  {#if item.icon}<i class="fa-solid fa-{item.icon}"></i>{/if}
                  {item.title} (Grupo sem link direto)
                </span>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </nav>
</div>

<style>
  .sidebar {
    height: 100vh;
    padding: 16px;
    overflow-y: auto;
    z-index: 400;
    width: 100%;
    position: relative;
  }

  @media (min-width: 1024px) {
    .sidebar {
      width: 255px;
    }
  }


  .sidebar__header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
    flex-direction: column;
    text-decoration: none;
  }

  .sidebar__header :global(svg) {
    transform: scale(1.45);
    height: 100px;
  }

  .sidebar__title {
    font-size: 20px;
  }

  .dnd-item {
    z-index: 1000000 !important;
  }

  
</style> 