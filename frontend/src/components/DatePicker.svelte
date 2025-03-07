<script lang="ts">
  import calendarIcon from "@assets/calendar_icon.svg?raw";
  import previousIcon from "@assets/next_icon.svg?raw";
  import nextIcon from "@assets/previous_icon.svg?raw";
  import { onMount } from "svelte";

  let {
    range,
    formName,
    value = $bindable(),
  }: {
    range: boolean;
    formName?: string;
    value?: string;
  } = $props();

  let dropdownPosition = $state("dropdown-center");

  let cally: HTMLDivElement;
  let yearSelectElement: HTMLSelectElement;
  let dateValue: HTMLInputElement;
  let detailsDropdown: HTMLDetailsElement;
  let dropdownContent: HTMLDivElement;

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

    function handleToggle(e: MouseEvent) {
      if (
        detailsDropdown.open &&
        !detailsDropdown.contains(e.target as Node) &&
        !cally.contains(e.target as Node)
      ) {
        detailsDropdown.open = false;
      }
    }

    document.addEventListener("mousedown", handleToggle);

    if (range) {
      const rem = parseFloat(
        getComputedStyle(document.documentElement).fontSize
      );
      let windowSize = window.innerWidth / rem;

      function setDropdownPosition() {
        windowSize = window.innerWidth / rem;

        if (windowSize < 40) {
          dropdownPosition = "dropdown-center";
        } else if (windowSize < 48) {
          dropdownPosition = "dropdown-end";
        } else if (windowSize < 80) {
          dropdownPosition = "dropdown-end";
        } else {
          dropdownPosition = "dropdown-center";
        }
      }
      setDropdownPosition();

      window.addEventListener("resize", setDropdownPosition);
    }

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
          dateValue.value = `${new Date(start).toLocaleDateString(
            "pt-PT"
          )} - ${e.detail.toLocaleDateString("pt-PT")}`;
        } else {
          dateValue.value = e.detail.toLocaleDateString("pt-PT");
        }

        // @ts-ignore
        detailsDropdown.open = false;
        cally.style.opacity = "1";
      }
    });
    return () => {
      document.removeEventListener("mousedown", handleToggle);
      if (range && window.location.pathname === "/contratos") {
        // @ts-ignore typescript thinks the setDropdownPosition function is not defined but int his condition it is
        window.removeEventListener("resize", setDropdownPosition);
      }
    };
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

<details
  class={[
    "dropdown select-none max-sm:w-[90%]",
    range ? dropdownPosition : "dropdown-center",
  ]}
  bind:this={detailsDropdown}
>
  <summary
    class="list-none"
    onclick={() => {
      detailsDropdown.open = !detailsDropdown.open;
      dropdownContent.style.opacity = detailsDropdown.open ? "1" : "0";
    }}
  >
    <div
      tabindex="0"
      role="button"
      class="input cursor-pointer caret-transparent"
      bind:this={cally}
    >
      {@html calendarIcon}
      <input
        class="cursor-pointer"
        bind:this={dateValue}
        bind:value
        placeholder={range ? "dd/mm/aaaa" : "dd/mm/aaaa - dd/mm/aaaa"}
        name={formName}
        onkeydown={(e) => e.preventDefault()}
        oninput={(e) => e.preventDefault()}
        required
      />
    </div>
  </summary>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    tabindex="0"
    class="dropdown-content rounded-box border border-zinc-200 bg-base-100 card-sm shadow-lg mt-1 w-max"
    bind:this={dropdownContent}
  >
    {#if range}
      <calendar-range class="cally" months={2} bind:this={calendar}>
        {@render yearSelect()}
        {@html previousIcon}
        {@html nextIcon}
        <div class="grid grid-cols-2 gap-4">
          <calendar-month></calendar-month>
          <calendar-month offset={1}></calendar-month>
        </div>
      </calendar-range>
    {:else}
      <calendar-date class="cally" bind:this={calendar}>
        {@render yearSelect()}
        {@html previousIcon}
        {@html nextIcon}
        <calendar-month></calendar-month>
      </calendar-date>
    {/if}
  </div>
</details>

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
  input::placeholder {
    color: var(--color-base);
    opacity: 0.5;
  }

  input:hover::placeholder {
    opacity: 1;
  }

  .input:hover,
  .input:focus {
    border-color: var(--color-secondary);
    box-shadow:
      0 4px 6px -1px rgb(0 0 0 / 0.1),
      0 2px 4px -2px rgb(0 0 0 / 0.1);
    opacity: 1;
  }
</style>
