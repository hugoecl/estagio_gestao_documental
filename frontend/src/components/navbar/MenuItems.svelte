<script>
  import { slide } from 'svelte/transition';
  import API_BASE_URL from "@api/base-url";
  
  // Define props for this component
  export let item;
</script>

<li>
  {#if item.children && item.children.length > 0}
    <details open>
      <summary>
        {#if item.icon_type === 'image' && item.icon_image_path}
          <img 
            src={`${API_BASE_URL}/${item.icon_image_path}`} 
            alt={item.title} 
            class="w-5 h-5 object-contain inline-block mr-2" 
          />
        {:else if item.icon && (!item.icon_type || item.icon_type === 'fontawesome')}
          <i class="fa-solid fa-{item.icon}"></i>
        {/if}
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
      {#if item.icon_type === 'image' && item.icon_image_path}
        <img 
          src={`${API_BASE_URL}/${item.icon_image_path}`} 
          alt={item.title} 
          class="w-5 h-5 object-contain inline-block mr-2" 
        />
      {:else if item.icon && (!item.icon_type || item.icon_type === 'fontawesome')}
        <i class="fa-solid fa-{item.icon}"></i>
      {/if}
      {item.title}
    </a>
  {:else}
    <details open>
      <summary>
        {#if item.icon_type === 'image' && item.icon_image_path}
          <img 
            src={`${API_BASE_URL}/${item.icon_image_path}`} 
            alt={item.title} 
            class="w-5 h-5 object-contain inline-block mr-2" 
          />
        {:else if item.icon && (!item.icon_type || item.icon_type === 'fontawesome')}
          <i class="fa-solid fa-{item.icon}"></i>
        {/if}
        {item.title} 
      </summary>
    </details>
  {/if}
</li> 