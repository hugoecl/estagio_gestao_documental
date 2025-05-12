<script lang="ts">
    import { onMount, tick } from "svelte";
    import NotificationDropdown from "./NotificationDropdown.svelte";
    import type { NotificationResponse } from "@lib/types/notification";
    import API_BASE_URL from "@api/base-url"; // Assuming we need API base
    import { handleFetch } from "@api/fetch-handler";
    import {
        showAlert,
        AlertType,
        AlertPosition,
    } from "@components/alert/alert";

    // --- State ---
    let unreadCount = $state(0);
    let lastShownUnreadCount = $state(0); // To avoid spamming alerts
    let notifications = $state<NotificationResponse[]>([]);
    let showDropdown = $state(false);
    let isLoadingCount = $state(true);
    let isLoadingList = $state(false);
    let buttonRef: HTMLButtonElement | null = $state(null); // Ref for the button
    let dropdownRef: HTMLDivElement | null = $state(null); // Ref for the dropdown container

    // --- API Calls ---
    async function fetchUnreadCount() {
        isLoadingCount = true;
        try {
            const response = await handleFetch(
                `${API_BASE_URL}/notifications/unread/count`,
                {
                    method: "GET",
                    credentials: "include",
                },
            );
            if (response.ok) {
                const data = await response.json();
                const newUnreadCount = data.count ?? 0;
                unreadCount = newUnreadCount;

                if (newUnreadCount > 0 && newUnreadCount > lastShownUnreadCount) {
                    showAlert(
                        "Tem novas notificações por ler.",
                        AlertType.INFO,
                        AlertPosition.BOTTOM_RIGHT
                    );
                    lastShownUnreadCount = newUnreadCount;
                } else if (newUnreadCount === 0) {
                    lastShownUnreadCount = 0; // Reset if all are read
                }
            } else {
                // If it's a 401, it's an auth issue, don't show an error alert, just reset count.
                // Middleware should handle actual page redirects if session is truly invalid.
                if (response.status !== 401) {
                     console.error("Failed to fetch unread count:", response.statusText);
                     // Optionally show a generic alert for non-auth errors
                     // showAlert("Erro ao buscar contagem de notificações.", AlertType.WARNING, AlertPosition.TOP);
                }
                unreadCount = 0; // Reset on any error, including 401
            }
        } catch (e) {
            console.error("Error fetching unread count:", e);
            unreadCount = 0; // Reset on error
        } finally {
            isLoadingCount = false;
        }
    }

    async function fetchNotifications() {
        if (!showDropdown) return; // Only fetch when dropdown is open
        isLoadingList = true;
        try {
            const response = await handleFetch(
                `${API_BASE_URL}/notifications/unread`, // Fetch unread initially
                {
                    method: "GET",
                    credentials: "include",
                },
            );
            if (response.ok) {
                const fetchedData = await response.json();
                console.log("Fetched notifications data:", fetchedData); // Log raw data
                notifications = fetchedData; // Assign to state
            } else {
                if (response.status !== 401) {
                    console.error("Failed to fetch notifications:", response.statusText);
                    showAlert(
                        "Erro ao carregar notificações.",
                        AlertType.ERROR,
                        AlertPosition.TOP,
                    );
                }
                notifications = []; // Reset on any error
            }
        } catch (e) {
            console.error("Error fetching notifications:", e);
            notifications = [];
            showAlert(
                "Erro ao carregar notificações.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        } finally {
            isLoadingList = false;
        }
    }

    async function markAsRead(id: number) {
        try {
            const response = await handleFetch(
                `${API_BASE_URL}/notifications/read`,
                {
                    method: "POST",
                    credentials: "include",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ ids: [id] }),
                },
            );
            if (!response.ok) {
                throw new Error(
                    `Failed to mark notification ${id} as read: ${response.statusText}`,
                );
            }
            // Optimistically update UI or refetch
            notifications = notifications.map((n) =>
                n.id === id ? { ...n, isRead: true } : n,
            );
            await fetchUnreadCount(); // Update count immediately
        } catch (e) {
            console.error(`Error marking notification ${id} as read:`, e);
            showAlert(
                "Erro ao marcar notificação como lida.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        }
    }

    async function markAllAsRead() {
        try {
            const response = await handleFetch(
                `${API_BASE_URL}/notifications/read/all`,
                {
                    method: "POST",
                    credentials: "include",
                },
            );
            if (!response.ok) {
                throw new Error(
                    `Failed to mark all notifications as read: ${response.statusText}`,
                );
            }
            // Optimistically update UI or refetch
            notifications = notifications.map((n) => ({ ...n, isRead: true }));
            unreadCount = 0;
        } catch (e) {
            console.error("Error marking all notifications as read:", e);
            showAlert(
                "Erro ao marcar todas as notificações como lidas.",
                AlertType.ERROR,
                AlertPosition.TOP,
            );
        }
    }

    // --- Event Handlers ---
    function toggleDropdown() {
        showDropdown = !showDropdown;
        if (showDropdown) {
            fetchNotifications(); // Fetch list when opening
        }
    }

    // --- Lifecycle & Outside Click ---
    onMount(() => {
        fetchUnreadCount();
        // Optional: Set interval to poll for new count?
        // const intervalId = setInterval(fetchUnreadCount, 60000); // e.g., every minute

        function handleClickOutside(event: MouseEvent) {
            if (
                showDropdown &&
                buttonRef &&
                !buttonRef.contains(event.target as Node) &&
                dropdownRef &&
                !dropdownRef.contains(event.target as Node)
            ) {
                showDropdown = false;
            }
        }

        document.addEventListener("click", handleClickOutside, true);

        return () => {
            // clearInterval(intervalId);
            document.removeEventListener("click", handleClickOutside, true);
        };
    });
</script>

<div class="dropdown dropdown-end">
    <button
        bind:this={buttonRef}
        class="btn btn-ghost m-1"
        onclick={toggleDropdown}
        aria-label="Toggle Notifications"
        aria-haspopup="true"
        aria-expanded={showDropdown}
    >
        <div class="indicator">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-5 w-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
                ></path>
            </svg>
            {#if !isLoadingCount && unreadCount > 0}
                <span
                    class="badge badge-xs badge-primary indicator-item animate-pulse"
                    >{unreadCount > 9 ? "9+" : unreadCount}</span
                >
            {/if}
        </div>
    </button>

    {#if showDropdown}
        <div
            bind:this={dropdownRef}
            class="dropdown-content z-[100] mt-3"
            role="menu"
            aria-orientation="vertical"
            aria-labelledby="notifications-button"
        >
            {#if isLoadingList}
                <div class="card bg-base-100 shadow-lg border border-base-content/10 w-80 p-4">
                    <span class="loading loading-spinner loading-md mx-auto"></span>
                </div>
            {:else}
                <NotificationDropdown
                    {notifications}
                    onMarkAsRead={markAsRead}
                    onMarkAllAsRead={markAllAsRead}
                />
            {/if}
        </div>
    {/if}
</div>
