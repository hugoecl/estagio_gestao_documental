<script lang="ts">
    import { onMount, tick } from "svelte";
    import {
        getMyRemainingVacationDays,
        getMyVacationRequests,
        submitVacationRequest,
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
    let myRequests = $state<VacationRequestDisplay[]>([]);
    let isLoadingDays = $state(true);
    let isLoadingRequests = $state(true);
    let error = $state<string | null>(null);

    // --- Custom Calendar State & Logic ---
    interface CalendarDay {
        dayOfMonth: number;
        date: Date; // Represents the actual date (normalized to midnight UTC for comparisons)
        isCurrentMonth: boolean;
        isToday: boolean;
        status: string | null; // e.g., 'user_pending', 'user_approved', 'colleague_approved'
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
    let baseCalendarStructure = $state<CalendarMonth[]>([]); // Raw structure, changes only with year
    let displayedCalendarData = $state<CalendarMonth[]>([]); // What's rendered

    // Date selection state
    let selectionStartDate = $state<Date | null>(null);
    let selectionEndDate = $state<Date | null>(null);

    const monthNames = [
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
    ];
    const dayNames = ["Seg", "Ter", "Qua", "Qui", "Sex", "Sáb", "Dom"];

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

    function generateBaseCalendarStructure(year: number): CalendarMonth[] {
        const newBaseData: CalendarMonth[] = [];
        const today = new Date();
        today.setUTCHours(0, 0, 0, 0);

        for (let monthIndex = 0; monthIndex < 12; monthIndex++) {
            const monthName = monthNames[monthIndex];
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
        baseStructure: CalendarMonth[],
        requests: VacationRequestDisplay[],
        currentSelectionStart: Date | null,
        currentSelectionEnd: Date | null,
    ): CalendarMonth[] {
        if (!baseStructure || !baseStructure.length) return [];

        return baseStructure.map((month) => ({
            ...month,
            weeks: month.weeks.map((week) =>
                week.map((baseDay) => {
                    // Create a copy to avoid mutating baseStructure's day objects
                    const day = { ...baseDay };

                    day.status = null; // Reset status
                    day.tooltip = null;
                    day.isSelected = false;
                    day.isRangeStart = false;
                    day.isRangeEnd = false;

                    // Apply request statuses
                    for (const req of requests) {
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
                            // Add other statuses like 'colleague_approved' here in Phase 4
                            break;
                        }
                    }

                    // Apply selection visuals
                    if (currentSelectionStart && currentSelectionEnd) {
                        if (
                            day.date >= currentSelectionStart &&
                            day.date <= currentSelectionEnd
                        ) {
                            day.isSelected = true;
                            if (
                                day.date.getTime() ===
                                currentSelectionStart.getTime()
                            )
                                day.isRangeStart = true;
                            if (
                                day.date.getTime() ===
                                currentSelectionEnd.getTime()
                            )
                                day.isRangeEnd = true;
                        }
                    } else if (currentSelectionStart) {
                        if (
                            day.date.getTime() ===
                            currentSelectionStart.getTime()
                        ) {
                            day.isSelected = true;
                            day.isRangeStart = true;
                            day.isRangeEnd = true; // Single day selection
                        }
                    }
                    return day;
                }),
            ),
        }));
    }

    // --- Fetch Initial Data ---
    onMount(async () => {
        isLoadingDays = true;
        isLoadingRequests = true;
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
            // Initial generation and display update will be handled by the $effect
        }
    });

    // Effect to update displayed calendar when year, requests, or selection change
    $effect(() => {
        const year = currentYear;
        const requests = myRequests;
        const startSel = selectionStartDate;
        const endSel = selectionEndDate;

        let currentBase = baseCalendarStructure;
        // Only rebuild base structure if year has actually changed or it's empty
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
                requests,
                startSel,
                endSel,
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
            if (clickedDate < selectionStartDate) {
                selectionEndDate = selectionStartDate;
                selectionStartDate = clickedDate;
            } else {
                selectionEndDate = clickedDate;
            }
        } else {
            selectionStartDate = clickedDate;
            selectionEndDate = null;
        }
        // $effect will update displayedCalendarData
    }

    function clearSelection() {
        selectionStartDate = null;
        selectionEndDate = null;
        // $effect will update displayedCalendarData
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

    // --- New Request Modal ---
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
                // The $effect will update displayedCalendarData
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
                                            class="p-0.5 h-7 w-full flex items-center justify-center text-xs
                                                    border border-transparent focus:outline-none focus:ring-1 focus:ring-primary"
                                            class:rounded={!day.isSelected ||
                                                (day.isRangeStart &&
                                                    day.isRangeEnd)}
                                            class:rounded-r-none={day.isSelected &&
                                                day.isRangeStart &&
                                                !day.isRangeEnd &&
                                                day.isCurrentMonth}
                                            class:rounded-l-none={day.isSelected &&
                                                day.isRangeEnd &&
                                                !day.isRangeStart &&
                                                day.isCurrentMonth}
                                            class:rounded-none={day.isSelected &&
                                                !day.isRangeStart &&
                                                !day.isRangeEnd &&
                                                day.isCurrentMonth}
                                            class:opacity-40={!day.isCurrentMonth}
                                            class:font-semibold={day.isCurrentMonth}
                                            class:bg-primary={day.isSelected &&
                                                day.isCurrentMonth}
                                            class:text-primary-content={day.isSelected &&
                                                day.isCurrentMonth}
                                            class:bg-success={day.status ===
                                                "user_approved" &&
                                                !day.isSelected &&
                                                day.isCurrentMonth}
                                            class:text-success-content={day.status ===
                                                "user_approved" &&
                                                !day.isSelected &&
                                                day.isCurrentMonth}
                                            class:bg-warning={day.status ===
                                                "user_pending" &&
                                                !day.isSelected &&
                                                day.isCurrentMonth}
                                            class:text-warning-content={day.status ===
                                                "user_pending" &&
                                                !day.isSelected &&
                                                day.isCurrentMonth}
                                            class:!border-primary={day.isToday &&
                                                !day.status &&
                                                !day.isSelected &&
                                                day.isCurrentMonth}
                                            class:text-primary={day.isToday &&
                                                !day.status &&
                                                !day.isSelected &&
                                                day.isCurrentMonth}
                                            class:hover:bg-base-300={!day.status &&
                                                !day.isSelected &&
                                                day.isCurrentMonth &&
                                                !(
                                                    day.isSelected &&
                                                    day.isCurrentMonth
                                                )}
                                            class:cursor-not-allowed={!day.isCurrentMonth ||
                                                (!!day.status &&
                                                    day.status ===
                                                        "colleague_approved")}
                                            title={day.tooltip ||
                                                `${day.dayOfMonth}/${month.monthIndex + 1}/${month.year}${day.isToday ? " (Hoje)" : ""}`}
                                            disabled={!day.isCurrentMonth ||
                                                (!!day.status &&
                                                    day.status ===
                                                        "colleague_approved")}
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
    .btn-day-selected {
        @apply bg-primary text-primary-content;
    }
    .btn-day-in-range {
        @apply bg-primary/70 text-primary-content rounded-none;
    }
    .btn-day-range-start {
        @apply bg-primary text-primary-content rounded-r-none;
    }
    .btn-day-range-end {
        @apply bg-primary text-primary-content rounded-l-none;
    }

    /* Ensure day buttons in a week flow correctly for range highlighting */
    /* Target the grid of day buttons within each week */
    div.grid > div.grid.grid-cols-7 {
        display: flex; /* Allows children to not wrap individually for rounded corners */
        flex-wrap: nowrap; /* Prevent wrapping within a week */
    }
    div.grid > div.grid.grid-cols-7 > button {
        flex-grow: 1; /* Make buttons take up equal space */
        flex-basis: 0; /* Allow shrinking/growing from 0 basis */
        min-width: 0; /* Important for flex items to shrink properly */
    }
</style>
