use anyhow::anyhow;
use anyhow::Result;
use chrono::Local;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::TimeZone;
use chrono::{DateTime, TimeDelta, Utc};
use gloo_storage::Storage;
use leptos::*;

use leptos_use::*;

use thaw::*;

mod protos;
mod storage;

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
fn DateTimeSet(
    date_signal: RwSignal<Option<NaiveDate>>,
    time_signal: RwSignal<Option<NaiveTime>>,
) -> impl IntoView {
    view! {
        <Flex>
            <label>"開始時刻: "</label>
            <DatePicker value=date_signal />
            <TimePicker value=time_signal />
        </Flex>
    }
}

// Component to use the current time as start time.
#[component]
fn SetCurrentTimeAsStartTime(
    #[prop(into)] set_start_time: Callback<DateTime<Local>>,
) -> impl IntoView {
    view! {
        <Button
            class="btn btn-primary"
            on:click=move |_| {
                set_start_time.call(Local::now());
            }
        >
            "現在時刻を開始時刻として保存"
        </Button>
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
                        let seconds = TimeDelta::seconds(seconds);
                        set_interval.set(seconds);
                    } else {
                        logging::error!("Failed to parse value to integer: '{value}'");
                    }
                }
            />

        </div>
    }
}

#[component]
fn StartTimeToday(
    #[prop(into)] get_initial_start_time: Callback<(), Option<DateTime<Utc>>>,
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

    let (date_today, set_date_today) = create_signal(Local::now());

    let (current_time, set_current_time) = create_signal(Local::now());
    use_interval_fn(
        move || {
            set_current_time.set(Local::now());
        },
        1000,
    );

    create_effect(move |_| {
        let now = current_time.get();
        let diff = subtract_dates(now, date_today.get());
        if diff.num_days() != 0 {
            set_date_today.set(now);
        }
    });

    view! {
        <div>"現在時刻:" {move || current_time.get().format("%H:%M:%S").to_string()}</div>

        <div>
            "今日(" {move || date_today.get().format("%Y-%m-%d").to_string()}
            ")の開始時間は"
            <span class="badge text-bg-primary">
                // Access the date today so that this gets updated every day.
                // The value is not used, since the time is calculated independent of the value.
                // Note: This comment cannot be in the block below. Leptosfmt deletes it.
                {move || {
                    let _ = date_today.get();
                    let initial_start_time = get_initial_start_time.call(());
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
        <Button
            class="btn btn-dark border"
            on_click=move |_| {
                set_html_theme("dark");
            }
        >
            "Dark"
        </Button>
        <Button
            class="btn btn-light border"
            on_click=move |_| {
                set_html_theme("light");
            }
        >
            "Light"
        </Button>
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
        <Switch value=switch_signal class="border" />
        <div hidden=move || !switch_signal.get()>
            <Button on_click=move |_| {
                gloo_storage::LocalStorage::clear();
            }>"Clear storage"</Button>
        </div>
    }
}

fn main() {
    let storage = storage::Storage::new();
    let interval_rw_signal: RwSignal<TimeDelta> =
        create_rw_signal(storage.get_start_interval().unwrap_or(TimeDelta::zero()));

    // TODO: Clean this up. Return both date and time?
    let start_time = storage.get_start_time();
    let date_signal = start_time.map(|start_time| {
        let start_time: DateTime<Local> = DateTime::from(start_time);
        start_time.date_naive()
    });
    let time_signal = start_time.map(|start_time| {
        let start_time: DateTime<Local> = DateTime::from(start_time);
        start_time.time()
    });

    // These are the source of truth for the start time.
    let date_signal: RwSignal<Option<NaiveDate>> = RwSignal::new(date_signal);
    let time_signal: RwSignal<Option<NaiveTime>> = RwSignal::new(time_signal);

    // Memoized drived signal for calculating the initial start time in UTC, when the user
    // interacts with the DatePicker or the TimePicker.
    // Note that the start time is saved to local storage.
    let get_initial_start_time = create_memo(move |_| {
        let new_time = time_signal.get();
        let new_date = date_signal.get();

        let (Some(new_time), Some(new_date)) = (new_time, new_date) else {
            logging::log!("new_time {:?} or new_date {:?} is None", new_time, new_date);
            return None;
        };
        let new_date_time = NaiveDateTime::new(new_date, new_time);
        let local_date_time = naive_datetime_to_local(new_date_time);
        let Ok(local_date_time) = local_date_time else {
            logging::error!("local_date_time is None");
            return None;
        };

        let utc_date_time = local_date_time.to_utc();
        Some(utc_date_time)
    });

    create_effect(move |_| {
        let value = get_initial_start_time.get();
        if value.is_none() {
            return;
        }
        let mut storage = storage::Storage::new();
        storage.set_start_time(value.unwrap());
    });

    create_effect(move |_| {
        let interval = interval_rw_signal.get();
        let mut storage = storage::Storage::new();
        storage.set_start_interval(interval)
    });

    let set_start_time = move |new_time: DateTime<Local>| {
        date_signal.set(Some(new_time.date_naive()));
        time_signal.set(Some(new_time.time()));
    };

    mount_to_body(move || {
        view! {
            <h1>"聖遺物マラソン開始時間計算"</h1>
            <h2>
                <StartTimeToday
                    get_initial_start_time=move |_| get_initial_start_time.get()
                    interval=interval_rw_signal.read_only()
                />
            </h2>

            <DateTimeSet date_signal time_signal />
            <SetCurrentTimeAsStartTime set_start_time />
            <Interval interval_rw_signal=interval_rw_signal />
            <hr />

            <DebugFeatures />
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

    // Even though they are only different by 1 second, the result should be 1 day since the
    // dates are different.
    #[test]
    fn test_subtract_dates_only_1_sec_diff() {
        let start = Local.ymd(2023, 7, 3).and_hms(0, 0, 0);
        let end = Local.ymd(2023, 7, 2).and_hms(23, 59, 59);
        assert_eq!(subtract_dates(start, end), TimeDelta::days(1));
    }
}
