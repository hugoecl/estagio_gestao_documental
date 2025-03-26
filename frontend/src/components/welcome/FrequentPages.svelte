<script lang="ts">
  import { getIcon, getName } from "@utils/frequent-page-utils";
  import { onMount } from "svelte";

  let frequentPages = $state<[string, number][]>([]);
  let isLoading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let menuPageNames = $state<Record<string, string>>({});

  const { menuPageNamesProps }: { menuPageNamesProps: Record<string, string> } =
    $props();

  onMount(async () => {
    try {
      menuPageNames = menuPageNamesProps;

      const { getAnalytics } = await import("@api/analytics-api");
      const data = await getAnalytics();

      frequentPages = data
        .sort((a, b) => b[1] - a[1])
        .slice(0, 7)
        .filter(([path]) => path !== "/");
    } catch (err) {
      console.error(err);
      error = "Não foi possível carregar as páginas frequentes";
    } finally {
      isLoading = false;
    }
  });
</script>

<div class="card bg-base-100 shadow-xl">
  <div class="card-body">
    <h2 class="card-title text-primary">
      <i class="fa-solid fa-star mr-2"></i>
      Páginas Frequentes
    </h2>

    {#if isLoading}
      <div class="flex justify-center p-4">
        <span class="loading loading-spinner loading-md"></span>
      </div>
    {:else if error}
      <div class="alert alert-error">{error}</div>
    {:else if frequentPages.length === 0}
      <p class="text-center text-gray-500">
        Nenhuma página visitada recentemente
      </p>
    {:else}
      <div class="grid grid-cols-2 gap-2 mt-2">
        {#each frequentPages as [pagePath, visitCount]}
          <a
            href={pagePath}
            class="btn btn-outline flex items-center justify-start gap-2 hover:bg-primary hover:text-primary-content"
            title="{visitCount} visitas"
          >
            <i class="fa-solid fa-{getIcon(pagePath)}"></i>
            <span class="truncate">{getName(menuPageNames, pagePath)}</span>
          </a>
        {/each}
      </div>
    {/if}
  </div>
</div>
