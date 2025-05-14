<script lang="ts">
    import calendarIcon from "@assets/calendar_icon.svg?raw";
    import previousIcon from "@assets/next_icon.svg?raw";
    import nextIcon from "@assets/previous_icon.svg?raw";
    import { currentModal } from "@stores/modal-store";
    import {
        DMYToYMD,
        getFirstDateFromCallyRange,
        getFirstDateFromRangeToYMD,
        getSecondDateFromCallyRange,
        getSecondDateFromRangeToYMD,
    } from "src/utils/date-utils";
    import { onMount, tick } from "svelte";
    import { get } from "svelte/store";

    // --- Props ---
    let {
        range,
        formName,
        value = $bindable(), // Use bindable for two-way binding from parent
        required = true,
        inputClass = "", // Keep optional props
        disabled = false,
        onchange, // Keep event handlers
        onblur,
    }: {
        range: boolean;
        formName?: string;
        value?: string | [string, string] | null; // Allow array for range
        required?: boolean;
        inputClass?: string;
        disabled?: boolean;
        onchange?: (event: Event) => void;
        onblur?: (event: FocusEvent) => void;
    } = $props();

    // --- State ---
    let yearSelectElement: HTMLSelectElement;
    let calendar: any; // Cally instance
    let detailsDropdown: HTMLDetailsElement;
    let dropdownContent: HTMLDivElement;
    let callyContainerDiv: HTMLDivElement; // The visible input div
    let hiddenDateInput: HTMLInputElement; // Hidden input for form submission if needed

    // Store previous Cally value (YYYY/MM/DD format) to detect actual user selection changes
    let previousCallyValue: string | undefined = undefined;

    // --- Derived Values ---
    // Display value (DD/MM/YYYY or DD/MM/YYYY - DD/MM/YYYY)
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

    // Cally value (YYYY/MM/DD or YYYY/MM/DD-YYYY/MM/DD)
    const callyValue = $derived.by(() => {
        try {
            if (
                Array.isArray(value) &&
                value.length === 2 &&
                value[0] &&
                value[1]
            ) {
                const firstYMD = value[0];
                const year = firstYMD.substring(6, 10);
                // const secondYMD = getSecondDateFromRangeToYMD(value[1], "/");
                const secondYMD = value[1];
                if (yearSelectElement && yearSelectElement.value !== year)
                    yearSelectElement.value = year;

                const result = `${DMYToYMD(firstYMD, "-")}/${DMYToYMD(secondYMD, "-")}`;
                if (calendar && calendar.value !== result) {
                    // If external change, update cally's focus and internal previous value
                    calendar.focusedDate = result;
                    previousCallyValue = result;
                }
                return result;
            } else if (typeof value === "string" && value) {
                if (!range) {
                    const firstYMD = DMYToYMD(value, "-");
                    const year = value.substring(6, 10);
                    if (yearSelectElement && yearSelectElement.value !== year)
                        yearSelectElement.value = year;
                    if (calendar && calendar.value !== firstYMD) {
                        calendar.focusedDate = firstYMD;
                        previousCallyValue = firstYMD;
                    }
                    return firstYMD;
                } else {
                    const firstYMD = DMYToYMD(value.substring(0, 10), "-");
                    const year = firstYMD.substring(0, 4);
                    if (yearSelectElement && yearSelectElement.value !== year)
                        yearSelectElement.value = year;

                    const secondYMD = DMYToYMD(value.substring(13, 23), "-");

                    const result = `${firstYMD}/${secondYMD}`;
                    if (calendar && calendar.value !== result) {
                        // If external change, update cally's focus and internal previous value
                        calendar.focusedDate = result;
                        previousCallyValue = result;
                    }
                    return result;
                }
            }
        } catch (e) {
            console.error("Error processing date value for cally:", value, e);
        }
        // if (calendar) calendar.focusedDate = undefined;
        // previousCallyValue = undefined;
        // return undefined;
    });

    // --- Year Select Options ---
    const dates: number[] = [];
    const now: Date = new Date();
    const nowISOString = now.toISOString().substring(0, 10);
    const currentYear = now.getFullYear();
    const currentYearString = currentYear.toString();
    const todayCallyFormat = `${currentYear}/${(now.getMonth() + 1).toString().padStart(2, "0")}/${now.getDate().toString().padStart(2, "0")}`;
    for (let i = -10; i <= 10; i++) {
        dates.push(currentYear + i);
    }

    // --- Lifecycle and Event Handlers ---
    onMount(() => {
        import("cally");

        // --- Outside Click Handler ---
        function handleDocumentClick(e: MouseEvent) {
            if (disabled) return;
            // Check if the click is outside the <details> element AND outside the portal-rendered dropdown content
            if (
                detailsDropdown &&
                detailsDropdown.open &&
                !detailsDropdown.contains(e.target as Node) &&
                dropdownContent &&
                !dropdownContent.contains(e.target as Node)
            ) {
                detailsDropdown.open = false; // Close the details element
                // The ontoggle handler will manage moving the content back
            }
        }
        document.addEventListener("mousedown", handleDocumentClick);

        const handleCallyFocusDay = (e: any) => {
            yearSelectElement.value = e.detail.getUTCFullYear();

            if (
                e.currentTarget.value.length !== 0 &&
                previousCallyValue !== e.currentTarget.value
            ) {
                previousCallyValue = e.currentTarget.value;

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

                    // displayValue = `${firstDate} - ${secondDate}`;
                    // value = displayValue;
                    value = `${firstDate} - ${secondDate}`;
                } else {
                    // displayValue= firstDate;
                    // value = displayValue.value;
                    value = firstDate;
                }

                // @ts-ignore
                detailsDropdown.open = false;
                handleDropdownToggle(false);
                callyContainerDiv.style.opacity = "1";
            }
        };

        // --- Cally Date Selection Handler ---
        // const handleCallyFocusDay = (e: any) => {
        //     if (disabled) return;

        //     if (yearSelectElement && e.detail) {
        //         yearSelectElement.value = e.detail.getUTCFullYear();
        //     }

        //     // Update only if Cally provides a value and it's different from the last processed one
        //     if (
        //         e.currentTarget.value.length !== 0 &&
        //         e.currentTarget.value !== previousCallyValue
        //     ) {
        //         previousCallyValue = e.currentTarget.value; // Update tracker

        //         let newValueToEmit: string | [string, string] | null = null;
        //         let isValidSelection = false;

        //         try {
        //             if (range) {
        //                 if (newCallyValue.includes("-")) {
        //                     // Complete range
        //                     const firstDate = getFirstDateFromCallyRange(
        //                         newCallyValue,
        //                         "/",
        //                     );
        //                     const secondDate = getSecondDateFromCallyRange(
        //                         newCallyValue,
        //                         "/",
        //                     );
        //                     newValueToEmit = [firstDate, secondDate];
        //                     isValidSelection = true;
        //                 }
        //             } else {
        //                 if (!newCallyValue.includes("-")) {
        //                     // Single date
        //                     newValueToEmit = getFirstDateFromCallyRange(
        //                         newCallyValue,
        //                         "/",
        //                     );
        //                     isValidSelection = true;
        //                 }
        //             }
        //         } catch (err) {
        //             console.error(
        //                 "Error parsing date from Cally:",
        //                 newCallyValue,
        //                 err,
        //             );
        //             isValidSelection = false;
        //         }

        //         if (isValidSelection) {
        //             value = newValueToEmit; // Update Svelte state

        //             if (onchange) {
        //                 // Call parent's onchange if provided
        //                 const newDisplay = Array.isArray(newValueToEmit)
        //                     ? `${newValueToEmit[0]} - ${newValueToEmit[1]}`
        //                     : newValueToEmit;
        //                 const syntheticEvent = {
        //                     target: { value: newDisplay },
        //                     currentTarget: hiddenDateInput,
        //                 } as unknown as Event;
        //                 onchange(syntheticEvent);
        //             }
        //             detailsDropdown.open = false; // Close dropdown
        //         }
        //     }
        // };

        // --- Attach Listener After Cally Renders ---
        // Use tick to wait for initial render, then check periodically until calendar exists
        const setupListener = () => {
            if (calendar && !calendar._hasListenerAttached) {
                calendar.removeEventListener("focusday", handleCallyFocusDay); // Ensure no duplicates
                calendar.addEventListener("focusday", handleCallyFocusDay);
                calendar._hasListenerAttached = true;
            } else if (!calendar) {
                requestAnimationFrame(setupListener); // Try again next frame
            }
        };
        tick().then(setupListener);

        // --- Cleanup ---
        return () => {
            document.removeEventListener("mousedown", handleDocumentClick);
            window.removeEventListener("resize", positionDropdown); // Use named function
            if (calendar)
                calendar.removeEventListener("focusday", handleCallyFocusDay);
            // Remove portal and its listeners if they exist
            if (dropdownContainer && dropdownContainer.parentNode) {
                dropdownContainer.parentNode.removeChild(dropdownContainer);
            }
            if (dropdownContent?._scrollHandler)
                window.removeEventListener(
                    "scroll",
                    dropdownContent._scrollHandler,
                    true,
                );
            if (dropdownContent?._resizeHandler)
                window.removeEventListener(
                    "resize",
                    dropdownContent._resizeHandler,
                );
        };
    });

    // --- Portal and Positioning ---
    let dropdownContainer: HTMLElement;
    let originalParent: HTMLElement | null;

    // Use a named function for positioning
    const positionDropdown = async () => {
        if (
            !detailsDropdown ||
            !detailsDropdown.open ||
            !callyContainerDiv ||
            !dropdownContent
        )
            return;

        await tick();

        dropdownContent.style.opacity = "0";
        dropdownContent.style.visibility = "hidden";
        dropdownContent.style.display = "block"; // Ensure it's rendered for measurement
        await tick();
        const dropdownRect = dropdownContent.getBoundingClientRect();
        dropdownContent.style.display = "";
        dropdownContent.style.visibility = "";

        if (dropdownRect.width === 0 || dropdownRect.height === 0) return; // Bail if no dimensions yet

        const inputRect = callyContainerDiv.getBoundingClientRect();
        const spaceBelow = window.innerHeight - inputRect.bottom - 10;
        const spaceAbove = inputRect.top - 10;

        let top: number;
        let left = inputRect.left; // Default alignment
        let position: "fixed" | "absolute" = "fixed";
        const modalParent = findModalParent(callyContainerDiv);

        // Vertical placement
        if (spaceBelow >= dropdownRect.height || spaceBelow >= spaceAbove) {
            top = inputRect.bottom + 5; // Place below
        } else {
            top = inputRect.top - dropdownRect.height - 5; // Place above
        }

        // Horizontal placement (try to center, then align left/right)
        left = inputRect.left + inputRect.width / 2 - dropdownRect.width / 2;

        // Clamp to viewport
        left = Math.max(10, left);
        left = Math.min(left, window.innerWidth - dropdownRect.width - 10);
        top = Math.max(10, top);
        top = Math.min(top, window.innerHeight - dropdownRect.height - 10);

        // Adjust for modal context
        if (modalParent) {
            const modalBounds = modalParent.getBoundingClientRect();
            position = "absolute";
            // Calculate relative positions, considering modal scroll
            top = top - modalBounds.top + modalParent.scrollTop;
            left = left - modalBounds.left + modalParent.scrollLeft;

            // Re-clamp within modal (approximate)
            left = Math.max(5, left);
            top = Math.max(5, top);
            left = Math.min(left, modalBounds.width - dropdownRect.width - 5);
            top = Math.min(top, modalBounds.height - dropdownRect.height - 5); // Less reliable if modal isn't full height
        }

        dropdownContent.style.position = position;
        dropdownContent.style.top = `${top}px`;
        dropdownContent.style.left = `${left}px`;
        dropdownContent.style.opacity = "1";
    };

    async function handleDropdownToggle(open: boolean) {
        await tick();
        if (disabled) {
            detailsDropdown.open = false;
            return;
        }

        if (open) {
            originalParent = dropdownContent.parentElement;
            if (!dropdownContainer) {
                dropdownContainer = document.createElement("div");
                dropdownContainer.className = "datepicker-portal z-[10001]"; // High z-index
                const modalCtx = findModalParent(callyContainerDiv);
                (modalCtx || document.body).appendChild(dropdownContainer);
            }
            // Ensure content is in portal before positioning
            if (dropdownContent.parentElement !== dropdownContainer) {
                dropdownContainer.appendChild(dropdownContent);
                await tick();
            }
            requestAnimationFrame(positionDropdown); // Use rAF for smoother initial positioning

            // Add scroll/resize listeners for dynamic repositioning
            if (!dropdownContent._scrollHandler) {
                dropdownContent._scrollHandler = positionDropdown;
                window.addEventListener(
                    "scroll",
                    dropdownContent._scrollHandler,
                    true,
                );
            }
            if (!dropdownContent._resizeHandler) {
                dropdownContent._resizeHandler = positionDropdown;
                window.addEventListener(
                    "resize",
                    dropdownContent._resizeHandler,
                );
            }
        } else {
            // When closing
            // Remove listeners
            if (dropdownContent?._scrollHandler)
                window.removeEventListener(
                    "scroll",
                    dropdownContent._scrollHandler,
                    true,
                );
            if (dropdownContent?._resizeHandler)
                window.removeEventListener(
                    "resize",
                    dropdownContent._resizeHandler,
                );
            delete dropdownContent._scrollHandler;
            delete dropdownContent._resizeHandler;

            // Move back to original parent if it was moved and exists
            if (
                originalParent &&
                dropdownContent.parentElement === dropdownContainer
            ) {
                originalParent.appendChild(dropdownContent);
                dropdownContent.style.position = "absolute";
                dropdownContent.style.top = "";
                dropdownContent.style.left = "";
            }
            dropdownContent.style.opacity = "0";
            // Ensure it's visually hidden quickly
            dropdownContent.style.top = "-9999px";
            dropdownContent.style.left = "-9999px";
        }
    }

    function findModalParent(element: HTMLElement): HTMLElement | null {
        return get(currentModal);
    }

    async function handleYearChange(e: Event) {
        // if (disabled || !calendar) return;
        // const newYear = (e.target as HTMLSelectElement).value;
        // let focusDateStr = todayCallyFormat; // Use Cally format

        // // Try to keep the current month/day
        // if (calendar.focusedDate) {
        //     const currentFocus =
        //         calendar.focusedDate.length > 10
        //             ? calendar.focusedDate.substring(0, 10)
        //             : calendar.focusedDate;
        //     if (currentFocus && currentFocus.includes("/")) {
        //         focusDateStr = currentFocus.replace(/^\d{4}/, newYear);
        //     } else {
        //         focusDateStr = `${newYear}/01/01`;
        //     }
        // } else {
        //     focusDateStr = `${newYear}/01/01`;
        // }

        // calendar.focusedDate = focusDateStr;
        // await tick();

        if (disabled || !calendar) return;

        if (calendar.focusedDate === undefined) {
            calendar.focusedDate = nowISOString.replace(
                currentYearString,
                e.currentTarget!.value,
            );
        } else {
            const focusedYear = calendar.focusedDate.substring(0, 4);
            calendar.focusedDate = calendar.focusedDate
                .substring(0, 10)
                .replace(focusedYear, e.currentTarget.value);
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
            aria-label="Select year"
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
            e.stopPropagation(); /* Prevent doc click closing */
        }}
        {onblur}
    >
        <div
            tabindex={disabled ? -1 : 0}
            class="input input-bordered w-full cursor-pointer caret-transparent flex items-center {inputClass}"
            class:input-disabled={disabled}
            class:input-error={inputClass.includes("input-error")}
            bind:this={callyContainerDiv}
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
                bind:this={hiddenDateInput}
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
        style="position: absolute; top: -9999px; left: -9999px;"
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
        background-color: var(--color-secondary);
    }
    /* General Component Styles */
    .input {
        min-height: 2.5rem;
        height: auto;
        padding-left: 0.75rem;
        padding-right: 0.75rem;
        display: flex;
        align-items: center;
    }
    .input:hover:not(.input-disabled),
    .input:focus:not(.input-disabled),
    .input:focus-within:not(.input-disabled) {
        border-color: var(--fallback-a, oklch(var(--a) / 1));
        outline: 2px solid transparent;
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
    .datepicker-portal {
        position: absolute;
        z-index: 10001 !important;
    }
    .dropdown-content {
        z-index: 10002 !important;
        display: block; /* Keep block for portal */
    }
    summary::marker,
    summary::-webkit-details-marker {
        display: none;
        content: "";
    }
</style>
