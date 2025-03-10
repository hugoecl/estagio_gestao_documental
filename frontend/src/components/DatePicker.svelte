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

  // for checking if the value has really changed or if the user just chenged week/month/date on the calendar
  let oldValue: string;
  const callyValue = $derived.by(() => {
    if (value) {
      // callyValue comes in the format dd/mm/yyyy - dd/mm/yyyy
      const firstYear = value.substring(6, 10);
      const firstMonth = value.substring(3, 5);
      const firstDay = value.substring(0, 2);
      if (range) {
        const secondYear = value.substring(19, 23);
        const secondMonth = value.substring(16, 18);
        const secondDay = value.substring(13, 15);

        const result = `${firstYear}-${firstMonth}-${firstDay}/${secondYear}-${secondMonth}-${secondDay}`;
        oldValue = result;
        return result;
      }
      // callyValue comes in the format dd/mm/yyyy
      else {
        const result = `${firstYear}-${firstMonth}-${firstDay}`;
        oldValue = result;
        return result;
      }
    }
  });

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

    calendar?.addEventListener("focusday", (e: any) => {
      yearSelectElement.value = e.detail.getUTCFullYear();

      if (
        e.currentTarget.value.length !== 0 &&
        oldValue !== e.currentTarget.value
      ) {
        oldValue = e.currentTarget.value;

        // e.currentTarget.value is something like 2012/12/24-2012/12/25
        const firstYear = e.currentTarget.value.substring(0, 4);
        const firstMonth = e.currentTarget.value.substring(5, 7);
        const firstDay = e.currentTarget.value.substring(8, 10);
        if (range) {
          const secondYear = e.currentTarget.value.substring(11, 15);
          const secondMonth = e.currentTarget.value.substring(16, 18);
          const secondDay = e.currentTarget.value.substring(19, 21);

          dateValue.value = `${firstDay}/${firstMonth}/${firstYear} - ${secondDay}/${secondMonth}/${secondYear}`;
          value = dateValue.value;
        } else {
          dateValue.value = `${firstDay}/${firstMonth}/${firstYear}`;
          value = dateValue.value;
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
        const currentYear = calendar.value.substring(0, 4);
        calendar.focusedDate = calendar.value.replace(
          currentYear,
          e.currentTarget.value
        );
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
      <calendar-range
        class="cally"
        months={2}
        bind:this={calendar}
        value={callyValue}
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
      <calendar-date class="cally" bind:this={calendar} value={callyValue}>
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
