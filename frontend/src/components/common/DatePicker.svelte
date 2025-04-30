<script lang="ts">
    import calendarIcon from "@assets/calendar_icon.svg?raw";
    import previousIcon from "@assets/next_icon.svg?raw"; // Check if these are correct icons
    import nextIcon from "@assets/previous_icon.svg?raw"; // Check if these are correct icons
    import { currentModal } from "@stores/modal-store";
    import {
        getFirstDateFromCallyRange,
        getFirstDateFromRangeToYMD,
        getSecondDateFromCallyRange,
        getSecondDateFromRangeToYMD,
    } from "src/utils/date-utils";
    import { onMount, tick } from "svelte";
    import { get } from "svelte/store";

    let {
        range,
        formName,
        value = $bindable(), // Use bindable for two-way binding
        required = true,
        inputClass = "",
        disabled = false,
        onchange, // Prop for change event
        onblur, // Prop for blur event
    }: {
        range: boolean;
        formName?: string;
        value?: string | [string, string] | null;
        required?: boolean;
        inputClass?: string;
        disabled?: boolean;
        onchange?: (event: Event) => void;
        onblur?: (event: FocusEvent) => void;
    } = $props();

    let dropdownPosition = $state("dropdown-center");
    let yearSelectElement: HTMLSelectElement;
    let calendar: any; // Instance of the cally component
    let oldValue: string | undefined; // Store previous cally value (YYYY/MM/DD or YYYY/MM/DD-YYYY/MM/DD)

    // Format the value for display (dd/mm/yyyy or dd/mm/yyyy - dd/mm/yyyy)
    const displayValue = $derived.by(() => {
        if (
            Array.isArray(value) &&
            value.length === 2 &&
            value[0] &&
            value[1]
        ) {
            return `${value[0]} - ${value[1]}`;
        } else if (typeof value === "string") {
            return value;
        }
        return "";
    });

    // Format the value for Cally (YYYY/MM/DD or YYYY/MM/DD-YYYY/MM/DD)
    const callyValue = $derived.by(() => {
        try {
            if (
                Array.isArray(value) &&
                value.length === 2 &&
                value[0] &&
                value[1]
            ) {
                // Range: value is ["dd/mm/yyyy", "dd/mm/yyyy"]
                const [firstYMD, year] = getFirstDateFromRangeToYMD(
                    value[0],
                    "/",
                );
                const secondYMD = getSecondDateFromRangeToYMD(value[1], "/");
                if (yearSelectElement) yearSelectElement.value = year;
                const result = `${firstYMD}/${secondYMD}`;
                // Don't update oldValue here, let the event listener do it
                if (calendar && calendar.value !== result)
                    calendar.focusedDate = result;
                return result;
            } else if (
                typeof value === "string" &&
                value &&
                /^\d{2}\/\d{2}\/\d{4}$/.test(value)
            ) {
                // Single date: value is "dd/mm/yyyy"
                const [firstYMD, year] = getFirstDateFromRangeToYMD(value, "/");
                if (yearSelectElement) yearSelectElement.value = year;
                // Don't update oldValue here
                if (calendar && calendar.value !== firstYMD)
                    calendar.focusedDate = firstYMD;
                return firstYMD;
            }
        } catch (e) {
            console.error("Error processing date value for cally:", value, e);
        }
        // Handle null, undefined, or invalid initial value
        if (calendar) calendar.focusedDate = undefined;
        return undefined; // Cally expects undefined for empty
    });

    interface ExtendedHTMLDivElement extends HTMLDivElement {
        _scrollHandler?: (event: Event) => void;
        _resizeHandler?: (event: Event) => void;
    }

    let callyContainer: HTMLDivElement; // The div acting as the input
    let dateValueInput: HTMLInputElement; // The hidden input holding the display value
    let detailsDropdown: HTMLDetailsElement;
    let dropdownContent: ExtendedHTMLDivElement;

    const dates: number[] = [];
    const now: Date = new Date();
    const nowISOString = now.toISOString().substring(0, 10).replace(/-/g, "/"); // Use / for cally
    const currentYear = now.getFullYear();
    for (let i = -10; i <= 10; i++) {
        dates.push(currentYear + i);
    }

    onMount(() => {
        import("cally");

        function handleDocumentClick(e: MouseEvent) {
            if (disabled) return;
            if (
                detailsDropdown.open &&
                !detailsDropdown.contains(e.target as Node) &&
                !callyContainer.contains(e.target as Node) &&
                !dropdownContent.contains(e.target as Node)
            ) {
                detailsDropdown.open = false;
                // handleDropdownToggle(false); // Let the ontoggle event handle this
            }
        }

        document.addEventListener("mousedown", handleDocumentClick);

        const rem = parseFloat(
            getComputedStyle(document.documentElement).fontSize,
        );
        let windowSize = window.innerWidth / rem;

        function setDropdownPosition() {
            // Simplified position logic, adjust if needed
            dropdownPosition = "dropdown-end"; // Default to end, better for forms
        }

        setDropdownPosition(); // Initial position
        window.addEventListener("resize", setDropdownPosition);

        // --- Cally Event Listener ---
        const handleCallyFocusDay = (e: any) => {
            if (disabled) return;

            const targetCalendar = e.currentTarget; // Get the specific calendar instance
            const callyCurrentValue = targetCalendar.value; // YYYY/MM/DD or YYYY/MM/DD-YYYY/MM/DD

            if (yearSelectElement) {
                yearSelectElement.value = e.detail.getUTCFullYear().toString();
            }

            // Check if the value actually changed from the *last selected* value
            if (callyCurrentValue && oldValue !== callyCurrentValue) {
                oldValue = callyCurrentValue; // Update internal tracking

                let newValue: string | [string, string] | null;
                if (range) {
                    const firstDate = getFirstDateFromCallyRange(
                        callyCurrentValue,
                        "/",
                    );
                    const secondDate = getSecondDateFromCallyRange(
                        callyCurrentValue,
                        "/",
                    );
                    newValue = [firstDate, secondDate];
                } else {
                    newValue = getFirstDateFromCallyRange(
                        callyCurrentValue,
                        "/",
                    );
                }

                // Update the bound value prop
                value = newValue;

                // Manually trigger the onchange event passed as prop
                if (onchange) {
                    // Simulate event target for parent component
                    const syntheticEvent = {
                        target: { value: displayValue }, // Pass the display value
                        currentTarget: dateValueInput, // Reference the hidden input
                        // Add other event properties if needed
                    } as unknown as Event;
                    onchange(syntheticEvent);
                }

                // Close dropdown after selection
                detailsDropdown.open = false;
                // handleDropdownToggle(false); // Let ontoggle handle visual hiding
            }
        };

        if (calendar) {
            calendar.removeEventListener("focusday", handleCallyFocusDay); // Clean up just in case
            calendar.addEventListener("focusday", handleCallyFocusDay);
            calendar._hasListenerAttached = true; // Mark as attached
        }

        return () => {
            document.removeEventListener("mousedown", handleDocumentClick);
            window.removeEventListener("resize", setDropdownPosition);
            if (calendar)
                calendar.removeEventListener("focusday", handleCallyFocusDay);
            if (dropdownContainer && dropdownContainer.parentNode) {
                dropdownContainer.parentNode.removeChild(dropdownContainer);
            }
        };
    });

    let dropdownContainer: HTMLElement;
    let originalParent: HTMLElement | null;

    async function handleDropdownToggle(open: boolean) {
        await tick();
        if (disabled) {
            detailsDropdown.open = false; // Ensure it stays closed if disabled
            return;
        }

        if (open) {
            originalParent = dropdownContent.parentElement;
            const modalParent = findModalParent(callyContainer);

            if (!dropdownContainer) {
                dropdownContainer = document.createElement("div");
                dropdownContainer.className = "datepicker-portal z-[10001]";
                if (modalParent) modalParent.appendChild(dropdownContainer);
                else document.body.appendChild(dropdownContainer);
            }

            const positionDropdown = () => {
                if (!detailsDropdown.open) return;
                const inputRect = callyContainer.getBoundingClientRect();
                const dropdownRect = dropdownContent.getBoundingClientRect();

                let top = inputRect.bottom + 5;
                let left =
                    inputRect.left +
                    inputRect.width / 2 -
                    dropdownRect.width / 2;
                let position: "fixed" | "absolute" = "fixed";

                if (modalParent) {
                    const modalBounds = modalParent.getBoundingClientRect();
                    position = "absolute";
                    top = inputRect.bottom - modalBounds.top + 5;
                    left =
                        inputRect.left -
                        modalBounds.left +
                        inputRect.width / 2 -
                        dropdownRect.width / 2;

                    if (left + dropdownRect.width > modalBounds.width - 10)
                        left = modalBounds.width - dropdownRect.width - 10;
                    if (left < 10) left = 10;
                    if (top + dropdownRect.height > modalBounds.height - 10)
                        top =
                            inputRect.top -
                            modalBounds.top -
                            dropdownRect.height -
                            5;
                } else {
                    const viewportWidth = document.documentElement.clientWidth;
                    const viewportHeight =
                        document.documentElement.clientHeight;
                    if (left + dropdownRect.width > viewportWidth - 10)
                        left = viewportWidth - dropdownRect.width - 10;
                    if (left < 10) left = 10;
                    if (top + dropdownRect.height > viewportHeight - 10)
                        top = inputRect.top - dropdownRect.height - 5;
                }

                dropdownContent.style.position = position;
                dropdownContent.style.top = `${Math.max(5, top)}px`;
                dropdownContent.style.left = `${Math.max(5, left)}px`;
                dropdownContent.style.opacity = "1";
            };

            dropdownContainer.appendChild(dropdownContent);
            positionDropdown();

            const scrollHandler = () => positionDropdown();
            window.addEventListener("scroll", scrollHandler, true);
            const resizeHandler = () => positionDropdown();
            window.addEventListener("resize", resizeHandler);

            dropdownContent._scrollHandler = scrollHandler;
            dropdownContent._resizeHandler = resizeHandler;
        } else if (
            originalParent &&
            dropdownContent.parentElement !== originalParent
        ) {
            if (dropdownContent._scrollHandler)
                window.removeEventListener(
                    "scroll",
                    dropdownContent._scrollHandler,
                    true,
                );
            if (dropdownContent._resizeHandler)
                window.removeEventListener(
                    "resize",
                    dropdownContent._resizeHandler,
                );
            delete dropdownContent._scrollHandler;
            delete dropdownContent._resizeHandler;

            originalParent.appendChild(dropdownContent);
            dropdownContent.style.position = "absolute";
            dropdownContent.style.top = "";
            dropdownContent.style.left = "";
            dropdownContent.style.opacity = "0";
        } else {
            dropdownContent.style.opacity = "0";
        }
    }

    function findModalParent(element: HTMLElement): HTMLElement | null {
        return get(currentModal);
    }

    function handleYearChange(e: Event) {
        if (disabled || !calendar) return;
        const newYear = (e.target as HTMLSelectElement).value;
        let focusDateStr = nowISOString; // Default to today if no focus date

        if (calendar.focusedDate) {
            // Cally's focusedDate might be YYYY/MM/DD or YYYY/MM/DD-YYYY/MM/DD
            focusDateStr =
                calendar.focusedDate.length > 10
                    ? calendar.focusedDate.substring(0, 10) // Use start date of range
                    : calendar.focusedDate;
        }

        // Ensure focusDateStr is in YYYY/MM/DD format before replacing year
        if (focusDateStr && focusDateStr.includes("/")) {
            const currentYearInFocus = focusDateStr.substring(0, 4);
            calendar.focusedDate = focusDateStr.replace(
                currentYearInFocus,
                newYear,
            );
        } else {
            // Fallback if format is unexpected, focus on Jan 1st of new year
            calendar.focusedDate = `${newYear}/01/01`;
        }
    }
</script>

{#snippet yearSelect()}
    <div slot="heading">
        <select
            bind:this={yearSelectElement}
            class="select select-sm select-bordered"
            {disabled}
            onchange={handleYearChange}
        >
            {#each dates as year}
                <option value={year} selected={year === currentYear}
                    >{year}</option
                >
            {/each}
        </select>
    </div>
{/snippet}

<details
    class="dropdown w-full"
    bind:this={detailsDropdown}
    ontoggle={(e) =>
        handleDropdownToggle((e.target as HTMLDetailsElement).open)}
>
    <summary
        class="list-none marker:hidden"
        role="button"
        aria-haspopup="dialog"
        aria-expanded={detailsDropdown?.open ?? false}
        onclick={(e) => {
            if (disabled) e.preventDefault();
        }}
        {onblur}
    >
        <div
            tabindex={disabled ? -1 : 0}
            class="input input-bordered w-full cursor-pointer caret-transparent flex items-center {inputClass}"
            class:input-disabled={disabled}
            class:input-error={inputClass.includes("input-error")}
            bind:this={callyContainer}
            aria-label="Select date"
        >
            {@html calendarIcon}
            <span
                class="flex-grow ml-2 truncate"
                class:opacity-50={!displayValue && !disabled}
            >
                {displayValue ||
                    (range ? "dd/mm/aaaa - dd/mm/aaaa" : "dd/mm/aaaa")}
            </span>
            <input
                type="hidden"
                bind:this={dateValueInput}
                value={displayValue}
                name={formName}
                {required}
                {disabled}
            />
        </div>
    </summary>
    <div
        tabindex="-1"
        class="dropdown-content rounded-box border border-base-content/10 bg-base-100 shadow-lg mt-1 w-max p-2 opacity-0 transition-opacity duration-100"
        bind:this={dropdownContent}
        aria-hidden={!(detailsDropdown?.open ?? false)}
    >
        {#if range}
            <calendar-range
                class="cally"
                months={2}
                bind:this={calendar}
                value={callyValue}
                locale="pt-PT"
                {disabled}
            >
                {@render yearSelect()}
                {@html previousIcon}
                {@html nextIcon}
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
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
                {disabled}
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
        background-color: var(--fallback-p, oklch(var(--p) / 1));
        color: var(--fallback-pc, oklch(var(--pc) / 1));
    }
    calendar-month::part(selected) {
        background-color: var(--fallback-p, oklch(var(--p) / 1));
        color: var(--fallback-pc, oklch(var(--pc) / 1));
    }
    calendar-month::part(in-range) {
        background-color: var(--fallback-p, oklch(var(--p) / 0.2));
    }

    /* Ensure the input div looks like other inputs */
    .input {
        min-height: 3rem; /* Match default input height */
        height: auto; /* Allow height to adjust slightly if needed */
        padding-left: 0.75rem;
        padding-right: 0.75rem;
        color: black;
    }

    .input:hover:not(.input-disabled),
    .input:focus:not(.input-disabled),
    .input:focus-within:not(.input-disabled) {
        /* Add focus-within */
        border-color: var(--fallback-a, oklch(var(--a) / 1));
        outline: 2px solid transparent; /* Standard focus outline */
        outline-offset: 2px;
        opacity: 1;
    }
    .input-disabled {
        cursor: not-allowed;
        background-color: var(--fallback-b2, oklch(var(--b2) / 1));
        border-color: var(--fallback-b2, oklch(var(--b2) / 1));
        opacity: 0.5;
    }
    .input-disabled span,
    .input-disabled svg {
        cursor: not-allowed;
        opacity: 0.5;
    }

    /* Portal styling */
    .datepicker-portal {
        position: absolute;
        z-index: 10001 !important;
    }
    .dropdown-content {
        z-index: 10002 !important;
    }
</style>
