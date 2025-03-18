/**
 * Convert DD/MM/YYYY to Date Object
 */
export function DMYToDate(date: string): Date {
  const day = date.substring(0, 2);
  const month = date.substring(3, 5);
  const year = date.substring(6, 10);
  return new Date(+year, +month - 1, +day);
}

/**
 * Get first date string (DD/MM/YYYY) from Cally Date Range (YYYY/MM/DD-YYYY/MM/DD)
 */
export function getFirstDateFromCallyRange(
  dateRange: string,
  separator: string
): string {
  const day = dateRange.substring(8, 10);
  const month = dateRange.substring(5, 7);
  const year = dateRange.substring(0, 4);
  return `${day}${separator}${month}${separator}${year}`;
}

/**
 * Get second date string (DD/MM/YYYY) from Cally Date Range (YYYY/MM/DD-YYYY/MM/DD)
 */
export function getSecondDateFromCallyRange(
  dateRange: string,
  separator: string
): string {
  const day = dateRange.substring(19, 21);
  const month = dateRange.substring(16, 18);
  const year = dateRange.substring(11, 15);
  return `${day}${separator}${month}${separator}${year}`;
}

/**
 * Get first date string (YYYY/MM/DD) from Date Range (DD/MM/YYYY - DD/MM/YYYY)
 * @returns Tuple with the date and the year
 */
export function getFirstDateFromRangeToYMD(
  dateRange: string,
  separator: string
): [string, string] {
  const day = dateRange.substring(0, 2);
  const month = dateRange.substring(3, 5);
  const year = dateRange.substring(6, 10);
  return [`${year}${separator}${month}${separator}${day}`, year];
}

/**
 * Get second date string (YYYY/MM/DD) from Date Range (DD/MM/YYYY - DD/MM/YYYY)
 */
export function getSecondDateFromRangeToYMD(
  dateRange: string,
  separator: string
): string {
  const day = dateRange.substring(13, 15);
  const month = dateRange.substring(16, 18);
  const year = dateRange.substring(19, 23);
  return `${year}${separator}${month}${separator}${day}`;
}

/**
 * Get first date string (DD/MM/YYYY) from Date Range (DD/MM/YYYY - DD/MM/YYYY) and also return a Date object
 */
export function getFirstDateFromRange(dateRange: string): [string, Date] {
  const day = dateRange.substring(0, 2);
  const month = dateRange.substring(3, 5);
  const year = dateRange.substring(6, 10);
  return [`${day}/${month}/${year}`, new Date(+year, +month - 1, +day)];
}

/**
 * Get second date string (DD/MM/YYYY) from Date Range (DD/MM/YYYY - DD/MM/YYYY) and also return a Date object
 */
export function getSecondDateFromRange(dateRange: string): [string, Date] {
  const day = dateRange.substring(13, 15);
  const month = dateRange.substring(16, 18);
  const year = dateRange.substring(19, 23);
  return [`${day}/${month}/${year}`, new Date(+year, +month - 1, +day)];
}

/**
 * Convert "DD/MM/YYYY, HH:MM:SS" to Date Object
 */
export function DMYHMSToDate(date: string): Date {
  const day = date.substring(0, 2);
  const month = date.substring(3, 5);
  const year = date.substring(6, 10);
  const hour = date.substring(12, 14);
  const minute = date.substring(15, 17);
  const second = date.substring(18, 20);
  return new Date(+year, +month - 1, +day, +hour, +minute, +second);
}
