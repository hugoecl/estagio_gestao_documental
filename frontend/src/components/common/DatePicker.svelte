<script lang="ts">
  import calendarIcon from "@assets/calendar_icon.svg?raw";
  import previousIcon from "@assets/next_icon.svg?raw";
  import nextIcon from "@assets/previous_icon.svg?raw";
  import {
    getFirstDateFromCallyRange,
    getFirstDateFromRangeToYMD,
    getSecondDateFromCallyRange,
    getSecondDateFromRangeToYMD,
  } from "src/utils/date-utils";
  import { onMount, tick } from "svelte";

  let {
    range,
    formName,
    value = $bindable(),
    positionEnd,
    required = true,
  }: {
    range: boolean;
    formName?: string;
    value?: string;
    positionEnd?: boolean;
    required?: boolean;
  } = $props();

  let dropdownPosition = $state("dropdown-center");
  let yearSelectElement: HTMLSelectElement;

  // svelte throws this warning because we are binding an element that is inside a if statement but in this case since the if statement is controlled by a prop it is safe to ignore this warning
  // svelte-ignore non_reactive_update
  let calendar: any;

  // for checking if the value has really changed or if the user just chenged week/month/date on the calendar
  let oldValue: string;
  const callyValue = $derived.by(() => {
    if (value) {
      // callyValue comes in the format dd/mm/yyyy - dd/mm/yyyy
      const [first, year] = getFirstDateFromRangeToYMD(value, "-");
      if (yearSelectElement) {
        yearSelectElement.value = year;
      }

      if (range) {
        const second = getSecondDateFromRangeToYMD(value, "-");

        const result = `${first}/${second}`;

        oldValue = result;
        if (calendar) {
          calendar.focusedDate = result;
        }
        return result;
      }
      // callyValue comes in the format dd/mm/yyyy
      else {
        oldValue = first;
        if (calendar) {
          calendar.focusedDate = first;
        }
        return oldValue;
      }
    }
  });

  interface ExtendedHTMLDivElement extends HTMLDivElement {
    _scrollHandler?: (event: Event) => void;
  }

  let cally: HTMLDivElement;
  let dateValue: HTMLInputElement;
  let detailsDropdown: HTMLDetailsElement;
  let dropdownContent: ExtendedHTMLDivElement;

  const dates: number[] = [];
  const now: Date = new Date();
  const nowISOString = now.toISOString().substring(0, 10);
  const currentYear = now.getFullYear();
  const currentYearString = currentYear.toString();
  for (let i = -10; i <= 10; i++) {
    dates.push(currentYear + i);
  }

  onMount(() => {
    import("cally");

    function handleToggle(e: MouseEvent) {
      if (
        detailsDropdown.open &&
        !detailsDropdown.contains(e.target as Node) &&
        !cally.contains(e.target as Node) &&
        // check if click is inside dropdown portal content
        !dropdownContent.contains(e.target as Node)
      ) {
        detailsDropdown.open = false;
        handleDropdownToggle(false);
      }
    }

    document.addEventListener("mousedown", handleToggle);

    const rem = parseFloat(getComputedStyle(document.documentElement).fontSize);

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

    if (range) {
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
        const firstDate = getFirstDateFromCallyRange(
          e.currentTarget.value,
          "/"
        );
        if (range) {
          const secondDate = getSecondDateFromCallyRange(
            e.currentTarget.value,
            "/"
          );

          dateValue.value = `${firstDate} - ${secondDate}`;
          value = dateValue.value;
        } else {
          dateValue.value = firstDate;
          value = dateValue.value;
        }

        // @ts-ignore
        detailsDropdown.open = false;
        handleDropdownToggle(false);
        cally.style.opacity = "1";
      }
    });
    return () => {
      document.removeEventListener("mousedown", handleToggle);
      if (range) {
        // @ts-ignore typescript thinks the setDropdownPosition function is not defined but int his condition it is
        window.removeEventListener("resize", setDropdownPosition);
      }
    };
  });

  let dropdownContainer: HTMLElement;
  let originalParent: HTMLElement | null;

  async function handleDropdownToggle(open: boolean) {
    await tick();

    if (open) {
      // Save original parent
      originalParent = dropdownContent.parentElement;

      // Create container if it doesn't exist
      if (!dropdownContainer) {
        dropdownContainer = document.createElement("div");
        dropdownContainer.className = "datepicker-portal";
        document.body.appendChild(dropdownContainer);
      }

      // Position function that can be reused
      const positionDropdown = () => {
        const inputRect = cally.getBoundingClientRect();
        const dropdownRect = dropdownContent.getBoundingClientRect();

        // Center the dropdown beneath the input
        const left =
          inputRect.left + inputRect.width / 2 - dropdownRect.width / 2;

        // Apply position
        dropdownContent.style.position = "fixed";
        dropdownContent.style.top = `${inputRect.bottom + 5}px`; // 5px gap
        dropdownContent.style.left = `${Math.max(5, left)}px`; // Prevent overflow left
        dropdownContent.style.opacity = "1";
      };

      // Move to container and position
      dropdownContainer.appendChild(dropdownContent);
      positionDropdown();

      // Add scroll listener to update position
      const scrollHandler = () => positionDropdown();
      window.addEventListener("scroll", scrollHandler, true);

      // Store handler reference to remove later
      dropdownContent._scrollHandler = scrollHandler;
    } else if (
      originalParent &&
      dropdownContent.parentElement !== originalParent
    ) {
      // Remove scroll handler
      if (dropdownContent._scrollHandler) {
        window.removeEventListener(
          "scroll",
          dropdownContent._scrollHandler,
          true
        );
        delete dropdownContent._scrollHandler;
      }

      // Restore to original position
      originalParent.appendChild(dropdownContent);
      dropdownContent.style.position = "absolute";
      dropdownContent.style.top = "";
      dropdownContent.style.left = "";
    }
  }
</script>

{#snippet yearSelect()}
  <div slot="heading">
    <select
      bind:this={yearSelectElement}
      class="select select-secondary"
      onchange={(e) => {
        if (calendar.focusedDate === undefined) {
          calendar.focusedDate = nowISOString.replace(
            currentYearString,
            e.currentTarget.value
          );
        } else {
          const focusedYear = calendar.focusedDate.substring(0, 4);
          calendar.focusedDate = calendar.focusedDate
            .substring(0, 10)
            .replace(focusedYear, e.currentTarget.value);
        }
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
    positionEnd ? "dropdown-end" : range ? dropdownPosition : "dropdown-center",
  ]}
  bind:this={detailsDropdown}
>
  <summary
    class="list-none"
    onclick={() => {
      detailsDropdown.open = !detailsDropdown.open;
      dropdownContent.style.opacity = detailsDropdown.open ? "1" : "0";
      handleDropdownToggle(detailsDropdown.open);
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
        {required}
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
        locale="pt-PT"
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
      <calendar-date
        class="cally"
        bind:this={calendar}
        value={callyValue}
        locale="pt-PT"
      >
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
