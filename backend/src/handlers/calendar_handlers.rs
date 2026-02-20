//! Handlers for calendar events (fixed and movable holidays).
//! Phase 1: Fixed national holidays + movable (Easter-based). No database.

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use chrono::NaiveDate;
use serde::Serialize;

use crate::auth::validate_session;

/// Query parameters for calendar events endpoint.
#[derive(serde::Deserialize)]
pub struct CalendarEventsQuery {
    pub year: i32,
}

/// Response item for a calendar event (holiday).
#[derive(Serialize)]
pub struct CalendarEventResponse {
    pub start_date: String,
    pub end_date: String,
    pub title: String,
}

/// Fixed Portuguese national holidays (day, month, title).
const FIXED_HOLIDAYS: &[(u32, u32, &str)] = &[
    (1, 1, "Ano Novo"),
    (25, 4, "Dia da Liberdade"),
    (1, 5, "Dia do Trabalhador"),
    (10, 6, "Dia de Portugal"),
    (15, 8, "Assunção de Nossa Senhora"),
    (5, 10, "Implantação da República"),
    (1, 11, "Todos os Santos"),
    (1, 12, "Restauração da Independência"),
    (8, 12, "Imaculada Conceição"),
    (25, 12, "Natal"),
];

/// Computes Easter Sunday for the given year using the Anonymous Gregorian algorithm.
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

/// Returns movable holidays for the given year (Carnaval, Sexta-feira Santa, Páscoa, Corpo de Deus).
fn movable_holidays(year: i32) -> Vec<CalendarEventResponse> {
    let Some(easter) = easter_sunday(year) else {
        return Vec::new();
    };

    let mut events = Vec::new();

    // Carnaval (Terça-feira de Carnaval) = 47 days before Easter
    {
        let carnaval = easter - chrono::Duration::days(47);
        let s = carnaval.format("%Y-%m-%d").to_string();
        events.push(CalendarEventResponse {
            start_date: s.clone(),
            end_date: s,
            title: "Carnaval".to_string(),
        });
    }

    // Sexta-feira Santa = 2 days before Easter
    {
        let sexta_santa = easter - chrono::Duration::days(2);
        let s = sexta_santa.format("%Y-%m-%d").to_string();
        events.push(CalendarEventResponse {
            start_date: s.clone(),
            end_date: s,
            title: "Sexta-feira Santa".to_string(),
        });
    }

    // Páscoa (Easter Sunday)
    let s = easter.format("%Y-%m-%d").to_string();
    events.push(CalendarEventResponse {
        start_date: s.clone(),
        end_date: s,
        title: "Páscoa".to_string(),
    });

    // Corpo de Deus = 60 days after Easter (Thursday)
    {
        let corpo_deus = easter + chrono::Duration::days(60);
        let s = corpo_deus.format("%Y-%m-%d").to_string();
        events.push(CalendarEventResponse {
            start_date: s.clone(),
            end_date: s,
            title: "Corpo de Deus".to_string(),
        });
    }

    events
}

/// Returns fixed national holidays for the given year.
/// Requires authenticated session.
pub async fn get_calendar_events(
    session: Session,
    query: web::Query<CalendarEventsQuery>,
) -> impl Responder {
    let _user_id = match validate_session(&session) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let year = query.year;

    let mut events: Vec<CalendarEventResponse> = FIXED_HOLIDAYS
        .iter()
        .filter_map(|&(day, month, title)| {
            NaiveDate::from_ymd_opt(year, month, day).map(|date| {
                let date_str = date.format("%Y-%m-%d").to_string();
                CalendarEventResponse {
                    start_date: date_str.clone(),
                    end_date: date_str,
                    title: title.to_string(),
                }
            })
        })
        .collect();

    events.extend(movable_holidays(year));

    HttpResponse::Ok().json(events)
}
