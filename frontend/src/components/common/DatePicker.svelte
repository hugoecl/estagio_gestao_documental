<script lang="ts">
    import calendarIcon from "@assets/calendar_icon.svg?raw";
    import previousIcon from "@assets/next_icon.svg?raw";
    import nextIcon from "@assets/previous_icon.svg?raw";
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
        value = $bindable(),
        required = true,
    }: {
        range: boolean;
        formName?: string;
        value?: string;
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
        _resizeHandler?: (event: Event) => void;
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
                    "/",
                );
                if (range) {
                    const secondDate = getSecondDateFromCallyRange(
                        e.currentTarget.value,
                        "/",
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
        };
    });

    let dropdownContainer: HTMLElement;
    let originalParent: HTMLElement | null;

    async function handleDropdownToggle(open: boolean) {
        await tick();

        if (open) {
            // Save original parent
            originalParent = dropdownContent.parentElement;

            const modalParent = findModalParent(cally);

            // Create container if it doesn't exist
            if (!dropdownContainer) {
                dropdownContainer = document.createElement("div");
                dropdownContainer.className = "datepicker-portal";
                if (modalParent) {
                    modalParent.appendChild(dropdownContainer);
                } else {
                    document.body.appendChild(dropdownContainer);
                }
            }

            // Position function that can be reused
            const positionDropdown = () => {
                const inputRect = cally.getBoundingClientRect();
                const dropdownRect = dropdownContent.getBoundingClientRect();

                // If we're in a modal, get the modal boundaries
                if (modalParent) {
                    const modalBounds = modalParent.getBoundingClientRect();

                    // Calculate initial centered position
                    let left =
                        inputRect.left -
                        modalBounds.left +
                        inputRect.width / 2 -
                        dropdownRect.width / 2;
                    const top = modalBounds.height / 2 - dropdownRect.height;

                    // Ensure dropdown stays within modal boundaries
                    // Check if it would overflow right edge
                    if (left + dropdownRect.width > modalBounds.width - 20) {
                        // Right-align with 20px padding from modal edge
                        left = modalBounds.width - dropdownRect.width - 20;
                    }

                    // Check if it would overflow left edge
                    if (left < 20) {
                        // Left-align with 20px padding from modal edge
                        left = 20;
                    }

                    // Apply positions within modal context
                    dropdownContent.style.position = "absolute";
                    dropdownContent.style.top = `${top + 30}px`;
                    dropdownContent.style.left = `${Math.max(5, left)}px`;
                } else {
                    // Regular viewport positioning
                    const viewportWidth = document.documentElement.clientWidth;
                    let left =
                        inputRect.left +
                        inputRect.width / 2 -
                        dropdownRect.width / 2;

                    // Ensure dropdown stays in viewport
                    if (left + dropdownRect.width > viewportWidth - 20) {
                        left = viewportWidth - dropdownRect.width - 20;
                    }
                    if (left < 20) {
                        left = 20;
                    }

                    dropdownContent.style.position = "fixed";
                    dropdownContent.style.top = `${inputRect.bottom + 5}px`;
                    dropdownContent.style.left = `${left}px`;
                }
            };

            // Move to container and position
            dropdownContainer.appendChild(dropdownContent);
            positionDropdown();

            // Add scroll listener to update position
            const scrollHandler = () => positionDropdown();
            window.addEventListener("scroll", scrollHandler, true);

            const resizeHandler = () => positionDropdown();
            window.addEventListener("resize", resizeHandler);

            // Store handler reference to remove later
            dropdownContent._scrollHandler = scrollHandler;
            dropdownContent._resizeHandler = resizeHandler;
        } else if (
            originalParent &&
            dropdownContent.parentElement !== originalParent
        ) {
            // Remove scroll handler
            if (dropdownContent._scrollHandler) {
                window.removeEventListener(
                    "scroll",
                    dropdownContent._scrollHandler,
                    true,
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
    function findModalParent(element: HTMLElement): HTMLElement | null {
        return get(currentModal);
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
                        e.currentTarget.value,
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
        "dropdown select-none max-sm:w-[90%] w-full ",
        // positionEnd ? "dropdown-end" : range ? dropdownPosition : "dropdown-center",
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
            class="input input-bordered w-full cursor-pointer caret-transparent flex items-center"
            bind:this={cally}
        >
            {@html calendarIcon}
            <input
                class="flex-grow ml-2 truncate cursor-pointer"
                bind:this={dateValue}
                bind:value
                placeholder={range ? "dd/mm/aaaa - dd/mm/aaaa" : "dd/mm/aaaa"}
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
        class="dropdown-content rounded-box border border-zinc-200 bg-base-200 card-sm shadow mt-1 w-max"
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

    /* .input {
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
    } */
    .input {
        min-height: 3rem;
        height: auto;
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
