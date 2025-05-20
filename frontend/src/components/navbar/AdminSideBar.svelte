<script lang="ts">
  import { onMount } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { getNavigationMenu, reorderPages } from "@api/custom-pages-api";
  import { showAlert, AlertType, AlertPosition } from "@components/alert/alert";
  import type { NavigationItem } from "@lib/types/custom-page";
  import { toSearchString } from "@utils/search-utils";
  import jccBigLogo from "src/assets/jcc_big.svg?raw"; // Import logo correctly
  import MenuItems from "./MenuItems.svelte";
  import ReorderItem from "./ReorderItem.svelte";
  import StaticMenu from "@components/navbar/StaticMenu.astro?raw";

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

  // Handle DnD events for root level items
  function handleDndConsider(e) {
    const { items: newItems } = e.detail;
    menuItems = [...newItems];
  }
  
  function handleDndFinalize(e) {
    const { items: newItems } = e.detail;
    menuItems = [...newItems];
    
    // Update display_order for items
    updateDisplayOrders();
  }
  
  // Listen for updates from child components
  onMount(() => {
    const handleItemUpdated = (e) => {
      const { id, children } = e.detail;
      updateItemChildren(id, children);
    };
    
    document.addEventListener('itemUpdated', handleItemUpdated);
    
    return () => {
      document.removeEventListener('itemUpdated', handleItemUpdated);
    };
  });
  
  // Update a specific item's children
  function updateItemChildren(itemId, children) {
    menuItems = menuItems.map(item => {
      if (item.id === itemId) {
        return { ...item, children };
      }
      
      if (item.children?.length) {
        const updatedChildren = updateChildrenRecursively(item.children, itemId, children);
        return { ...item, children: updatedChildren };
      }
      
      return item;
    });
    
    // Update all display orders after a child change
    updateDisplayOrders();
  }
  
  // Recursively update children
  function updateChildrenRecursively(items, targetId, newChildren) {
    return items.map(item => {
      if (item.id === targetId) {
        return { ...item, children: newChildren };
      }
      
      if (item.children?.length) {
        const updatedChildren = updateChildrenRecursively(item.children, targetId, newChildren);
        return { ...item, children: updatedChildren };
      }
      
      return item;
    });
  }
  
  // Update display orders for all items
  function updateDisplayOrders() {
    // Update root items display_order
    for (let i = 0; i < menuItems.length; i++) {
      menuItems[i].display_order = i;
    }
    
    // Update child items display_order recursively
    menuItems = menuItems.map(item => {
      if (item.children?.length) {
        return {
          ...item,
          children: updateChildDisplayOrders(item.children)
        };
      }
      return item;
    });
  }
  
  function updateChildDisplayOrders(children, startIndex = 0) {
    return children.map((child, index) => {
      const updatedChild = { ...child, display_order: startIndex + index };
      
      if (child.children?.length) {
        updatedChild.children = updateChildDisplayOrders(child.children);
      }
      
      return updatedChild;
    });
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
</script>

<div class="sidebar bg-base-200">
  <a href="/" class="sidebar__header">
    {@html jccBigLogo}
    <h1 class="sidebar__title font-bold">Gestão Documental</h1>
  </a>

  <nav>
    {@html StaticMenu }
    <div class="pb-3 flex justify-center items-center">
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
        <div class="menu bg-base-100 border border-zinc-200 rounded-box w-56 p-2">
          <section 
            use:dndzone={{ 
              items: menuItems,
              type: 'root-items',
              flipDurationMs: 150,
              dropTargetStyle: {
                outline: '2px dashed #4CAF50'
              }
            }}
            onconsider={handleDndConsider}
            onfinalize={handleDndFinalize}
            class="space-y-1 min-h-[50px]"
          >
            {#each menuItems as item (item.id)}
              <ReorderItem {item} groupId="root-items" />
            {/each}
          </section>
          
          <div class="text-xs text-base-content/70 mt-2 p-2">
            <p>Dica: Arraste os itens para reorganizar</p>
          </div>
        </div>
      {:else}
        <ul class="menu bg-base-100 border border-zinc-200 rounded-box w-56">
          {#each menuItems as item (item.id)}
            <MenuItems {item} />
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

  nav {
    position: relative;
    z-index: 400;
  }
  
  :global(.dndzone-activating) {
    transition: transform 150ms ease;
  }
  
  :global(.dndzone-copy-active) {
    cursor: copy;
  }
  
  :global(.dndzone-keyboard-active) {
    cursor: grab;
  }
</style> 