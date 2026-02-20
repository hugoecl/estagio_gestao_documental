//! Working days calculation: excludes weekends (Sábado, Domingo) and holidays.

use chrono::{Datelike, NaiveDate, Weekday};
use std::collections::HashSet;

/// Fixed Portuguese national holidays (day, month).
const FIXED_HOLIDAYS: &[(u32, u32)] = &[
    (1, 1),   // Ano Novo
    (25, 4),  // Dia da Liberdade
    (1, 5),   // Dia do Trabalhador
    (10, 6),  // Dia de Portugal
    (15, 8),  // Assunção
    (5, 10),  // Implantação da República
    (1, 11),  // Todos os Santos
    (1, 12),  // Restauração
    (8, 12),  // Imaculada Conceição
    (25, 12), // Natal
];

fn easter_sunday(year: i32) -> Option<NaiveDate> {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = ((h + l - 7 * m + 114) / 31) as u32;
    let day = (((h + l - 7 * m + 114) % 31) + 1) as u32;
    NaiveDate::from_ymd_opt(year, month, day)
}

/// Returns all holiday dates (fixed + movable) for the given year.
pub fn get_holiday_dates_for_year(year: i32) -> HashSet<NaiveDate> {
    let mut dates = HashSet::new();

    for &(day, month) in FIXED_HOLIDAYS {
        if let Some(d) = NaiveDate::from_ymd_opt(year, month, day) {
            dates.insert(d);
        }
    }

    if let Some(easter) = easter_sunday(year) {
        dates.insert(easter - chrono::Duration::days(47)); // Carnaval
        dates.insert(easter - chrono::Duration::days(2));  // Sexta-feira Santa
        dates.insert(easter);                               // Páscoa
        dates.insert(easter + chrono::Duration::days(60)); // Corpo de Deus
    }

    dates
}

/// Counts working days (dias úteis) in the range [start, end].
/// Range is INCLUSIVE: both start and end dates are counted.
/// Excludes weekends (Saturday, Sunday) and holidays.
pub fn count_working_days(start: NaiveDate, end: NaiveDate, _year: i32) -> i64 {
    let mut holidays = HashSet::new();
    for y in start.year()..=end.year() {
        holidays.extend(get_holiday_dates_for_year(y));
    }
    let mut count = 0i64;
    let mut current = start;

    while current <= end {
        let wd = current.weekday();
        let is_weekend = wd == Weekday::Sat || wd == Weekday::Sun;
        let is_holiday = holidays.contains(&current);
        if !is_weekend && !is_holiday {
            count += 1;
        }
        if let Some(next) = current.succ_opt() {
            current = next;
        } else {
            break;
        }
    }

    count
}
