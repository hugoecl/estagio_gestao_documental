<script lang="ts">
    import { onMount, tick } from "svelte";
    import {
        getMyRemainingVacationDays,
        getMyVacationRequests,
        submitVacationRequest,
        getSharedCalendarVacations,
        cancelVacationRequest,
        requestVacationCancellation,
    } from "@api/vacation-api";
    import { getCalendarEvents } from "@api/calendar-api";
    import type {
        RemainingVacationDaysResponse,
        VacationRequest,
        CreateVacationRequestPayload,
        VacationRequestDisplay,
    } from "@lib/types/vacation";
    import { VacationRequestStatus } from "@lib/types/vacation";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";
    import { countWorkingDays } from "@utils/working-days";

    // --- State ---
    let remainingDaysInfo = $state<RemainingVacationDaysResponse | null>(null);
    let myRequests = $state<VacationRequestDisplay[]>([]); // Store processed requests
    let colleagueVacations = $state<
        Array<{ start_date: string; end_date: string; status: string }>
    >([]);
    let calendarEvents = $state<
        Array<{ start_date: string; end_date: string; title: string }>
    >([]);
    let isLoadingDays = $state(true);
    let isLoadingRequests = $state(true);
    let isLoadingShared = $state(false); // New loading state for shared data
    let error = $state<string | null>(null);
    let isCancelling = $state(false);
    let cancelRequestId = $state<number | null>(null);
    let cancellationRequestId = $state<number | null>(null);

    // --- Custom Calendar State & Logic ---
    interface CalendarDay {
        dayOfMonth: number;
        date: Date;
        isCurrentMonth: boolean;
        isToday: boolean;
        status: string | null;
        tooltip: string | null;
        isSelected: boolean;
        isRangeStart: boolean;
        isRangeEnd: boolean;
    }

    interface CalendarMonth {
        monthName: string;
        monthIndex: number; // 0-11
        year: number;
        weeks: CalendarDay[][];
    }

    let currentYear = $state(new Date().getFullYear());
    let previousYear = $state(currentYear); // Track previous year to detect changes
    let realCurrentYear = $derived(new Date().getFullYear()); // Track actual current year
    let baseCalendarStructure = $state<CalendarMonth[]>([]);
    let displayedCalendarData = $state<CalendarMonth[]>([]);

    // Date selection state
    let selectionStartDate = $state<Date | null>(null);
    let selectionEndDate = $state<Date | null>(null);
    let hoveredDate = $state<Date | null>(null);
    let selectedDaysCount = $state(0);
    let projectedRemainingDays = $state<number | null>(null);

    const monthNames = $state([
        // Made reactive for consistency, though not strictly necessary
        "Janeiro",
        "Fevereiro",
        "Março",
        "Abril",
        "Maio",
        "Junho",
        "Julho",
        "Agosto",
        "Setembro",
        "Outubro",
        "Novembro",
        "Dezembro",
    ]);
    const dayNames = $state(["Seg", "Ter", "Qua", "Qui", "Sex", "Sáb", "Dom"]);

    // Modal state for new request
    let requestModalRef: HTMLDialogElement | null = $state(null);
    let newRequestStartDate = $state<string>(""); // YYYY-MM-DD
    let newRequestEndDate = $state<string>(""); // YYYY-MM-DD
    let newRequestNotes = $state<string>("");
    let isSubmittingRequest = $state(false);
    let newRequestErrors = $state<Record<string, string>>({});

    function getDaysInMonth(year: number, month: number): number {
        // month is 0-indexed
        return new Date(year, month + 1, 0).getDate();
    }

    function getFirstDayOfMonth(year: number, month: number): number {
        // month is 0-indexed
        const day = new Date(year, month, 1).getDay(); // Sunday: 0, Monday: 1, ..., Saturday: 6
        return day === 0 ? 6 : day - 1; // Adjust to Monday: 0, ..., Sunday: 6
    }

    // estagio_gestao_documental/frontend/src/components/vacations/VacationCalendar.svelte
    // Replace your existing getDayClasses function with this:

    function getDayClasses(day: CalendarDay): string {
        const base =
            "p-0.5 h-7 w-full flex items-center justify-center text-xs border border-transparent focus:outline-none focus:ring-1 focus:ring-primary";
        let styleClasses = ""; // Accumulate specific styles here
        let roundingClasses = "rounded"; // Default to fully rounded
        let hoverClasses = "hover:bg-base-300"; // Default hover

        // Check if this day is in the current year (to disable selection)
        const isCurrentYear = currentYear === realCurrentYear;
        
        if (!day.isCurrentMonth) {
            return `${base} opacity-40 cursor-not-allowed rounded`;
        }

        // If not in current year, make all days non-interactive
        if (!isCurrentYear) {
            styleClasses += " opacity-70 cursor-not-allowed";
            hoverClasses = "";
        }
        
        styleClasses += " font-semibold";

        const isPreviewing =
            selectionStartDate &&
            !selectionEndDate &&
            hoveredDate &&
            hoveredDate >= selectionStartDate;
        let isPartOfHoverPreview = false;
        if (
            isPreviewing &&
            hoveredDate &&
            selectionStartDate &&
            day.date >= selectionStartDate &&
            day.date <= hoveredDate
        ) {
            isPartOfHoverPreview = true;
        }

        let hasStatusStyle = false;

        // 1. Apply vacation status styles (highest priority for background/text)
        if (day.status === "user_approved") {
            styleClasses += " bg-success text-success-content";
            hasStatusStyle = true;
        } else if (day.status === "user_pending") {
            styleClasses += " bg-warning text-warning-content"; // THIS IS FOR PENDING
            hasStatusStyle = true;
        } else if (day.status === "colleague_approved") {
            styleClasses +=
                " bg-neutral/60 text-neutral-content opacity-80 cursor-not-allowed";
            hoverClasses = ""; // No hover on colleague days
            hasStatusStyle = true;
        } else if (day.status === "colleague_pending") {
            styleClasses +=
                " bg-neutral/40 text-neutral-content opacity-70 cursor-not-allowed";
            hoverClasses = ""; // No hover on colleague pending days
            hasStatusStyle = true;
        } else if (day.status === "holiday") {
            styleClasses += " bg-error/20 text-error border border-error/50";
            hasStatusStyle = true;
        }

        // 2. Apply selection and hover preview styles
        // These will primarily affect days *without* an overriding vacation status,
        // or could add borders/text styles to days that already have a status background.
        if (day.isSelected) {
            if (selectionEndDate) {
                // --- Confirmed Selection ---
                if (!hasStatusStyle) {
                    // Only set background if no status style already did
                    // styleClasses += " bg-primary text-primary-content";
                } else {
                    // If has status, maybe just change text or add border to indicate selection
                    styleClasses += " text-primary-content"; // Ensure text contrasts with status bg
                }
                if (day.isRangeStart && day.isRangeEnd) {
                    styleClasses += " bg-accent";
                    roundingClasses = "rounded";
                } else if (day.isRangeStart) {
                    styleClasses += " bg-accent";
                    roundingClasses = "rounded-l rounded-r-none";
                } else if (day.isRangeEnd) {
                    styleClasses += " bg-accent";
                    roundingClasses = "rounded-r rounded-l-none";
                } else {
                    styleClasses += " bg-secondary";
                    roundingClasses = "rounded-none";
                }
                hoverClasses = ""; // No default hover on selected items
            } else if (isPartOfHoverPreview) {
                // --- Hover Preview ---
                if (!hasStatusStyle) {
                    // Using your preferred hover colors (assumed to be info-based)
                    styleClasses += " text-info-content";
                    if (
                        day.date.getTime() === selectionStartDate!.getTime() &&
                        day.date.getTime() === hoveredDate!.getTime()
                    ) {
                        styleClasses += " bg-info";
                        roundingClasses = "rounded";
                    } else if (
                        day.date.getTime() === selectionStartDate!.getTime()
                    ) {
                        styleClasses += " bg-info";
                        roundingClasses = "rounded-l rounded-r-none";
                    } else if (day.date.getTime() === hoveredDate!.getTime()) {
                        styleClasses += " bg-accent";
                        roundingClasses = "rounded-r rounded-l-none";
                    } else {
                        styleClasses += " bg-neutral/40 text-neutral-content";
                        roundingClasses = "rounded-none";
                    }
                }
                hoverClasses = "";
            } else if (
                selectionStartDate &&
                day.date.getTime() === selectionStartDate.getTime()
            ) {
                // Only start date selected (anchor point)
                if (!hasStatusStyle) {
                    styleClasses += " bg-primary text-primary-content";
                } else {
                    styleClasses += " text-primary-content";
                }
                roundingClasses = "rounded";
                hoverClasses = "";
            }
        }

        // 3. Today's date styling (if not styled by status or active selection)
        if (!hasStatusStyle && !day.isSelected && day.isToday) {
            styleClasses += " !border-primary text-primary"; // Ensure it's rounded by default
            if (hoverClasses === "hover:bg-base-300") {
                hoverClasses = "hover:bg-primary/10";
            }
        }

        // Construct final classes
        let finalClasses = `${base} ${styleClasses} ${roundingClasses} ${hoverClasses}`;

        if (day.status === "colleague_approved" || day.status === "colleague_pending" || !day.isCurrentMonth) {
            if (!finalClasses.includes(" cursor-not-allowed"))
                finalClasses += " cursor-not-allowed";
        }

        return finalClasses.trim().replace(/\s+/g, " ");
    }

    function generateBaseCalendarStructure(year: number): CalendarMonth[] {
        const newBaseData: CalendarMonth[] = [];
        const today = new Date();
        today.setUTCHours(0, 0, 0, 0);

        for (let monthIndex = 0; monthIndex < 12; monthIndex++) {
            const monthName = monthNames[monthIndex]; // Access reactive state
            const daysInCurrentMonth = getDaysInMonth(year, monthIndex);
            const firstDayOfWeek = getFirstDayOfMonth(year, monthIndex);

            const currentMonthDays: CalendarDay[] = [];

            const prevMonthIndex = monthIndex === 0 ? 11 : monthIndex - 1;
            const prevMonthYear = monthIndex === 0 ? year - 1 : year;
            const daysInPrevMonth = getDaysInMonth(
                prevMonthYear,
                prevMonthIndex,
            );

            for (let i = 0; i < firstDayOfWeek; i++) {
                const day = daysInPrevMonth - firstDayOfWeek + 1 + i;
                const date = new Date(
                    Date.UTC(prevMonthYear, prevMonthIndex, day),
                );
                currentMonthDays.push({
                    dayOfMonth: day,
                    date,
                    isCurrentMonth: false,
                    isToday: date.getTime() === today.getTime(),
                    status: null,
                    tooltip: null,
                    isSelected: false,
                    isRangeStart: false,
                    isRangeEnd: false,
                });
            }

            for (let day = 1; day <= daysInCurrentMonth; day++) {
                const date = new Date(Date.UTC(year, monthIndex, day));
                currentMonthDays.push({
                    dayOfMonth: day,
                    date,
                    isCurrentMonth: true,
                    isToday: date.getTime() === today.getTime(),
                    status: null,
                    tooltip: null,
                    isSelected: false,
                    isRangeStart: false,
                    isRangeEnd: false,
                });
            }

            const totalCellsRequired = currentMonthDays.length > 35 ? 42 : 35;
            let finalCellsToFill = totalCellsRequired - currentMonthDays.length;

            const nextMonthIndex = monthIndex === 11 ? 0 : monthIndex + 1;
            const nextMonthYear = monthIndex === 11 ? year + 1 : year;

            for (let i = 1; i <= finalCellsToFill; i++) {
                if (currentMonthDays.length >= totalCellsRequired) break;
                const date = new Date(
                    Date.UTC(nextMonthYear, nextMonthIndex, i),
                );
                currentMonthDays.push({
                    dayOfMonth: i,
                    date,
                    isCurrentMonth: false,
                    isToday: date.getTime() === today.getTime(),
                    status: null,
                    tooltip: null,
                    isSelected: false,
                    isRangeStart: false,
                    isRangeEnd: false,
                });
            }

            while (
                currentMonthDays.length < totalCellsRequired &&
                currentMonthDays.length > 0
            ) {
                const lastDayObj =
                    currentMonthDays[currentMonthDays.length - 1];
                const nextDayDate = new Date(lastDayObj.date);
                nextDayDate.setUTCDate(lastDayObj.date.getUTCDate() + 1);
                currentMonthDays.push({
                    dayOfMonth: nextDayDate.getUTCDate(),
                    date: nextDayDate,
                    isCurrentMonth: false,
                    isToday: nextDayDate.getTime() === today.getTime(),
                    status: null,
                    tooltip: null,
                    isSelected: false,
                    isRangeStart: false,
                    isRangeEnd: false,
                });
            }
            if (currentMonthDays.length > totalCellsRequired) {
                currentMonthDays.splice(totalCellsRequired);
            }

            const weeks: CalendarDay[][] = [];
            for (let i = 0; i < currentMonthDays.length; i += 7) {
                weeks.push(currentMonthDays.slice(i, i + 7));
            }
            newBaseData.push({ monthName, monthIndex, year, weeks });
        }
        return newBaseData;
    }

    // estagio_gestao_documental/frontend/src/components/vacations/VacationCalendar.svelte
    // Replace your existing applyVisualsToCalendar function (around L341-L477) with this:

    // estagio_gestao_documental/frontend/src/components/vacations/VacationCalendar.svelte
    // Replace your existing applyVisualsToCalendar function (around L340-L492)

    function applyVisualsToCalendar(
        baseStructureInput: CalendarMonth[],
        userRequests: VacationRequestDisplay[],
        colleagueDateRanges: Array<{ start_date: string; end_date: string; status: string }>,
        holidayDateRanges: Array<{ start_date: string; end_date: string; title: string }>,
        currentSelectionStart: Date | null,
        currentSelectionEnd: Date | null,
        currentHoveredDate: Date | null,
    ): CalendarMonth[] {
        if (!baseStructureInput || !baseStructureInput.length) {
            // console.log('ApplyVisuals: baseStructureInput is empty or null, returning empty.');
            return [];
        }

        const newCalendarStructure = JSON.parse(
            JSON.stringify(baseStructureInput),
        ) as CalendarMonth[];

        const colleagueBookedPeriods = colleagueDateRanges.map((range) => ({
            start: new Date(range.start_date + "T00:00:00Z").getTime(),
            end: new Date(range.end_date + "T00:00:00Z").getTime(),
            status: range.status,
        }));

        const holidayPeriods = holidayDateRanges.map((range) => ({
            start: new Date(range.start_date + "T00:00:00Z").getTime(),
            end: new Date(range.end_date + "T00:00:00Z").getTime(),
            title: range.title,
        }));

        newCalendarStructure.forEach((month) => {
            month.weeks.forEach((week) => {
                week.forEach((day) => {
                    day.date = new Date(day.date); // Ensure it's a Date object
                    const dayTime = day.date.getTime();

                    // Reset visual properties
                    day.status = null;
                    day.tooltip = null;
                    day.isSelected = false;
                    day.isRangeStart = false;
                    day.isRangeEnd = false;

                    // --- Debugging Loop for userRequests ---
                    // console.log(`Processing Day: ${day.date.toISOString().slice(0,10)}`);

                    // --- 1. Apply Colleague's Approved and Pending Vacations ---
                    for (const colleaguePeriod of colleagueBookedPeriods) {
                        if (
                            dayTime >= colleaguePeriod.start &&
                            dayTime <= colleaguePeriod.end
                        ) {
                            if (colleaguePeriod.status === "APPROVED" || colleaguePeriod.status === "CANCELLATION_REQUESTED") {
                                day.status = "colleague_approved";
                                day.tooltip = colleaguePeriod.status === "CANCELLATION_REQUESTED" ? "Férias Colega (Cancelamento pedido)" : "Férias Colega (Aprovadas)";
                            } else if (colleaguePeriod.status === "PENDING") {
                                day.status = "colleague_pending";
                                day.tooltip = "Férias Colega (Pendentes)";
                            }
                            // console.log(` -> Status set to ${day.status}`);
                            break;
                        }
                    }

                    // --- 2. Apply User's Actual Vacation Request Statuses ---
                    if (day.status !== "colleague_approved") {
                        for (const req of userRequests) {
                            // console.log(`  Comparing with User Request ID: ${req.id}, Status: ${req.status}, Start: ${req.start_date}, End: ${req.end_date}`);
                            const reqStartTime = new Date(
                                req.start_date + "T00:00:00Z",
                            ).getTime();
                            const reqEndTime = new Date(
                                req.end_date + "T00:00:00Z",
                            ).getTime();

                            // console.log(`    DayTime: ${dayTime}, ReqStartTime: ${reqStartTime}, ReqEndTime: ${reqEndTime}`);

                            if (
                                dayTime >= reqStartTime &&
                                dayTime <= reqEndTime
                            ) {
                                // console.log(`    -> DAY IS WITHIN REQUEST RANGE.`);
                                if (
                                    req.status === VacationRequestStatus.Pending
                                ) {
                                    day.status = "user_pending";
                                    day.tooltip = `Meu Pedido (Pendente): ${req.startDateDisplay} - ${req.endDateDisplay}`;
                                } else if (
                                    req.status ===
                                    VacationRequestStatus.Approved
                                ) {
                                    day.status = "user_approved";
                                    day.tooltip = `Meu Pedido (Aprovado): ${req.startDateDisplay} - ${req.endDateDisplay}`;
                                } else if (
                                    req.status ===
                                    VacationRequestStatus.Rejected
                                ) {
                                    // Optionally handle rejected for tooltip or a very subtle style, but typically not a strong background
                                    // day.status = 'user_rejected';
                                    // day.tooltip = `Meu Pedido (Rejeitado): ${req.startDateDisplay} - ${req.endDateDisplay}`;
                                    // console.log(`LOG: Day ${day.date.toISOString().slice(0,10)} status: user_rejected (Raw req.status: ${req.status})`);
                                }
                                break;
                            }
                        }
                    }

                    // --- 2b. Apply Fixed Holidays (when no vacation status, or append to tooltip) ---
                    for (const period of holidayPeriods) {
                        if (dayTime >= period.start && dayTime <= period.end) {
                            if (day.status === null) {
                                day.status = "holiday";
                                day.tooltip = `Feriado: ${period.title}`;
                            } else if (day.tooltip) {
                                day.tooltip += ` | Feriado: ${period.title}`;
                            }
                            break;
                        }
                    }

                    // --- 3. Apply Selection / Hover Preview Visuals ---
                    if (day.status !== "colleague_approved" && day.status !== "colleague_pending") {
                        const isPreviewing =
                            currentSelectionStart &&
                            !currentSelectionEnd &&
                            currentHoveredDate &&
                            currentHoveredDate.getTime() >=
                                currentSelectionStart.getTime();
                        let effectiveRangeEndForSelection = currentSelectionEnd;
                        if (isPreviewing && currentHoveredDate) {
                            effectiveRangeEndForSelection = currentHoveredDate;
                        }

                        if (
                            currentSelectionStart &&
                            effectiveRangeEndForSelection
                        ) {
                            const selStartTime =
                                currentSelectionStart.getTime();
                            const selEndTime =
                                effectiveRangeEndForSelection.getTime();

                            if (
                                dayTime >= selStartTime &&
                                dayTime <= selEndTime
                            ) {
                                day.isSelected = true;
                                if (dayTime === selStartTime)
                                    day.isRangeStart = true;

                                if (
                                    currentSelectionEnd &&
                                    dayTime === currentSelectionEnd.getTime()
                                ) {
                                    day.isRangeEnd = true;
                                } else if (
                                    isPreviewing &&
                                    currentHoveredDate &&
                                    dayTime === currentHoveredDate.getTime()
                                ) {
                                    day.isRangeEnd = true;
                                }
                            }
                        } else if (currentSelectionStart) {
                            if (dayTime === currentSelectionStart.getTime()) {
                                day.isSelected = true;
                                day.isRangeStart = true;
                                day.isRangeEnd = true;
                            }
                        }
                    }
                });
            });
        });
        return newCalendarStructure;
    }

    // --- Fetch Initial Data ---
    async function fetchAllCalendarData(year: number) {
        isLoadingDays = true;
        isLoadingRequests = true;
        isLoadingShared = true; // Set loading for shared data
        error = null;
        
        try {
            // Always fetch data for the specific year requested
            const [daysData, requestsData, sharedColleagueData, eventsData] =
                await Promise.all([
                    getMyRemainingVacationDays(year),
                    getMyVacationRequests(year),
                    getSharedCalendarVacations(year),
                    getCalendarEvents(year),
                ]);

            // Process data for the specific year
            if (daysData) {
                remainingDaysInfo = daysData;
            } else {
                // If no data returned for this year, show zeros
                remainingDaysInfo = {
                    total_allocated_days: 0,
                    approved_days_taken: 0,
                    pending_days_requested: 0,
                    remaining_days: 0
                };
            }

            // Process vacation requests for the specific year
            myRequests = processVacationRequestsForDisplay(
                requestsData || [],
                year,
                eventsData || []
            );
            colleagueVacations = sharedColleagueData || [];
            calendarEvents = eventsData || [];

            // Handle year-specific UI logic in the template
        } catch (e: any) {
            console.error("Error fetching vacation data for year:", year, e);
            error = `Erro ao carregar dados: ${e.message}`;
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoadingDays = false;
            isLoadingRequests = false;
            isLoadingShared = false;
        }
    }

    onMount(async () => {
        // Load data for initial year (current year)
        await fetchAllCalendarData(currentYear);
    });

    // Add effect to reload data when year changes
    $effect(() => {
        const year = currentYear;
        
        // Don't have any other effects in here - this should ONLY be triggered
        // when the user explicitly changes the year via the UI controls
        if (year !== previousYear) {
            // Only reload data when the year changes
            fetchAllCalendarData(year);
            
            // Clear any date selection when changing years
            if (selectionStartDate || selectionEndDate) {
                clearSelection();
            }
            
            // Update previous year
            previousYear = year;
        }
    });

    // Effect to update displayed calendar when year, myRequests, colleagueVacations, selection, or hover changes
    $effect(() => {
        const year = currentYear;
        const _myRequests = myRequests;
        const _colleagueVacations = colleagueVacations;
        const _calendarEvents = calendarEvents;
        const _selectionStartDate = selectionStartDate;
        const _selectionEndDate = selectionEndDate;
        const _hoveredDate = hoveredDate;


        let currentBase = baseCalendarStructure;
        if (
            !currentBase.length ||
            (currentBase[0] && currentBase[0].year !== year)
        ) {
            currentBase = generateBaseCalendarStructure(year);
            baseCalendarStructure = currentBase;
        }

        if (currentBase.length > 0) {
            displayedCalendarData = applyVisualsToCalendar(
                currentBase,
                _myRequests,
                _colleagueVacations,
                _calendarEvents,
                _selectionStartDate,
                _selectionEndDate,
                _hoveredDate,
            );
            
            // Update selected days count (working days only) when hovering changes
            if (_selectionStartDate && !_selectionEndDate && _hoveredDate && _hoveredDate >= _selectionStartDate) {
                selectedDaysCount = countWorkingDays(_selectionStartDate, _hoveredDate, _calendarEvents);
                updateProjectedRemainingDays();
            }
        } else {
            displayedCalendarData = [];
        }
    });

    function handleDayClick(day: CalendarDay) {
        if (!day.isCurrentMonth) return;
        
        // Prevent selecting dates if not in the current year
        if (currentYear !== realCurrentYear) {
            showAlert(
                `Não é possível selecionar datas fora do ano atual (${realCurrentYear}).`,
                AlertType.INFO,
                AlertPosition.TOP
            );
            return;
        }
        
        const clickedDate: Date = day.date;

        if (day.status && day.status === "colleague_approved") {
            showAlert(
                "Este dia não está disponível pois coincide com as férias de um colega.",
                AlertType.WARNING,
                AlertPosition.TOP,
            );
            return;
        }
        if (day.status && day.status === "colleague_pending") {
            showAlert(
                "Este dia não está disponível pois coincide com as férias pendentes de um colega.",
                AlertType.WARNING,
                AlertPosition.TOP,
            );
            return;
        }
        if (day.status && day.status === "user_approved") {
            showAlert(
                "Já tem férias aprovadas para este dia.",
                AlertType.INFO,
                AlertPosition.TOP,
            );
            return;
        }

        if (!selectionStartDate) {
            selectionStartDate = clickedDate;
            selectionEndDate = null;
            selectedDaysCount = countWorkingDays(clickedDate, clickedDate, calendarEvents);
            updateProjectedRemainingDays();
        } else if (!selectionEndDate) {
            if (clickedDate.getTime() === selectionStartDate.getTime()) {
                selectionEndDate = clickedDate;
                selectedDaysCount = countWorkingDays(clickedDate, clickedDate, calendarEvents);
            } else if (clickedDate < selectionStartDate) {
                selectionEndDate = selectionStartDate;
                selectionStartDate = clickedDate;
                selectedDaysCount = countWorkingDays(selectionStartDate, selectionEndDate, calendarEvents);
            } else {
                selectionEndDate = clickedDate;
                selectedDaysCount = countWorkingDays(selectionStartDate, selectionEndDate, calendarEvents);
            }
            updateProjectedRemainingDays();
        } else {
            selectionStartDate = clickedDate;
            selectionEndDate = null;
            selectedDaysCount = countWorkingDays(clickedDate, clickedDate, calendarEvents);
            updateProjectedRemainingDays();
        }
        
    }

    function updateProjectedRemainingDays() {
        // Only calculate projection for current year
        if (currentYear !== realCurrentYear) {
            projectedRemainingDays = null;
            return;
        }
        
        if (remainingDaysInfo && selectedDaysCount > 0) {
            // Calculate remaining days directly from total, not from already calculated remaining_days
            projectedRemainingDays = Math.max(0, 
                (remainingDaysInfo.total_allocated_days || 0) - 
                (remainingDaysInfo.approved_days_taken || 0) - 
                (remainingDaysInfo.pending_days_requested || 0) - 
                selectedDaysCount);
        } else {
            projectedRemainingDays = null;
        }
    }

    function clearSelection() {
        selectionStartDate = null;
        selectionEndDate = null;
        hoveredDate = null;
        selectedDaysCount = 0;
        projectedRemainingDays = null;
    }

    function processVacationRequestsForDisplay(
        requests: VacationRequest[],
        year?: number,
        holidays: Array<{ start_date: string; end_date: string }> = []
    ): VacationRequestDisplay[] {
        // If a specific year is provided, filter requests for that year
        if (year !== undefined) {
            const yearStart = new Date(Date.UTC(year, 0, 1)); // Jan 1 of the year
            const yearEnd = new Date(Date.UTC(year, 11, 31)); // Dec 31 of the year
            
            requests = requests.filter(req => {
                const startDate = new Date(req.start_date + "T00:00:00Z");
                return startDate >= yearStart && startDate <= yearEnd;
            });
        }

        return requests.map((req) => {
            let statusEnum: VacationRequestStatus;
            switch (
                req.status.toUpperCase() // Convert API string status to enum
            ) {
                case "CANCELLATION_REQUESTED":
                    statusEnum = VacationRequestStatus.CancellationRequested;
                    break;
                case "CANCELLED":
                    statusEnum = VacationRequestStatus.Cancelled;
                    break;
                case "PENDING":
                    statusEnum = VacationRequestStatus.Pending;
                    break;
                case "APPROVED":
                    statusEnum = VacationRequestStatus.Approved;
                    break;
                case "REJECTED":
                    statusEnum = VacationRequestStatus.Rejected;
                    break;
                default:
                    console.warn(
                        `Unknown vacation status string: ${req.status}`,
                    );
                    statusEnum = VacationRequestStatus.Pending; // Or some default/error state
            }
            const start = new Date(req.start_date + "T00:00:00Z");
            const end = new Date(req.end_date + "T00:00:00Z");
            const requestedAt = new Date(req.requested_at);
            const actionedAt = req.actioned_at
                ? new Date(req.actioned_at)
                : null;

            let duration = 0;
            if (
                !isNaN(start.getTime()) &&
                !isNaN(end.getTime()) &&
                end >= start
            ) {
                duration = countWorkingDays(start, end, holidays);
            }

            return {
                ...req,
                startDateDisplay: !isNaN(start.getTime())
                    ? start.toLocaleDateString("pt-PT", { timeZone: "UTC" })
                    : "Inválida",
                endDateDisplay: !isNaN(end.getTime())
                    ? end.toLocaleDateString("pt-PT", { timeZone: "UTC" })
                    : "Inválida",
                requestedAtDisplay: requestedAt.toLocaleString("pt-PT", {
                    timeZone: "Europe/Lisbon",
                }),
                actionedAtDisplay: actionedAt
                    ? actionedAt.toLocaleString("pt-PT", {
                          timeZone: "Europe/Lisbon",
                      })
                    : undefined,
                duration,
                status: statusEnum,
            };
        });
    }

    // --- New Request Modal ---\
    function openRequestModal() {
        // Only allow requests for the current year
        if (currentYear !== realCurrentYear) {
            showAlert(
                `Os pedidos de férias só podem ser feitos para o ano atual (${realCurrentYear}).`,
                AlertType.INFO,
                AlertPosition.TOP
            );
            return;
        }

        if (selectionStartDate && selectionEndDate) {
            const yyyyMMDD = (date: Date) => date.toISOString().split("T")[0];
            newRequestStartDate = yyyyMMDD(selectionStartDate);
            newRequestEndDate = yyyyMMDD(selectionEndDate);
        } else {
            newRequestStartDate = "";
            newRequestEndDate = "";
            showAlert(
                "Por favor, selecione um intervalo de datas no calendário primeiro.",
                AlertType.INFO,
                AlertPosition.TOP,
            );
            return;
        }
        newRequestNotes = "";
        newRequestErrors = {};
        requestModalRef?.showModal();
    }

    function validateNewRequestForm(): boolean {
        newRequestErrors = {};
        if (!newRequestStartDate) {
            newRequestErrors.startDate = "Data de início é obrigatória.";
        }
        if (!newRequestEndDate) {
            newRequestErrors.endDate = "Data de fim é obrigatória.";
        }
        if (newRequestStartDate && newRequestEndDate) {
            const start = new Date(newRequestStartDate);
            const end = new Date(newRequestEndDate);
            if (isNaN(start.getTime()) || isNaN(end.getTime())) {
                newRequestErrors.general = "Datas inválidas.";
            } else if (end < start) {
                newRequestErrors.endDate =
                    "Data de fim não pode ser anterior à data de início.";
            }
            const requestedDuration = countWorkingDays(
                start,
                end,
                calendarEvents
            );
            if (
                remainingDaysInfo &&
                requestedDuration > remainingDaysInfo.remaining_days
            ) {
                newRequestErrors.general = `Não tem dias de férias suficientes. Restantes: ${remainingDaysInfo.remaining_days}, Pedidos: ${requestedDuration}.`;
            }
        }
        return Object.keys(newRequestErrors).length === 0;
    }

    // Function to handle cancellation of a pending request
    async function handleCancelRequest(requestId: number) {
        if (isCancelling) return; // Prevent multiple clicks
        
        isCancelling = true;
        cancelRequestId = requestId;
        
        try {
            const result = await cancelVacationRequest(requestId);
            
            if (result.success) {
                showAlert(
                    result.message || "Pedido de férias cancelado com sucesso.",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                
                // Refresh data with current year since we're modifying current year data
                await fetchAllCalendarData(currentYear);
            } else {
                showAlert(
                    result.message || "Falha ao cancelar o pedido de férias.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
        } catch (e: any) {
            console.error(`Error cancelling vacation request ${requestId}:`, e);
            showAlert(
                `Erro ao cancelar o pedido: ${e.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isCancelling = false;
            cancelRequestId = null;
        }
    }

    async function handleRequestCancellation(requestId: number) {
        if (cancellationRequestId) return;
        cancellationRequestId = requestId;
        try {
            const result = await requestVacationCancellation(requestId);
            if (result.success) {
                showAlert(result.message, AlertType.SUCCESS, AlertPosition.TOP);
                await fetchAllCalendarData(currentYear);
            } else {
                showAlert(result.message, AlertType.ERROR, AlertPosition.TOP);
            }
        } catch (e: any) {
            showAlert(`Erro: ${e.message}`, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            cancellationRequestId = null;
        }
    }
    
    async function handleNewRequestSubmit(e: Event) {
        e.preventDefault();
        if (!validateNewRequestForm()) {
            showAlert(
                "Por favor, corrija os erros no formulário.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
            return;
        }
        
        // Check for colleague vacation conflicts (both pending and approved)
        const requestStartDate = new Date(newRequestStartDate + "T00:00:00Z").getTime();
        const requestEndDate = new Date(newRequestEndDate + "T00:00:00Z").getTime();
        
        for (const colleagueVacation of colleagueVacations) {
            const colleagueStart = new Date(colleagueVacation.start_date + "T00:00:00Z").getTime();
            const colleagueEnd = new Date(colleagueVacation.end_date + "T00:00:00Z").getTime();
            
            // Check for overlap: (StartA <= EndB) and (EndA >= StartB)
            if (requestStartDate <= colleagueEnd && requestEndDate >= colleagueStart) {
                const status = colleagueVacation.status === "PENDING" ? "pendentes" : "aprovadas";
                newRequestErrors.general = `Este período coincide com férias ${status} de um colega. Por favor, escolha outras datas.`;
                showAlert(
                    newRequestErrors.general,
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
                return;
            }
        }
        
        isSubmittingRequest = true;
        const payload: CreateVacationRequestPayload = {
            start_date: newRequestStartDate,
            end_date: newRequestEndDate,
            notes: newRequestNotes || null,
        };

        try {
            const result = await submitVacationRequest(payload);
            if (result.success) {
                showAlert(
                    "Pedido de férias submetido com sucesso!",
                    AlertType.SUCCESS,
                    AlertPosition.TOP,
                );
                requestModalRef?.close();
                clearSelection();

                // Refresh data for the current year (not necessarily the displayed year)
                await fetchAllCalendarData(currentYear);
            } else {
                showAlert(
                    result.message || "Falha ao submeter pedido de férias.",
                    AlertType.ERROR,
                    AlertPosition.TOP,
                );
            }
        } catch (err: any) {
            showAlert(
                `Erro ao submeter pedido: ${err.message}`,
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isSubmittingRequest = false;
            isLoadingRequests = false;
            isLoadingDays = false;
        }
    }
</script>

<div class="space-y-6">
    <div
        class="flex flex-col sm:flex-row justify-between items-center gap-4 pb-4 border-b border-base-content/10"
    >
        <h1 class="text-2xl font-bold">Os Meus Pedidos de Férias</h1>
        <div class="flex gap-2 items-center">
            {#if selectionStartDate}
                <button
                    class="btn btn-outline btn-sm"
                    onclick={clearSelection}
                    title="Limpar seleção de datas"
                >
                    <i class="fa-solid fa-times mr-1"></i> Limpar Datas
                </button>
            {/if}
            <button
                class="btn btn-primary"
                onclick={openRequestModal}
                disabled={!selectionStartDate || !selectionEndDate || selectedDaysCount <= 0 || currentYear !== new Date().getFullYear()}
            >
                <i class="fa-solid fa-calendar-plus mr-2"></i>
                Pedir Férias
            </button>
        </div>
    </div>

    {#if error}
        <div class="alert alert-error">{error}</div>
    {/if}

    <!-- Vacation Days Summary -->
    <div class="card bg-base-200 shadow">
        <div class="card-body p-4">
            <h2 class="card-title text-base">
                Resumo de Dias de Férias ({currentYear})
            </h2>
            {#if isLoadingDays}
                <div class="flex justify-center py-3">
                    <span class="loading loading-dots loading-md"></span>
                </div>
            {:else}
                <!-- Display year-specific message -->
                {#if currentYear > realCurrentYear}
                    <div class="alert alert-info mt-2">
                        <i class="fa-solid fa-circle-info"></i>
                        <span>Os dias de férias para {currentYear} ainda não foram atribuídos. Serão disponibilizados automaticamente quando o ano começar.</span>
                    </div>
                {:else if currentYear < new Date().getFullYear()}
                    <div class="alert alert-info mt-2">
                        <i class="fa-solid fa-circle-info"></i>
                        <span>A ver dados históricos de {currentYear}. {remainingDaysInfo?.total_allocated_days === 0 ? 'Não existem registos para este ano.' : ''} Os pedidos de férias só podem ser feitos para o ano atual ({realCurrentYear}).</span>
                    </div>
                {/if}
                
                <!-- Always show the summary data -->
                <div class="flex w-full justify-between gap-2 text-sm mt-2 {currentYear !== new Date().getFullYear() ? 'opacity-70' : ''}" role="list">
                    <div class="text-center p-2 bg-base-100 rounded w-full">
                        <div class="font-semibold relative group cursor-help">
                            Total Dias Férias
                            <div class="absolute z-10 hidden group-hover:block bg-base-300 p-2 rounded shadow-lg text-xs w-64 text-left mt-1">
                                <p>Número total de dias de férias atribuídos a si para {currentYear}.</p>
                            </div>
                        </div>
                        <div class="text-lg">
                            {remainingDaysInfo?.total_allocated_days || 0}
                        </div>
                    </div>
                    <div class="text-center p-2 bg-base-100 rounded w-full">
                        <div class="font-semibold relative group cursor-help">
                            Aprovados
                            <div class="absolute z-10 hidden group-hover:block bg-base-300 p-2 rounded shadow-lg text-xs w-64 text-left mt-1">
                                <p>Dias de férias que foram aprovados para {currentYear}.</p>
                            </div>
                        </div>
                        <div class="text-lg text-success">
                            {remainingDaysInfo?.approved_days_taken || 0}
                        </div>
                    </div>
                    <div class="text-center p-2 bg-base-100 rounded w-full">
                        <div class="font-semibold relative group cursor-help">
                            Pendentes
                            <div class="absolute z-10 hidden group-hover:block bg-base-300 p-2 rounded shadow-lg text-xs w-64 text-left mt-1">
                                <p>Dias de férias que foram solicitados mas ainda aguardam aprovação para {currentYear}.</p>
                            </div>
                        </div>
                        <div class="text-lg text-warning">
                            {remainingDaysInfo?.pending_days_requested || 0}
                        </div>
                    </div>
                    <div class="text-center p-2 bg-base-100 rounded w-full">
                        <div class="font-semibold relative group cursor-help">
                            Dias Disponíveis
                            <div class="absolute z-10 hidden group-hover:block bg-base-300 p-2 rounded shadow-lg text-xs w-64 text-left mt-1">
                                <p class="mb-1">Cálculo de dias disponíveis para {currentYear}:</p>
                                <p>Total ({remainingDaysInfo?.total_allocated_days || 0}) - Aprovados ({remainingDaysInfo?.approved_days_taken || 0}) - Pendentes ({remainingDaysInfo?.pending_days_requested || 0}){selectedDaysCount > 0 && currentYear === realCurrentYear ? ` - Selecionados (${selectedDaysCount})` : ''} = <strong>{projectedRemainingDays !== null && selectedDaysCount > 0 && currentYear === realCurrentYear ? projectedRemainingDays : (remainingDaysInfo?.remaining_days || 0)}</strong></p>
                            </div>
                        </div>
                        <div class="text-lg font-bold text-primary">
                            {projectedRemainingDays !== null && selectedDaysCount > 0 && currentYear === realCurrentYear ? 
                              `${projectedRemainingDays} (após seleção)` : 
                              (remainingDaysInfo?.remaining_days || 0)}
                        </div>
                    </div>
                </div>
            {/if}
        </div>
    </div>

    <!-- Calendar Display -->
    <div class="card bg-base-100 shadow mb-4">
     <div class="card-body">
      <div class="flex justify-between items-center mb-4">
       <h2 class="card-title text-base">
        Calendario Anual {currentYear}
       </h2>
                <div>
                    <button
                        class="btn btn-sm btn-ghost"
                        onclick={() => currentYear--}
                        title="Ano Anterior"
                        aria-label="Ano Anterior"
                    >
                        <i class="fa-solid fa-chevron-left"></i>
                    </button>
                    <button
                        class="btn btn-sm btn-ghost"
                        onclick={() => (currentYear = realCurrentYear)}
                        title="Ano Atual"
                        aria-label="Ano Atual"
                    >
                        <i class="fa-solid fa-calendar-day"></i>
                    </button>
                    <button
                        class="btn btn-sm btn-ghost"
                        onclick={() => currentYear++}
                        title="Próximo Ano"
                        aria-label="Próximo Ano"
                    >
                        <i class="fa-solid fa-chevron-right"></i>
                    </button>
                </div>
            </div>

            {#if displayedCalendarData.length === 0 && isLoadingDays}
                <div class="flex justify-center items-center p-10">
                    <span class="loading loading-lg loading-spinner"></span>
                </div>
            {:else if displayedCalendarData.length === 0 && !isLoadingDays}
                <p class="text-center text-base-content/70 py-5">
                    Calendário indisponível.
                </p>
            {:else}
                <div
                    class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3"
                    role="grid"
                    tabindex="0"
                    onmouseleave={() => hoveredDate = null}
                    aria-label="Calendar grid"
                >
                    {#each displayedCalendarData as month (month.year + "-" + month.monthIndex)}
                        <div
                            class="border border-base-content/20 rounded-md p-1.5 bg-base-200/30 shadow-sm min-w-[260px]"
                            onmouseleave={() => {
                                if (selectionStartDate && !selectionEndDate) {
                                    hoveredDate = null;
                                }
                            }}
                            role="presentation"
                        >
                            <h3
                                class="text-sm font-semibold text-center mb-1.5 text-primary"
                            >
                                {month.monthName}
                            </h3>
                            <div
                                class="grid grid-cols-7 gap-px text-xs text-center"
                            >
                                {#each dayNames as dayName}
                                    <div
                                        class="font-medium text-base-content/70 pb-1"
                                    >
                                        {dayName}
                                    </div>
                                {/each}
                            </div>
                            {#each month.weeks as week, weekIndex (weekIndex)}
                                <div class="grid grid-cols-7 gap-px">
                                    {#each week as day, dayIndex (day.date.toISOString())}
                                        <button
                                            class={getDayClasses(day)}
                                            title={day.tooltip ||
                                                `${day.dayOfMonth}/${month.monthIndex + 1}/${month.year}${day.isToday ? " (Hoje)" : ""}`}
                                            disabled={!day.isCurrentMonth ||
                                                (!!day.status &&
                                                    (day.status ===
                                                        "user_approved" ||
                                                        day.status ===
                                                            "colleague_approved"))}
                                            onmouseenter={() => {
                                                if (
                                                    selectionStartDate &&
                                                    !selectionEndDate &&
                                                    day.isCurrentMonth &&
                                                    !day.status
                                                ) {
                                                    if (
                                                        day.date >=
                                                        selectionStartDate
                                                    ) {
                                                        hoveredDate = day.date;
                                                    }
                                                }
                                            }}
                                            onclick={() => handleDayClick(day)}
                                        >
                                            {day.dayOfMonth}
                                        </button>
                                    {/each}
                                </div>
                            {/each}
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    </div>

    <!-- List of My Requests -->
    <!-- Color Legend -->
    <div class="card bg-base-100 shadow mb-4">
     <div class="card-body p-4">
      <h2 class="card-title text-base">Legenda</h2>
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-2 text-sm mt-2">
       <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-success rounded"></div>
        <span>Suas férias aprovadas</span>
       </div>
       <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-warning rounded"></div>
        <span>Suas férias pendentes</span>
       </div>
       <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-neutral/60 rounded"></div>
        <span>Férias aprovadas de colegas</span>
       </div>
       <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-neutral/40 rounded"></div>
        <span>Férias pendentes de colegas</span>
       </div>
       <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-error/20 border border-error/50 rounded"></div>
        <span>Feriados</span>
       </div>
       <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-accent rounded"></div>
        <span>Dias selecionados {selectedDaysCount > 0 ? `(${selectedDaysCount} dias úteis)` : ''}</span>
       </div>
       <div class="flex items-center gap-2">
        <div class="w-4 h-4 bg-info rounded"></div>
        <span>Pré-visualização de seleção</span>
       </div>
      </div>
     </div>
    </div>

    <div class="card bg-base-100 shadow">
        <div class="card-body">
            <h2 class="card-title text-base">
                {currentYear < new Date().getFullYear() ? 'Histórico de' : ''} Meus Pedidos Registados {currentYear !== new Date().getFullYear() ? `(${currentYear})` : ''}
            </h2>
            {#if isLoadingRequests}
                <div class="flex justify-center py-5">
                    <span class="loading loading-spinner loading-lg"></span>
                </div>
            {:else if myRequests.length === 0}
                {#if currentYear > realCurrentYear}
                    <p class="text-center text-base-content/70 py-5">
                        Sem registos disponíveis para {currentYear}. Os pedidos de férias para anos futuros ainda não estão disponíveis.
                    </p>
                {:else if currentYear < new Date().getFullYear()}
                    <p class="text-center text-base-content/70 py-5">
                        Sem registos históricos de férias para {currentYear}.
                    </p>
                {:else}
                    <p class="text-center text-base-content/70 py-5">
                        Ainda não tem pedidos de férias registados para {currentYear}.
                    </p>
                {/if}
            {:else}
                <div class="overflow-x-auto">
                    <table class="table table-sm w-full">
                        <thead>
                            <tr>
                                <th>Início</th>
                                <th>Fim</th>
                                <th>Duração</th>
                                <th>Estado</th>
                                <th>Notas</th>
                                <th>Submetido Em</th>
                                <th>Processado Em</th>
                                <th>Ações</th>
                            </tr>
                        </thead>
                        <tbody class={currentYear !== new Date().getFullYear() ? 'opacity-70' : ''}>
                            {#each myRequests as req (req.id)}
                                <tr>
                                    <td>{req.startDateDisplay}</td>
                                    <td>{req.endDateDisplay}</td>
                                    <td>{req.duration} dias úteis</td>
                                    <td>
                                        <span
                                            class="badge badge-sm
                                            {req.status ===
                                            VacationRequestStatus.Approved
                                                ? 'badge-success'
                                                : ''}
                                            {req.status ===
                                            VacationRequestStatus.Pending
                                                ? 'badge-warning'
                                                : ''}
                                            {req.status ===
                                            VacationRequestStatus.Rejected
                                                ? 'badge-error'
                                                : ''}
                                            {req.status === VacationRequestStatus.CancellationRequested
                                                ? 'badge-warning'
                                                : ''}
                                            {req.status === VacationRequestStatus.Cancelled
                                                ? 'badge-ghost'
                                                : ''}
                                        "
                                        >
                                            {#if req.status === VacationRequestStatus.Approved}
                                                Aprovado
                                            {:else if req.status === VacationRequestStatus.Pending}
                                                Pendente
                                            {:else if req.status === VacationRequestStatus.Rejected}
                                                Rejeitado
                                            {:else if req.status === VacationRequestStatus.CancellationRequested}
                                                Cancelamento pedido
                                            {:else if req.status === VacationRequestStatus.Cancelled}
                                                Cancelado
                                            {/if}

                                        </span>
                                    </td>
                                    <td
                                        class="max-w-xs truncate"
                                        title={req.notes || ""}
                                        >{req.notes || "-"}</td
                                    >
                                    <td>{req.requestedAtDisplay}</td>
                                    <td>{req.actionedAtDisplay || "-"}</td>
                                    <td>
                                        {#if req.status === VacationRequestStatus.Pending && currentYear === realCurrentYear}
                                            <button 
                                                class="btn btn-xs btn-error" 
                                                disabled={isCancelling && cancelRequestId === req.id}
                                                onclick={(e) => {
                                                    e.stopPropagation();
                                                    handleCancelRequest(req.id);
                                                }}
                                            >
                                                {#if isCancelling && cancelRequestId === req.id}
                                                    <span class="loading loading-spinner loading-xs"></span>
                                                {:else}
                                                    <i class="fa-solid fa-xmark mr-1"></i> Cancelar
                                                {/if}
                                            </button>
                                        {:else if req.status === VacationRequestStatus.Approved && currentYear === realCurrentYear}
                                            <button
                                                class="btn btn-xs btn-warning"
                                                disabled={cancellationRequestId === req.id}
                                                onclick={(e) => {
                                                    e.stopPropagation();
                                                    handleRequestCancellation(req.id);
                                                }}
                                            >
                                                {#if cancellationRequestId === req.id}
                                                    <span class="loading loading-spinner loading-xs"></span>
                                                {:else}
                                                    <i class="fa-solid fa-ban mr-1"></i> Pedir cancelamento
                                                {/if}
                                            </button>
                                        {:else if req.status === VacationRequestStatus.CancellationRequested}
                                            <span class="text-warning text-xs">A aguardar admin</span>
                                        {:else}
                                            -
                                        {/if}
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {/if}
        </div>
    </div>
</div>

<!-- New Vacation Request Modal -->
<dialog class="modal" bind:this={requestModalRef}>
    <div class="modal-box">
        <form method="dialog">
            <button
                class="btn btn-sm btn-ghost absolute right-2 top-2"
                disabled={isSubmittingRequest}>✕</button
            >
        </form>
        <h3 class="font-bold text-lg">Novo Pedido de Férias</h3>
        <form onsubmit={handleNewRequestSubmit} class="space-y-4 pt-4">
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Data de Início*</span>
                </div>
                <input
                    type="date"
                    class="input input-bordered w-full"
                    bind:value={newRequestStartDate}
                    required
                    disabled={isSubmittingRequest}
                    class:input-error={newRequestErrors.startDate}
                />
                {#if newRequestErrors.startDate}
                    <span class="text-error text-xs mt-1"
                        >{newRequestErrors.startDate}</span
                    >
                {/if}
            </label>
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Data de Fim*</span>
                </div>
                <input
                    type="date"
                    class="input input-bordered w-full"
                    bind:value={newRequestEndDate}
                    required
                    disabled={isSubmittingRequest}
                    class:input-error={newRequestErrors.endDate}
                />
                {#if newRequestErrors.endDate}
                    <span class="text-error text-xs mt-1"
                        >{newRequestErrors.endDate}</span
                    >
                {/if}
            </label>
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">Notas (Opcional)</span>
                </div>
                <textarea
                    class="textarea textarea-bordered w-full"
                    placeholder="Alguma observação sobre o pedido?"
                    rows="3"
                    bind:value={newRequestNotes}
                    disabled={isSubmittingRequest}
                ></textarea>
            </label>
            {#if newRequestErrors.general}
                <div class="alert alert-error text-xs p-2 my-2">
                    {newRequestErrors.general}
                </div>
            {/if}
            <div class="modal-action mt-6">
                <button
                    type="button"
                    class="btn btn-ghost"
                    disabled={isSubmittingRequest}
                    onclick={() => requestModalRef?.close()}>Cancelar</button
                >
                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled={isSubmittingRequest}
                >
                    {#if isSubmittingRequest}
                        <span class="loading loading-spinner loading-sm"></span>
                        A Submeter...
                    {:else}
                        Submeter Pedido
                    {/if}
                </button>
            </div>
        </form>
    </div>
    <form method="dialog" class="modal-backdrop">
        <button disabled={isSubmittingRequest}>close</button>
    </form>
</dialog>

<!-- Styles removed as they were unused -->
