/**
 * Counts working days (dias úteis) in a date range.
 * Range is INCLUSIVE: start and end dates both count.
 * Excludes weekends (Saturday, Sunday) and holidays.
 */
export function countWorkingDays(
    start: Date,
    end: Date,
    holidays: Array<{ start_date: string; end_date: string }>
): number {
    const holidaySet = new Set<string>();
    for (const h of holidays) {
        const s = new Date(h.start_date + "T00:00:00Z");
        const e = new Date(h.end_date + "T00:00:00Z");
        for (let d = new Date(s); d <= e; d.setUTCDate(d.getUTCDate() + 1)) {
            holidaySet.add(d.toISOString().slice(0, 10));
        }
    }
    let count = 0;
    const curr = new Date(start);
    curr.setUTCHours(0, 0, 0, 0);
    const endNorm = new Date(end);
    endNorm.setUTCHours(0, 0, 0, 0);
    while (curr <= endNorm) {
        const day = curr.getUTCDay();
        const dateStr = curr.toISOString().slice(0, 10);
        if (day !== 0 && day !== 6 && !holidaySet.has(dateStr)) count++;
        curr.setUTCDate(curr.getUTCDate() + 1);
    }
    return count;
}
