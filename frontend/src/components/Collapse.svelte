<script lang="ts">
  import type { Snippet } from "svelte";

  const { title, children }: { title: string; children: Snippet } = $props();
</script>

<div
  class="collapse collapse-arrow bg-base-100 border border-zinc-300 collapse-open overflow-visible"
>
  <input
    type="radio"
    name={title}
    checked={true}
    onclick={(e) => {
      e.preventDefault();
      const parent = e.currentTarget.parentElement!;
      parent.classList.toggle("collapse-open");
      parent.classList.toggle("collapse-close");
    }}
  />
  <div class="collapse-title font-semibold">{title}</div>

  <div class="collapse-content text-sm">
    {@render children?.()}
  </div>
</div>

<style>
  .collapse {
    border-radius: 0.3rem;
    z-index: 0;
  }
  input {
    cursor: pointer;
  }
</style>
