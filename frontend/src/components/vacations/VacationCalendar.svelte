<script lang="ts">
    import { onMount, tick } from "svelte";
    import {
        getMyRemainingVacationDays,
        getMyVacationRequests,
        submitVacationRequest,
        getSharedCalendarVacations, // Import new API function
    } from "@api/vacation-api";
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

    // --- State ---
    let remainingDaysInfo = $state<RemainingVacationDaysResponse | null>(null);
    let myRequests = $state<VacationRequestDisplay[]>([]); // Store processed requests
    let colleagueVacations = $state<
        Array<{ start_date: string; end_date: string }>
    >([]);
    let isLoadingDays = $state(true);
    let isLoadingRequests = $state(true);
    let isLoadingShared = $state(false); // New loading state for shared data
    let error = $state<string | null>(null);

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
    let baseCalendarStructure = $state<CalendarMonth[]>([]);
    let displayedCalendarData = $state<CalendarMonth[]>([]);

    // Date selection state
    let selectionStartDate = $state<Date | null>(null);
    let selectionEndDate = $state<Date | null>(null);
    let hoveredDate = $state<Date | null>(null);

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
    const dayNames = $state(["Seg", "Ter", "Qua", "Qui", "Sex", "Sáb", "Dom"]); // Made reactive

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

    function getDayClasses(day: CalendarDay): string {
        let classes =
            "p-0.5 h-7 w-full flex items-center justify-center text-xs border border-transparent focus:outline-none focus:ring-1 focus:ring-primary"; // Base, no default rounding here

        if (!day.isCurrentMonth) {
            classes += " opacity-40 cursor-not-allowed rounded"; // Non-current month days are fully rounded and faded
        } else {
            classes += " font-semibold";

            const isPreviewing =
                selectionStartDate &&
                !selectionEndDate &&
                hoveredDate &&
                hoveredDate >= selectionStartDate;

            let isInHoverPreviewRange = false;

            if (
                isPreviewing &&
                hoveredDate &&
                selectionStartDate &&
                day.date >= selectionStartDate &&
                day.date <= hoveredDate
            ) {
                isInHoverPreviewRange = true;
            }

            let hasExplicitBg = false;

            let hasSpecificRounding = false;

            // 1. User's actual vacation status (highest priority for background and rounding)

            if (day.status === "user_approved") {
                classes += " bg-success text-success-content rounded";

                hasExplicitBg = true;

                hasSpecificRounding = true;
            } else if (day.status === "user_pending") {
                classes += " bg-warning text-warning-content rounded";

                hasExplicitBg = true;

                hasSpecificRounding = true;
            } else if (day.status === "colleague_approved") {
                classes +=
                    " bg-neutral text-neutral-content opacity-70 cursor-not-allowed rounded";

                hasExplicitBg = true;

                hasSpecificRounding = true;
            }

            // 2. If no vacation status, apply selection or hover preview styling

            if (!hasExplicitBg) {
                if (day.isSelected) {
                    // day.isSelected is true if it's part of *any* selection (confirmed or preview)

                    if (selectionEndDate) {
                        // --- Confirmed Selection ---

                        classes += " text-primary-content";

                        if (day.isRangeStart && day.isRangeEnd) {
                            // Single selected day

                            classes += " bg-primary rounded"; // Fully rounded
                        } else if (day.isRangeStart) {
                            // Head of selected range

                            classes += " bg-accent rounded-l rounded-r-none";
                        } else if (day.isRangeEnd) {
                            // Tail of selected range

                            classes += " bg-accent rounded-r rounded-l-none";
                        } else {
                            // In-between selected range

                            classes += " bg-secondary rounded-none";
                        }

                        hasExplicitBg = true;

                        hasSpecificRounding = true; // Selection logic dictates rounding
                    } else if (isInHoverPreviewRange) {
                        // --- Hover Preview ---

                        classes += " text-info-content";

                        if (
                            day.date.getTime() ===
                                selectionStartDate!.getTime() &&
                            day.date.getTime() === hoveredDate!.getTime()
                        ) {
                            // Single day hover

                            classes += " bg-info rounded";
                        } else if (
                            day.date.getTime() === selectionStartDate!.getTime()
                        ) {
                            // Start of hover preview

                            classes += " bg-accent rounded-l rounded-r-none";
                        } else if (
                            day.date.getTime() === hoveredDate!.getTime()
                        ) {
                            // End of hover preview

                            classes += " bg-accent rounded-r rounded-l-none";
                        } else {
                            // In-between hover preview

                            classes +=
                                " bg-neutral/40 text-neutral-content rounded-none";
                        }

                        hasExplicitBg = true;

                        hasSpecificRounding = true; // Hover preview logic dictates rounding
                    } else if (
                        selectionStartDate &&
                        day.date.getTime() === selectionStartDate.getTime()
                    ) {
                        // Only start date is selected, no hover, no end date (anchor point)

                        classes += " bg-primary text-primary-content rounded";

                        hasExplicitBg = true;

                        hasSpecificRounding = true;
                    }
                }
            }

            // 3. Today's date styling (if not styled by status or selection/hover)

            if (!hasExplicitBg && day.isToday) {
                classes += " !border-primary text-primary rounded";

                // hasExplicitBg is not set here because only border/text changes, background might still be needed for hover
            }

            // 4. Default hover for available, non-selected, non-status, non-preview days

            if (
                !hasExplicitBg &&
                !day.isSelected &&
                !isInHoverPreviewRange &&
                day.isCurrentMonth
            ) {
                if (day.isToday && classes.includes("!border-primary")) {
                    classes += " hover:bg-primary/10";
                } else {
                    classes += " hover:bg-base-300";
                }
            }

            // Apply default full rounding if no specific rounding has been applied yet

            // and it's a current month day (non-current month days already get rounded).

            if (day.isCurrentMonth && !hasSpecificRounding) {
                classes += " rounded";
            }
        }

        if (!day.isCurrentMonth || day.status === "colleague_approved") {
            if (!classes.includes(" cursor-not-allowed"))
                classes += " cursor-not-allowed";
        }

        return classes.trim().replace(/\s+/g, " "); // Clean up multiple spaces
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

    function applyVisualsToCalendar(
        baseStructureInput: CalendarMonth[],
        userRequests: VacationRequestDisplay[],
        colleagueDateRanges: Array<{ start_date: string; end_date: string }>,
        currentSelectionStart: Date | null,
        currentSelectionEnd: Date | null,
        currentHoveredDate: Date | null,
    ): CalendarMonth[] {
        if (!baseStructureInput || !baseStructureInput.length) return [];

        // console.log('applyVisualsToCalendar called with:', { userRequestsLen: userRequests.length, colleagueRangesLen: colleagueDateRanges.length, currentSelectionStart, currentSelectionEnd, currentHoveredDate });

        // IMPORTANT: Create deep copies to avoid mutating original baseStructure or shared day objects.
        // This is often a source of reactivity issues if not handled carefully.
        const baseStructure = JSON.parse(
            JSON.stringify(baseStructureInput),
        ) as CalendarMonth[];

        const colleagueBookedPeriods = colleagueDateRanges.map((range) => ({
            start: new Date(range.start_date + "T00:00:00Z"),
            end: new Date(range.end_date + "T00:00:00Z"),
        }));

        return baseStructure.map((month) => ({
            ...month,
            weeks: month.weeks.map((week) =>
                week.map((day) => {
                    // Ensure day.date is a Date object
                    day.date = new Date(day.date);

                    // Reset visual properties for each day on each run
                    day.status = null;
                    day.tooltip = null;
                    day.isSelected = false;
                    day.isRangeStart = false;
                    day.isRangeEnd = false;

                    // --- 1. Apply Colleague's Approved Vacations ---
                    for (const colleaguePeriod of colleagueBookedPeriods) {
                        if (
                            day.date >= colleaguePeriod.start &&
                            day.date <= colleaguePeriod.end
                        ) {
                            day.status = "colleague_approved";
                            day.tooltip = "Férias Colega";
                            break;
                        }
                    }

                    // --- 2. Apply User's Actual Vacation Request Statuses (Can override colleague for user's own display) ---
                    for (const req of userRequests) {
                        const reqStartDate = new Date(
                            req.start_date + "T00:00:00Z",
                        );
                        const reqEndDate = new Date(
                            req.end_date + "T00:00:00Z",
                        );
                        if (
                            day.date >= reqStartDate &&
                            day.date <= reqEndDate
                        ) {
                            if (req.status === VacationRequestStatus.Pending) {
                                day.status = "user_pending";
                                day.tooltip = `Meu Pedido (Pendente): ${req.startDateDisplay} - ${req.endDateDisplay}`;
                            } else if (
                                req.status === VacationRequestStatus.Approved
                            ) {
                                day.status = "user_approved";
                                day.tooltip = `Meu Pedido (Aprovado): ${req.startDateDisplay} - ${req.endDateDisplay}`;
                            }
                            // No break here if user's status should always override colleague's for display purposes
                        }
                    }

                    // --- 3. Apply Selection / Hover Preview Visuals ---
                    // Only apply if the day doesn't have a blocking status like 'colleague_approved'
                    // User's own 'user_approved' or 'user_pending' might still allow selection visuals if desired,
                    // but typically selection is for available days.
                    if (day.status !== "colleague_approved") {
                        const isPreviewing =
                            currentSelectionStart &&
                            !currentSelectionEnd &&
                            currentHoveredDate &&
                            currentHoveredDate >= currentSelectionStart;
                        let effectiveRangeEnd = currentSelectionEnd;
                        if (isPreviewing && currentHoveredDate) {
                            effectiveRangeEnd = currentHoveredDate;
                        }

                        if (currentSelectionStart && effectiveRangeEnd) {
                            if (
                                day.date >= currentSelectionStart &&
                                day.date <= effectiveRangeEnd
                            ) {
                                day.isSelected = true;
                                if (
                                    day.date.getTime() ===
                                    currentSelectionStart.getTime()
                                ) {
                                    day.isRangeStart = true;
                                }
                                // For the tail, check against actual selectionEndDate if it exists (confirmed selection),
                                // otherwise, if in preview, the hoveredDate is the temporary end.
                                if (
                                    currentSelectionEnd &&
                                    day.date.getTime() ===
                                        currentSelectionEnd.getTime()
                                ) {
                                    day.isRangeEnd = true;
                                } else if (
                                    isPreviewing &&
                                    currentHoveredDate &&
                                    day.date.getTime() ===
                                        currentHoveredDate.getTime()
                                ) {
                                    day.isRangeEnd = true;
                                }
                            }
                        } else if (currentSelectionStart) {
                            if (
                                day.date.getTime() ===
                                currentSelectionStart.getTime()
                            ) {
                                day.isSelected = true;
                                day.isRangeStart = true;
                                day.isRangeEnd = true;
                            }
                        }
                    }
                    return day;
                }),
            ),
        }));
    }

    // --- Fetch Initial Data ---
    async function fetchAllCalendarData(year: number) {
        isLoadingDays = true;
        isLoadingRequests = true;
        isLoadingShared = true; // Set loading for shared data
        error = null;
        try {
            // Fetch all three data points in parallel
            const [daysData, requestsData, sharedColleagueData] =
                await Promise.all([
                    getMyRemainingVacationDays(),
                    getMyVacationRequests(),
                    getSharedCalendarVacations(year), // Fetch colleague data
                ]);

            if (daysData) {
                remainingDaysInfo = daysData;
            } else {
                showAlert(
                    "Não foi possível carregar os seus dias de férias.",
                    AlertType.WARNING,
                    AlertPosition.TOP,
                );
            }
            myRequests = processVacationRequestsForDisplay(requestsData || []);
            colleagueVacations = sharedColleagueData || []; // Store colleague data
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
        await fetchAllCalendarData(currentYear);
        try {
            const [daysData, requestsData] = await Promise.all([
                getMyRemainingVacationDays(),
                getMyVacationRequests(),
            ]);

            if (daysData) {
                remainingDaysInfo = daysData;
            } else {
                showAlert(
                    "Não foi possível carregar os seus dias de férias.",
                    AlertType.WARNING,
                    AlertPosition.TOP,
                );
            }
            myRequests = processVacationRequestsForDisplay(requestsData || []);
        } catch (e: any) {
            console.error("Error fetching vacation data:", e);
            error = `Erro ao carregar dados: ${e.message}`;
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoadingDays = false;
            isLoadingRequests = false;
        }
    });

    // Effect to update displayed calendar when year, myRequests, colleagueVacations, selection, or hover changes
    $effect(() => {
        const year = currentYear;
        const _myRequests = myRequests;
        const _colleagueVacations = colleagueVacations;
        const _selectionStartDate = selectionStartDate;
        const _selectionEndDate = selectionEndDate;
        const _hoveredDate = hoveredDate;

        // Log to see if effect is running and with what data
        // console.log('Effect triggered:', { year, numMyRequests: _myRequests.length, numColleagueVacations: _colleagueVacations.length, start: _selectionStartDate, end: _selectionEndDate, hover: _hoveredDate });

        let currentBase = baseCalendarStructure;
        if (
            !currentBase.length ||
            (currentBase[0] && currentBase[0].year !== year)
        ) {
            // console.log('Generating new base structure for year:', year);
            currentBase = generateBaseCalendarStructure(year);
            baseCalendarStructure = currentBase;
        }

        if (currentBase.length > 0) {
            // console.log('Applying visuals...');
            displayedCalendarData = applyVisualsToCalendar(
                currentBase,
                _myRequests,
                _colleagueVacations,
                _selectionStartDate,
                _selectionEndDate,
                _hoveredDate,
            );
        } else {
            displayedCalendarData = [];
        }
    });

    function handleDayClick(day: CalendarDay) {
        if (!day.isCurrentMonth) return;
        const clickedDate: Date = day.date;

        if (day.status && day.status === "colleague_approved") {
            showAlert(
                "Este dia não está disponível pois coincide com as férias de um colega.",
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
        } else if (!selectionEndDate) {
            // Start is selected, now selecting end
            if (clickedDate.getTime() === selectionStartDate.getTime()) {
                // Clicking the start date again when only start is selected means make it a single-day selection
                selectionEndDate = clickedDate;
            } else if (clickedDate < selectionStartDate) {
                selectionEndDate = selectionStartDate;
                selectionStartDate = clickedDate;
            } else {
                selectionEndDate = clickedDate;
            }
        } else {
            // Both start and end are selected, reset and start new selection
            selectionStartDate = clickedDate;
            selectionEndDate = null;
        }
        hoveredDate = null; // Clear hover when a click modifies the selection
    }

    function clearSelection() {
        selectionStartDate = null;
        selectionEndDate = null;
        hoveredDate = null;
    }

    function processVacationRequestsForDisplay(
        requests: VacationRequest[],
    ): VacationRequestDisplay[] {
        return requests.map((req) => {
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
                duration =
                    Math.round(
                        (end.getTime() - start.getTime()) /
                            (1000 * 60 * 60 * 24),
                    ) + 1;
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
            };
        });
    }

    // --- New Request Modal ---\
    function openRequestModal() {
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
            const requestedDuration =
                Math.round(
                    (end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24),
                ) + 1;
            if (
                remainingDaysInfo &&
                requestedDuration > remainingDaysInfo.remaining_days
            ) {
                newRequestErrors.general = `Não tem dias de férias suficientes. Restantes: ${remainingDaysInfo.remaining_days}, Pedidos: ${requestedDuration}.`;
            }
        }
        return Object.keys(newRequestErrors).length === 0;
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

                isLoadingRequests = true;
                isLoadingDays = true;
                const [daysData, requestsData] = await Promise.all([
                    getMyRemainingVacationDays(),
                    getMyVacationRequests(),
                ]);
                if (daysData) remainingDaysInfo = daysData;
                myRequests = processVacationRequestsForDisplay(
                    requestsData || [],
                );
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
        <div class="flex gap-2">
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
                disabled={!selectionStartDate || !selectionEndDate}
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
            {:else if remainingDaysInfo}
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 text-sm">
                    <div class="text-center p-2 bg-base-100 rounded">
                        <div class="font-semibold">Alocados</div>
                        <div class="text-lg">
                            {remainingDaysInfo.total_allocated_days}
                        </div>
                    </div>
                    <div class="text-center p-2 bg-base-100 rounded">
                        <div class="font-semibold">Aprovados</div>
                        <div class="text-lg text-success">
                            {remainingDaysInfo.approved_days_taken}
                        </div>
                    </div>
                    <div class="text-center p-2 bg-base-100 rounded">
                        <div class="font-semibold">Pendentes</div>
                        <div class="text-lg text-warning">
                            {remainingDaysInfo.pending_days_requested}
                        </div>
                    </div>
                    <div class="text-center p-2 bg-base-100 rounded">
                        <div class="font-semibold">Restantes</div>
                        <div class="text-lg font-bold text-primary">
                            {remainingDaysInfo.remaining_days}
                        </div>
                    </div>
                </div>
            {:else}
                <p class="text-center text-base-content/70">
                    Não foi possível carregar o resumo de dias.
                </p>
            {/if}
        </div>
    </div>

    <!-- Calendar Display -->
    <div class="card bg-base-100 shadow">
        <div class="card-body">
            <div class="flex justify-between items-center mb-4">
                <h2 class="card-title text-base">
                    Calendário Anual - {currentYear}
                </h2>
                <div>
                    <button
                        class="btn btn-sm btn-ghost"
                        onclick={() => currentYear--}
                        title="Ano Anterior"
                    >
                        <i class="fa-solid fa-chevron-left"></i>
                    </button>
                    <button
                        class="btn btn-sm btn-ghost"
                        onclick={() => (currentYear = new Date().getFullYear())}
                        title="Ano Atual"
                    >
                        <i class="fa-solid fa-calendar-day"></i>
                    </button>
                    <button
                        class="btn btn-sm btn-ghost"
                        onclick={() => currentYear++}
                        title="Próximo Ano"
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
                >
                    {#each displayedCalendarData as month (month.year + "-" + month.monthIndex)}
                        <div
                            class="border border-base-content/20 rounded-md p-1.5 bg-base-200/30 shadow-sm min-w-[260px]"
                            onmouseleave={() => {
                                if (selectionStartDate && !selectionEndDate) {
                                    hoveredDate = null;
                                }
                            }}
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
    <div class="card bg-base-100 shadow">
        <div class="card-body">
            <h2 class="card-title text-base">Meus Pedidos Registados</h2>
            {#if isLoadingRequests}
                <div class="flex justify-center py-5">
                    <span class="loading loading-spinner loading-lg"></span>
                </div>
            {:else if myRequests.length === 0}
                <p class="text-center text-base-content/70 py-5">
                    Ainda não tem pedidos de férias registados.
                </p>
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
                            </tr>
                        </thead>
                        <tbody>
                            {#each myRequests as req (req.id)}
                                <tr>
                                    <td>{req.startDateDisplay}</td>
                                    <td>{req.endDateDisplay}</td>
                                    <td>{req.duration} dias</td>
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
                                        "
                                        >
                                            {req.status}
                                        </span>
                                    </td>
                                    <td
                                        class="max-w-xs truncate"
                                        title={req.notes || ""}
                                        >{req.notes || "-"}</td
                                    >
                                    <td>{req.requestedAtDisplay}</td>
                                    <td>{req.actionedAtDisplay || "-"}</td>
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
                class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
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

<style>
    /* Ensure day buttons in a week flow correctly for range highlighting */
    /* Target the grid of day buttons within each week */
    div.grid > div.grid.grid-cols-7 {
        /* Direct child grid for weeks */
        display: flex;
        flex-wrap: nowrap;
    }
    div.grid > div.grid.grid-cols-7 > button {
        /* Buttons within the week grid */
        flex-grow: 1;
        flex-basis: 0;
        min-width: 0;
    }
</style>
