<script lang="ts">
  import calendarIcon from "@assets/calendar_icon.svg?raw";
  import previousIcon from "@assets/next_icon.svg?raw";
  import nextIcon from "@assets/previous_icon.svg?raw";
  import "cally";
  import { onMount } from "svelte";

  const { range }: { range: boolean } = $props();

  let cally: HTMLButtonElement;
  let dateSpan: HTMLSpanElement;

  onMount(() => {
    cally = document.getElementById("cally") as HTMLButtonElement;
    dateSpan = cally.querySelector("span")!;

    const calendar = range
      ? document.querySelector("calendar-range")
      : document.querySelector("calendar-date");

    // on svelte 7.0.4 and cally 0.8.0 we have to add the event listener like this
    // for some reason the onchange svelte event is not working
    calendar?.addEventListener("change", (e: any) => {
      if (e.currentTarget.value.length !== 0) {
        if (range) {
          const [start, end] = e.currentTarget.value.split("/");
          dateSpan.innerHTML = `${new Date(start).toLocaleDateString(
            "pt-PT"
          )} - ${new Date(end).toLocaleDateString("pt-PT")}`;
        } else {
          dateSpan.innerHTML = new Date(
            e.currentTarget.value
          ).toLocaleDateString("pt-PT");
        }
        cally.click();
      }
    });
  });
</script>

<button
  popovertarget="cally-popover"
  class="input input-border"
  id="cally"
  style="anchor-name:--cally"
>
  {@html calendarIcon}

  <span>
    {#if range}
      dd/mm/aaaa - dd/mm/aaaa
    {:else}
      dd/mm/aaaa
    {/if}
  </span>
</button>

<div
  popover="auto"
  id="cally-popover"
  class="dropdown bg-base-100 rounded-box shadow-lg"
  style="position-anchor:--cally"
>
  {#if range}
    <calendar-range class="cally" months={2}>
      {@html previousIcon}
      {@html nextIcon}
      <div class="grid grid-cols-2 gap-4">
        <calendar-month></calendar-month>
        <calendar-month offset={1}></calendar-month>
      </div>
    </calendar-range>
  {:else}
    <calendar-date class="cally">
      {@html previousIcon}
      {@html nextIcon}
      <calendar-month></calendar-month>
    </calendar-date>
  {/if}
</div>

<style>
  calendar-month::part(heading) {
    text-transform: capitalize;
  }
</style>
