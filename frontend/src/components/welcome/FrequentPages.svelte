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
      const data = await getAnalytics(); // getAnalytics now returns [] on auth error

      // No need to check response.ok or response.status here if getAnalytics handles it
      // by returning an empty array on auth failure.

      if (data && data.length > 0) {
        frequentPages = data
          .sort((a, b) => b[1] - a[1])
          .filter(([path]) => path !== "/" && path !== "/iniciar-sessao/")
          .slice(0, 6);
      } else {
        // This case will be hit if getAnalytics returns an empty array due to auth or other issues
        frequentPages = [];
        // Don't set general error for auth issues, let it fail silently for this component
        // or show a specific "not available when logged out" message if desired.
        // If data is empty for other reasons, you might still want to log or set an error.
        if (data.length === 0 && !error) { // Avoid overwriting other potential errors
            console.warn("FrequentPages: No analytics data received, possibly due to auth or empty data.");
            // error = "Não foi possível carregar as páginas frequentes"; // Keep this commented or conditional
        }
      }
    } catch (err: any) { // Catch any error that getAnalytics might still throw for non-API issues
      console.error("Error in FrequentPages onMount:", err);
      if (!err.message || !err.message.toLowerCase().includes("unauthorized")) {
           error = "Não foi possível carregar as páginas frequentes";
      } else {
           frequentPages = [];
           error = null;
      }
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
