<script lang="ts">
  import calendarIcon from "@assets/calendar_icon.svg?raw";
  import previousIcon from "@assets/next_icon.svg?raw";
  import nextIcon from "@assets/previous_icon.svg?raw";
  import { onMount } from "svelte";

  const { range }: { range: boolean } = $props();

  let cally: HTMLInputElement;
  let yearSelectElement: HTMLSelectElement;
  // unique id for the popover
  const uniqueId = Math.random().toString(36).substring(7);

  // svelte throws this warning because we are binding an element that is inside a if statement but in this case since the if statement is controlled by a prop it is safe to ignore this warning
  // svelte-ignore non_reactive_update
  let calendar: any;

  const dates: number[] = [];
  const currentYear = new Date().getFullYear();
  for (let i = -10; i <= 10; i++) {
    dates.push(currentYear + i);
  }

  onMount(() => {
    import("cally");

    // for checking if the value has really changed or if the user just chenged week/month/date on the calendar
    let oldValue: string;

    // on svelte 7.0.4 and cally 0.8.0 we have to add the event listener like this
    // for some reason the onchange svelte event is not working
    calendar?.addEventListener("focusday", (e: any) => {
      yearSelectElement.value = e.detail.getUTCFullYear();

      if (
        e.currentTarget.value.length !== 0 &&
        oldValue !== e.currentTarget.value
      ) {
        oldValue = e.currentTarget.value;
        if (range) {
          const [start, _] = e.currentTarget.value.split("/");
          cally.value = `${new Date(start).toLocaleDateString(
            "pt-PT"
          )} - ${e.detail.toLocaleDateString("pt-PT")}`;
        } else {
          cally.value = e.detail.toLocaleDateString("pt-PT");
        }
        cally.click();
        cally.style.opacity = "1";
      }
    });
  });
</script>

{#snippet yearSelect()}
  <div slot="heading">
    <select
      bind:this={yearSelectElement}
      class="select select-secondary"
      onchange={(e) => {
        const date = new Date(calendar.value);
        // @ts-ignore
        date.setUTCFullYear(e.currentTarget.value);
        calendar.focusedDate = date.toISOString().slice(0, 10);
      }}
    >
      {#each dates as year}
        {#if year === currentYear}
          <option value={year} selected>{year}</option>
        {:else}
          <option value={year}>{year}</option>
        {/if}
      {/each}
    </select>
  </div>
{/snippet}

<label class="input hover:shadow-md hover:border-secondary">
  {@html calendarIcon}
  <input
    popovertarget={uniqueId}
    class="grow"
    bind:this={cally}
    type="button"
    required
    value={range ? "dd/mm/aaaa - dd/mm/aaaa" : "dd/mm/aaaa"}
    readonly
  />
</label>

<div
  popover="auto"
  id={uniqueId}
  class="dropdown bg-base-100 rounded-box shadow-lg mt-2"
>
  {#if range}
    <calendar-range
      class="cally border border-zinc-200"
      months={2}
      bind:this={calendar}
    >
      {@render yearSelect()}
      {@html previousIcon}
      {@html nextIcon}
      <div class="grid grid-cols-2 gap-4">
        <calendar-month></calendar-month>
        <calendar-month offset={1}></calendar-month>
      </div>
    </calendar-range>
  {:else}
    <calendar-date class="cally border border-zinc-200" bind:this={calendar}>
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

  .input {
    width: 100%;
  }

  input {
    text-align: left;
    opacity: 0.5;
  }

  input:hover {
    opacity: 1;
  }

  div[popover] {
    transition: all 0.3s ease;
  }

  @media (max-width: 640px) {
    .input {
      width: 90%;
    }
  }
</style>
