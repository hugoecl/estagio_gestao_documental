import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";

export interface CalendarEvent {
    start_date: string;
    end_date: string;
    title: string;
}

/**
 * Fetches fixed calendar events (holidays) for the given year.
 * Requires authenticated session.
 */
export async function getCalendarEvents(
    year: number
): Promise<CalendarEvent[]> {
    const response = await handleFetch(
        `${API_BASE_URL}/calendar-events?year=${year}`,
        {
            method: "GET",
            credentials: "include",
        }
    );

    if (response.ok) {
        return (await response.json()) as CalendarEvent[];
    }

    if (response.status === 401) {
        return [];
    }

    console.warn(
        `Failed to fetch calendar events for year ${year}: ${response.statusText}`
    );
    return [];
}
