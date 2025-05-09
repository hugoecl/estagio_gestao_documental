<script lang="ts">
    import { onMount } from "svelte";
    import Table from "@components/common/Table.svelte";
    import type { TableColumn } from "@lib/types/table";
    import { handleFetch } from "@api/fetch-handler";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    // Props
    let { recordId, apiBaseUrl }: { recordId: number; apiBaseUrl: string } =
        $props();

    // Type for Acknowledgment Detail (matching backend's AcknowledgmentDetail)
    interface AcknowledgmentEntry {
        user_id: number;
        username: string;
        email: string;
        record_id: number; // Though we filter by recordId, it's in the response
        acknowledged_at: string; // ISO string from backend
    }

    // State
    let acknowledgments = $state<Record<string, AcknowledgmentEntry>>({});
    let isLoading = $state(true);
    let error = $state<string | null>(null);

    const columns: TableColumn[] = [
        { header: "ID Utilizador", field: "user_id" },
        { header: "Nome Utilizador", field: "username" },
        { header: "Email", field: "email" },
        { header: "Data Confirmação", field: "acknowledged_at_formatted" },
    ];

    async function fetchAcknowledgments() {
        isLoading = true;
        error = null;
        try {
            const response = await handleFetch(
                `${apiBaseUrl}/records/${recordId}/acknowledgments`,
                {
                    method: "GET",
                    credentials: "include",
                },
            );

            if (response.ok) {
                const data: AcknowledgmentEntry[] = await response.json();
                const ackRecord: Record<string, AcknowledgmentEntry> = {};
                data.forEach((ack) => {
                    // Create a unique key, e.g., user_id (assuming one ack per user per record)
                    // Or use a composite key if needed, or just an index if backend guarantees order
                    ackRecord[ack.user_id.toString()] = {
                        ...ack,
                        acknowledged_at_formatted: new Date(
                            ack.acknowledged_at,
                        ).toLocaleString("pt-PT"),
                    };
                });
                acknowledgments = ackRecord;
            } else if (response.status === 403) {
                error = "Não tem permissão para ver estas confirmações.";
                showAlert(error, AlertType.ERROR, AlertPosition.TOP);
            } else {
                throw new Error(
                    `Falha ao carregar confirmações: ${response.statusText}`,
                );
            }
        } catch (e: any) {
            console.error("Error fetching acknowledgments:", e);
            error = e.message || "Erro desconhecido ao carregar confirmações.";
            showAlert(error, AlertType.ERROR, AlertPosition.TOP);
        } finally {
            isLoading = false;
        }
    }

    onMount(() => {
        if (recordId) {
            fetchAcknowledgments();
        } else {
            error = "ID do Registo não fornecido.";
            isLoading = false;
        }
    });
</script>

{#if error}
    <div class="alert alert-error">{error}</div>
{/if}

<Table
    data={acknowledgments}
    {columns}
    loading={isLoading}
    emptyMessage="Nenhuma confirmação de leitura encontrada para este registo."
    keyField="user_id"
    searchFields={["username", "email", "acknowledged_at_formatted"]}
    rowClassName=""
    onRowClick={() => {}}
/>
