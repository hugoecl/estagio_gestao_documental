<script>
  import { dndzone } from 'svelte-dnd-action';
  
  // Props
  export let item;
  export let groupId = "root"; // Default group ID for the root level
  
  // States
  let isDragging = false;
  let childItems = item.children || [];

  // Handle drag in the child container
  function handleDndConsiderChildren(e) {
    const { items: newItems } = e.detail;
    childItems = [...newItems];
  }
  
  function handleDndFinalizeChildren(e) {
    const { items: newItems } = e.detail;
    childItems = [...newItems];
    
    // Update the parent item's children
    item.children = childItems;
    
    // Dispatch an event to notify parent of changes
    const updateEvent = new CustomEvent('itemUpdated', {
      bubbles: true,
      detail: { id: item.id, children: childItems }
    });
    document.dispatchEvent(updateEvent);
  }

  // Track whether this item is being dragged
  function setDraggingState(isDraggingNow) {
    isDragging = isDraggingNow;
  }

  $: childGroupId = `group-${item.id}`; // Create a unique group ID for children
</script>

<div class="dnd-item border-dashed border border-base-content/30 p-1 bg-base mb-2"
     class:is-dragging={isDragging}>
  <div class="handle p-2 bg-base-200 flex items-center cursor-grab">
    <i class="fa-solid fa-grip-vertical mr-2"></i>
    <span>{item.title}</span>
  </div>
  
  {#if childItems.length > 0}
    <div class="ml-4 mt-2 child-container"
         use:dndzone={{
           items: childItems, 
           flipDurationMs: 150,
           dropTargetStyle: {},
           dragDisabled: false,
           type: childGroupId, // Each nested list gets its own group ID
           dropFromOthersDisabled: true // Don't allow dropping from other groups
         }}
         onconsider={handleDndConsiderChildren}
         onfinalize={handleDndFinalizeChildren}>
      {#each childItems as child (child.id)}
        <svelte:self 
          item={child} 
          groupId={childGroupId} 
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .dnd-item {
    user-select: none;
    position: relative;
    transition: background-color 0.2s, transform 0.1s, box-shadow 0.2s;
    z-index: 100000 !important;
  }
  
  .handle {
    border-radius: 4px;
    user-select: none;
  }
  
  .child-container {
    position: relative;
  }
  
  .is-dragging {
    opacity: 0.5 !important;
    outline: 2px solid #4CAF50 !important;
    transform: scale(1.02) !important;
  }
  
  /* Make entire item appear draggable */
  .dnd-item:hover {
    cursor: move;
    background-color: rgba(76, 175, 80, 0.05);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }
</style> 