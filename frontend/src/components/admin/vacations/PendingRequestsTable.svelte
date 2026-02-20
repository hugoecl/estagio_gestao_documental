<script lang="ts">
    import { onMount, tick } from "svelte";
    import Table from "@components/common/Table.svelte";
    import type { VacationRequestWithUser } from "@lib/types/vacation";
    import { VacationRequestStatus } from "@lib/types/vacation";
    import type { TableColumn } from "@lib/types/table";
    import { getPendingRequestsForRole } from "@api/admin-vacation-api";
    import { getCalendarEvents } from "@api/calendar-api";
    import { countWorkingDays } from "@utils/working-days";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";
    // Import modal for actioning requests
    import ActionVacationRequestModal from "./ActionVacationRequestModal.svelte";

    // Props
    let { roleId, roleName }: { roleId: number; roleName: string } = $props();

    // State
    let pendingRequests = $state<Record<string, VacationRequestWithUser>>({});
    let isLoading = $state(true);
    let error = $state<string | null>(null);

    // State for the action modal
    let actionModalRef: HTMLDialogElement | undefined = $state(undefined);
    let selectedRequestToAction = $state<VacationRequestWithUser | null>(null);
    let allHolidays = $state<Array<{ start_date: string; end_date: string }>>([]);

    const columns: TableColumn[] = [
        { header: "ID Pedido", field: "id" },
        { header: "Utilizador", field: "username" },
        { header: "Email", field: "email" },
        { header: "Data Início", field: "startDateDisplay" },
        { header: "Data Fim", field: "endDateDisplay" },
        { header: "Duração", field: "durationDisplay" },
        { header: "Submetido Em", field: "requestedAtDisplay" },
        { header: "Notas Utilizador", field: "notes" },
    ];

    function processRequestsForDisplay(
        requests: VacationRequestWithUser[],
        holidays: Array<{ start_date: string; end_date: string }>
    ): Record<string, VacationRequestWithUser & { startDateDisplay: string, endDateDisplay: string, requestedAtDisplay: string, durationDisplay: string }> {
        const processed: Record<string, any> = {};
        requests.forEach(req => {
            const start = new Date(req.start_date + "T00:00:00Z");
            const end = new Date(req.end_date + "T00:00:00Z");
            const requestedAt = new Date(req.requested_at);
            let duration = 0;
            if (!isNaN(start.getTime()) && !isNaN(end.getTime()) && end >= start) {
                duration = countWorkingDays(start, end, holidays);
            }
            processed[req.id.toString()] = {
                ...req,
                startDateDisplay: !isNaN(start.getTime()) ? start.toLocaleDateString("pt-PT", { timeZone: 'UTC' }) : "Inválida",
                endDateDisplay: !isNaN(end.getTime()) ? end.toLocaleDateString("pt-PT", { timeZone: 'UTC' }) : "Inválida",
                requestedAtDisplay: requestedAt.toLocaleString("pt-PT"),
                durationDisplay: `${duration} dia${duration !== 1 ? 's' : ''} úteis`,
            };
        });
        return processed;
    }


    async function fetchPendingRequests() {
        isLoading = true;
        error = null;
        try {
            const currentYear = new Date().getFullYear();
            const [requestsArray, holidaysCurr, holidaysPrev] = await Promise.all([
                getPendingRequestsForRole(roleId),
                getCalendarEvents(currentYear),
                getCalendarEvents(currentYear - 1),
            ]);
            allHolidays = [...(holidaysCurr || []), ...(holidaysPrev || [])];
            pendingRequests = processRequestsForDisplay(requestsArray, allHolidays);
        } catch (e: any) {
            console.error(`Error fetching pending requests for role ${roleId}:`, e);
            error = `Erro ao carregar pedidos pendentes: ${e.message}`;
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoading = false;
        }
    }

    onMount(async () => {
        await fetchPendingRequests();
    });

    async function handleRowClick(id: string, request: VacationRequestWithUser) {
        selectedRequestToAction = request;
        await tick();
        if (actionModalRef) {
            actionModalRef.showModal();
        } else {
            console.error("actionModalRef is not yet available.");
        }
    }

    // Function to be called when a request is actioned in the modal
    function handleRequestActioned(actionedRequestId: number) {
        // Refetch or optimistically update the list
        fetchPendingRequests(); // Simplest way is to refetch
        selectedRequestToAction = null; // Reset selection
    }

</script>

{#if error}
    <div class="alert alert-error my-4">{error}</div>
{/if}

<Table
    data={pendingRequests}
    {columns}
    loading={isLoading}
    emptyMessage="Nenhum pedido de férias pendente para esta função."
    searchEmptyMessage="Nenhum pedido pendente encontrado para a sua pesquisa."
    keyField="id"
    searchFields={["username", "email", "startDateDisplay", "endDateDisplay", "notes"]}
    onRowClick={handleRowClick}
    rowClassName="hover:bg-base-300 cursor-pointer"
/>

<!-- 
    Placeholder for ActionVacationRequestModal 
    Will be added in the next step.
-->
{#if selectedRequestToAction}
    <ActionVacationRequestModal
        bind:modalRef={actionModalRef}
        request={selectedRequestToAction}
        holidays={allHolidays}
        onActionSuccess={handleRequestActioned}
        onClose={() => selectedRequestToAction = null}
    />
{/if}