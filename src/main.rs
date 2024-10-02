use anyhow::anyhow;
use anyhow::Result;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::{DateTime, TimeDelta, Utc};
use gloo_storage::Storage;
use leptos::*;

use leptos_use::*;

use thaw::*;

const START_TIME_KEY: &str = "start_time";
const START_INTERVAL_KEY: &str = "interval";

fn save_start_time(start_time: DateTime<Utc>) -> Result<()> {
    leptos::logging::log!("saving start time: {:?}", start_time);
    Ok(gloo_storage::LocalStorage::set(START_TIME_KEY, start_time)?)
}

fn get_start_time() -> Result<DateTime<Utc>> {
    leptos::logging::log!("loading start time");
    let get = gloo_storage::LocalStorage::get(START_TIME_KEY);
    logging::log!("from storage: {:?}", get);
    Ok(get?)
}

fn save_start_interval(interval: TimeDelta) -> Result<()> {
    Ok(gloo_storage::LocalStorage::set(
        START_INTERVAL_KEY,
        interval.num_seconds(),
    )?)
}

fn get_start_interval() -> Result<TimeDelta> {
    let interval = gloo_storage::LocalStorage::get(START_INTERVAL_KEY)?;

    TimeDelta::try_seconds(interval).ok_or(anyhow!("invalid interval"))
}

// Treats the parameters as dates and returns the TimeDelta.
// In other words, the dates are treated as if they are midnight times and performs subtraction.
// Example:
//   from:   2024-09-30
//   amount: 2024-09-28
//   output: TimeDelta(days=2)
fn subtract_dates(from: DateTime<Local>, amount: DateTime<Local>) -> TimeDelta {
    TimeDelta::days(1) * (from.date_naive() - amount.date_naive()).num_days() as i32
}

// Convert NaiveDateTime to DateTime<Local>.
fn naive_datetime_to_local(naive_datetime: NaiveDateTime) -> Result<DateTime<Local>> {
    let local_timezone = Local;
    let mapped_time = local_timezone.from_local_datetime(&naive_datetime);
    let local_datetime = mapped_time
        .single()
        .ok_or(anyhow!("no local time found in {naive_datetime}"))?;
    Ok(local_datetime)
}

#[component]
fn DateTimeSet(initial_time_rw_signal: RwSignal<Option<DateTime<Utc>>>) -> impl IntoView {
    let date_signal = RwSignal::new(None);
    let time_signal = RwSignal::new(None);
    if let Ok(start_time) = get_start_time() {
        let local_time: DateTime<Local> = DateTime::from(start_time);
        date_signal.set(Some(local_time.date_naive()));
        time_signal.set(Some(local_time.time()));
    }

    create_effect(move |_| {
        let new_time = time_signal.get();
        let new_date = date_signal.get();

        let (Some(new_time), Some(new_date)) = (new_time, new_date) else {
            logging::log!("new_time {:?} or new_date {:?} is None", new_time, new_date);
            return;
        };
        logging::log!("New TimeSet via timepicker to: {:?}", new_time);
        logging::log!("New TimeSet via date to: {:?}", new_date);
        let new_date_time = NaiveDateTime::new(new_date, new_time);
        logging::log!("New TimeSet to: {:?}", new_date_time);
        let local_date_time = naive_datetime_to_local(new_date_time);
        let Ok(local_date_time) = local_date_time else {
            logging::error!("local_date_time is None");
            return;
        };

        let utc_date_time = local_date_time.to_utc();
        save_start_time(utc_date_time);
        initial_time_rw_signal.set(Some(utc_date_time));
    });

    view! {
        <Flex>
            <label>"開始時刻: "</label>
            <DatePicker value=date_signal/>
            <TimePicker value=time_signal/>
        </Flex>
    }
}

#[component]
fn Interval(interval_rw_signal: RwSignal<TimeDelta>) -> impl IntoView {
    // TODO: add a tooltip.
    // example:
    // https://flowbite.com/docs/components/tooltips/
    // Consider warning user of 0 sec interval. They might end up "catching up" and missing a few
    // artifacts.
    let (interval, set_interval) = interval_rw_signal.split();
    if let Ok(saved_interval) = get_start_interval() {
        set_interval.set(saved_interval);
    }

    view! {
        <div>
            <label for="interval-per-day">"毎日置きたい間隔(秒):"</label>
            <input
                type="number"
                id="interval-per-day"
                name="interval-per-day"
                min="0"
                prop:value=move || interval.get().num_seconds()
                on:input=move |ev| {
                    let value = event_target_value(&ev);
                    if let Ok(seconds) = value.parse::<i64>() {
                        logging::log!("Parsed interval: {}", seconds);
                        let seconds = TimeDelta::seconds(seconds);
                        set_interval.set(seconds);
                        save_start_interval(seconds);
                    } else {
                        logging::error!("Failed to parse value to integer: {value}");
                    }
                }
            />

        </div>
    }
}

#[component]
fn StartTimeToday(
    iniitial_start_time: ReadSignal<Option<DateTime<Utc>>>,
    interval: ReadSignal<TimeDelta>,
) -> impl IntoView {
    fn todays_start_time(
        initial_start_time: DateTime<Local>,
        interval: TimeDelta,
    ) -> Option<DateTime<Local>> {
        let days_since_start = subtract_dates(Local::now(), initial_start_time);
        if days_since_start.num_days() < 0 {
            return None;
        }

        let offset = interval * days_since_start.num_days() as i32;
        Some(initial_start_time + days_since_start + offset)
    }

    let today = Local::now();
    let formatted_date = today.format("%Y-%m-%d").to_string();

    let (current_time, set_current_time) = create_signal(Local::now());
    use_interval_fn(
        move || {
            set_current_time.set(Local::now());
        },
        1000,
    );

    view! {
        <div>
            "現在時刻:"
            {move || current_time.get().format("%Y-%m-%d %H:%M:%S").to_string()}
        </div>

        <div>
            "今日(" {formatted_date} ")の開始時間は"
            <span class="badge text-bg-primary">
                {move || {
                    let initial_start_time = iniitial_start_time.get();
                    let interval = interval.get();
                    let Some(initial_start_time) = initial_start_time else {
                        return "不明".to_string();
                    };
                    let initial_start_time: DateTime<Local> = DateTime::from(initial_start_time);
                    let start_local_time = todays_start_time(initial_start_time, interval);
                    if let Some(start_local_time) = start_local_time {
                        start_local_time.format("%H:%M:%S").to_string()
                    } else {
                        "不明".to_string()
                    }
                }}

            </span>
        </div>
    }
}

#[component]
fn ThemeSwitcher() -> impl IntoView {
    let set_html_theme = move |theme: &str| {
        let document = use_document();
        let body = document.body();
        body.expect("body should exist")
            .set_attribute("data-bs-theme", theme)
            .unwrap();
    };
    view! {
        <Button class="btn btn-dark border" on_click=move |_| {
            set_html_theme("dark");
        }>"Dark"</Button>
        <Button class="btn btn-light border" on_click=move |_| {
            set_html_theme("light");
        }>"Light"</Button>
    }
}

#[component]
fn DebugFeatures() -> impl IntoView {
    let switch_signal = create_rw_signal(false);
    create_effect(move |_| {
        if switch_signal.get() {
            logging::log!("Debug features are on");
        } else {
            logging::log!("Debug features are off");
        }
    });

    view! {
        "開発用"
        <Switch value=switch_signal class="border"/>
        <div hidden=move || !switch_signal.get()>
            <Button on_click=move |_| {
                gloo_storage::LocalStorage::clear();
            }>"Clear storage"</Button>
        </div>
    }
}

fn main() {
    let start_time_rw_signal: RwSignal<Option<DateTime<Utc>>> = create_rw_signal(None);
    let interval_rw_signal: RwSignal<TimeDelta> = create_rw_signal(TimeDelta::zero());
    mount_to_body(move || {
        view! {
            <h1>"聖遺物マラソン開始時間計算"</h1>
            <h2>
                <StartTimeToday
                    iniitial_start_time=start_time_rw_signal.read_only()
                    interval=interval_rw_signal.read_only()
                />
            </h2>

            <DateTimeSet initial_time_rw_signal=start_time_rw_signal/>
            <Interval interval_rw_signal=interval_rw_signal/>
            <ThemeSwitcher/>
            <hr/>

            <DebugFeatures/>
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtract_dates_negative_days() {
        let start = Local.ymd(2022, 5, 1).and_hms(0, 0, 0);
        let end = Local.ymd(2022, 5, 2).and_hms(0, 0, 0);
        assert_eq!(subtract_dates(start, end), TimeDelta::days(-1));
    }

    #[test]
    fn test_subtract_dates_positive_days() {
        let start = Local.ymd(2022, 5, 3).and_hms(0, 0, 0);
        let end = Local.ymd(2022, 5, 2).and_hms(0, 0, 0);
        assert_eq!(subtract_dates(start, end), TimeDelta::days(1));
    }
}
