<script lang="ts">
  import calendarIcon from "@assets/calendar_icon.svg?raw";
  import previousIcon from "@assets/next_icon.svg?raw";
  import nextIcon from "@assets/previous_icon.svg?raw";
  import { onMount } from "svelte";

  const { range }: { range: boolean } = $props();

  let cally: HTMLButtonElement;
  let dateSpan: HTMLSpanElement;
  let calendar: any;

  // create a const named dates that is an array with all the dates from the past 10 years to the next 10 years
  const dates: number[] = [];
  const currentYear = new Date().getFullYear();
  for (let i = -10; i <= 10; i++) {
    dates.push(currentYear + i);
  }

  onMount(() => {
    import("cally");

    cally = document.getElementById("cally") as HTMLButtonElement;
    dateSpan = cally.querySelector("span")!;

    calendar = range
      ? document.querySelector("calendar-range")
      : document.querySelector("calendar-date");

    const yearSelect = document.getElementById(
      "yearSelect"
    ) as HTMLSelectElement;
    // for checking if the value has really changed or if the user just chenged week/month/date on the calendar
    let oldValue: string;

    // on svelte 7.0.4 and cally 0.8.0 we have to add the event listener like this
    // for some reason the onchange svelte event is not working
    calendar?.addEventListener("focusday", (e: any) => {
      yearSelect.value = e.detail.getUTCFullYear();

      if (
        e.currentTarget.value.length !== 0 &&
        oldValue !== e.currentTarget.value
      ) {
        oldValue = e.currentTarget.value;
        if (range) {
          const [start, _] = e.currentTarget.value.split("/");
          dateSpan.innerHTML = `${new Date(start).toLocaleDateString(
            "pt-PT"
          )} - ${e.detail.toLocaleDateString("pt-PT")}`;
        } else {
          dateSpan.innerHTML = e.detail.toLocaleDateString("pt-PT");
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

{#snippet yearSelect()}
  <div slot="heading">
    <select
      id="yearSelect"
      class="select select-secondary"
      onchange={(e) => {
        const date = new Date(calendar.value);
        // @ts-ignore
        date.setUTCFullYear(e.currentTarget.value);

        calendar.focusedDate = date.toISOString().slice(0, 10);
      }}
    >
      {#each dates as year}
        {#if year === new Date().getFullYear()}
          <option value={year} selected>{year}</option>
        {:else}
          <option value={year}>{year}</option>
        {/if}
      {/each}
    </select>
  </div>
{/snippet}

<div
  popover="auto"
  id="cally-popover"
  class="dropdown bg-base-100 rounded-box shadow-lg"
  style="position-anchor:--cally"
>
  {#if range}
    <calendar-range class="cally border border-zinc-200" months={2}>
      {@render yearSelect()}
      {@html previousIcon}
      {@html nextIcon}
      <div class="grid grid-cols-2 gap-4">
        <calendar-month></calendar-month>
        <calendar-month offset={1}></calendar-month>
      </div>
    </calendar-range>
  {:else}
    <calendar-date class="cally">
      {@render yearSelect()}
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
  calendar-month::part(range-start),
  calendar-month::part(range-end) {
    background-color: var(--color-secondary);
  }
</style>
