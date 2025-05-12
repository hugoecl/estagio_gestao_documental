<script lang="ts">
    import type { NotificationResponse } from "@lib/types/notification"; // Assume this type exists or will be created
    import { tick } from "svelte";

    // Props
    let {
        notifications = [], // Default to an empty array directly
        onMarkAsRead, // Callback to mark specific notification as read
        onMarkAllAsRead, // Callback to mark all as read
    }: {
        notifications: NotificationResponse[];
        onMarkAsRead: (id: number) => Promise<void>;
        onMarkAllAsRead: () => Promise<void>;
    } = $props();

    let isLoading = $state(false); // State for marking as read

    async function handleMarkAllReadClick() {
        isLoading = true;
        try {
            await onMarkAllAsRead();
            // Parent component should refetch/update the list after this
        } catch (e) {
            console.error("Error marking all notifications as read:", e);
            // Optionally show an alert
        } finally {
            isLoading = false;
        }
    }

    async function handleNotificationClick(notification: NotificationResponse) {
        // 1. Mark as read (optimistically or after backend confirmation)
        if (!notification.isRead) {
            try {
                await onMarkAsRead(notification.id);
                // Parent should update the specific notification in the list
            } catch (e) {
                console.error(
                    `Error marking notification ${notification.id} as read:`,
                    e,
                );
                // Handle error (e.g., show alert) - don't navigate if marking failed?
                return;
            }
        }

        // 2. Navigate to the relevant record/page
        if (notification.pagePath && notification.recordId) {
            // Construct the URL carefully. Assuming record modals are opened via the page path
            // and might need the record ID as a query param or handled differently.
            // This might need adjustment based on how DynamicRecordPage handles opening specific records.
            // For now, just navigate to the page path. The record handling will need refinement later.
            const targetPath = notification.pagePath.endsWith("/")
                ? notification.pagePath
                : `${notification.pagePath}/`;
            // We probably need to trigger the modal open for the specific recordId here
            // instead of just navigating. This requires more complex state management or
            // potentially URL parameters. Let's just log for now.

            // Example of a possible future navigation (might need refinement):
            // window.location.href = `${targetPath}?record=${notification.recordId}`;
            if (typeof window !== "undefined") {
                window.location.href = targetPath; // Navigate to the page for now
            }
        } else {
            console.warn(
                "Notification is missing path or record ID:",
                notification,
            );
        }
    }

    // Function to format date (relative or absolute)
    function formatTimeAgo(
        dateString: string | Date | undefined | null,
    ): string {
        if (!dateString) {
            // console.warn("formatTimeAgo received invalid input:", dateString);
            return "Data inválida";
        }

        let date: Date;
        try {
            date =
                typeof dateString === "string"
                    ? new Date(dateString)
                    : dateString;
            if (isNaN(date.getTime())) {
                // Check if the date object is valid
                // console.warn("formatTimeAgo could not parse date:", dateString);
                return "Data inválida";
            }
        } catch (e) {
            // console.error("Error creating date in formatTimeAgo:", e);
            return "Data inválida";
        }

        const now = new Date();
        const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

        // Check if date is in the future or invalid seconds calculation
        if (seconds < 0 || isNaN(seconds)) {
            // console.warn("formatTimeAgo calculated invalid seconds:", seconds, "for date:", date);
            return "Data inválida"; // Or return formatted future date?
        }

        let interval = Math.floor(seconds / 31536000);
        if (interval > 1) return `${interval} anos atrás`;
        if (interval === 1) return `1 ano atrás`;

        interval = Math.floor(seconds / 2592000);
        if (interval > 1) return `${interval} meses atrás`;
        if (interval === 1) return `1 mês atrás`;

        interval = Math.floor(seconds / 86400);
        if (interval > 1) return `${interval} dias atrás`;
        if (interval === 1) return `1 dia atrás`;

        interval = Math.floor(seconds / 3600);
        if (interval > 1) return `${interval} horas atrás`;
        if (interval === 1) return `1 hora atrás`;

        interval = Math.floor(seconds / 60);
        if (interval > 1) return `${interval} minutos atrás`;
        if (interval === 1) return `1 minuto atrás`;

        return `agora mesmo`;
    }
</script>

<div
    class="card bg-base-100 shadow-lg border border-base-content/10 w-80 max-h-[70vh] flex flex-col"
>
    <div class="card-body p-3 flex-none">
        <div class="flex justify-between items-center">
            <h3 class="card-title text-base">Notificações</h3>
            {#if notifications.length > 0}
                <button
                    class="btn btn-xs btn-ghost text-primary"
                    onclick={handleMarkAllReadClick}
                    disabled={isLoading}
                >
                    {#if isLoading}
                        <span class="loading loading-spinner loading-xs"></span>
                    {:else}
                        Marcar todas como lidas
                    {/if}
                </button>
            {/if}
        </div>
    </div>

    <ul class="menu menu-sm p-0 overflow-y-auto flex-grow w-full">
        {#if notifications.length === 0}
            <li class="p-4 text-center text-base-content/60">
                Nenhuma notificação nova.
            </li>
        {:else}
            {#each notifications as notification (notification.id)}
                <li class:opacity-60={notification.isRead}>
                    <a
                        href="#"
                        class="block whitespace-normal hover:bg-base-200"
                        onclick={(e) => {
                            e.preventDefault();
                            handleNotificationClick(notification);
                        }}
                    >
                        <div class="flex flex-col">
                            <span class="font-semibold text-sm">
                                {#if !notification.isRead}
                                    <span class="text-primary mr-1">•</span>
                                {/if}
                                {notification.message}
                            </span>
                            <span class="text-xs text-base-content/70 mt-1">
                                {formatTimeAgo(notification.createdAt)}
                                {#if notification.pageName}
                                    <span> • {notification.pageName}</span>
                                {/if}
                            </span>
                        </div>
                    </a>
                </li>
            {/each}
        {/if}
    </ul>

    {#if notifications.length > 0}
        <div class="card-actions p-2 border-t border-base-content/10 flex-none">
            <!-- Optional: Link to see all notifications -->
            <!-- <a href="/notifications" class="btn btn-sm btn-ghost w-full">Ver todas</a> -->
        </div>
    {/if}
</div>
```

<style>
    /* Add any specific styles if needed */
    li > a {
        border-radius: 0; /* Remove default radius for tighter list */
        padding-top: 0.6rem;
        padding-bottom: 0.6rem;
    }
    li:not(:last-child) > a {
        border-bottom: 1px solid oklch(var(--b2) / 0.4);
    }
</style>
