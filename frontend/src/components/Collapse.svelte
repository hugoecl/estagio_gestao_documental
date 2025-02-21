<script lang="ts">
  import type { Snippet } from "svelte";

  const { title, children }: { title: string; children: Snippet } = $props();

  let isOpen = $state(false);
</script>

<div class="collapse collapse-arrow bg-base-100 border border-base-300">
  <input
    type="radio"
    name={title}
    onmousedown={(e) => {
      e.preventDefault();
      e.currentTarget.checked = !e.currentTarget.checked;
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
  }
  input {
    cursor: pointer;
  }
</style>
