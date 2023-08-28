use chrono::Datelike;

pub(super) fn date() -> u32 {
    let now = chrono::Utc::now().date_naive();

    let year = (now.year() as u32) * 10000;
    let month = now.month() * 100;
    let day = now.day();

    year + month + day
}
