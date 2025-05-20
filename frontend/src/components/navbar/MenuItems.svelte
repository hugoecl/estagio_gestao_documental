<script>
  import { slide } from 'svelte/transition';
  
  // Define props for this component
  export let item;
</script>

<li>
  {#if item.children && item.children.length > 0}
    <details open>
      <summary>
        {#if item.icon}<i class="fa-solid fa-{item.icon}"></i>{/if}
        {item.title}
      </summary>
      <ul>
        {#each item.children as child (child.id)}
          <svelte:self item={child} />
        {/each}
      </ul>
    </details>
  {:else if item.path}
    <a href={item.path.endsWith("/") ? item.path : item.path + "/"}>
      {#if item.icon}<i class="fa-solid fa-{item.icon}"></i>{/if}
      {item.title}
    </a>
  {:else}
    <details open>
      <summary>

        {#if item.icon}<i class="fa-solid fa-{item.icon}"></i>{/if}
        {item.title} 
      </summary>
    </details>
  {/if}
</li> 