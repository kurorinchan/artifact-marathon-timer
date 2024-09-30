use anyhow::anyhow;
use anyhow::Result;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::{DateTime, TimeDelta, Utc};
use gloo_storage::Storage;
use leptos::*;

use thaw::*;

const START_TIME_KEY: &str = "start_time";
const START_INTERVAL_KEY: &str = "interval";

const TIME_KEY_EXPERIMENT: &str = "experimental_time_key";

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

// The time format is like
fn javascript_time_to_local(time_string: &str) -> Result<DateTime<Local>> {
    let naive_datetime = NaiveDateTime::parse_from_str(time_string, "%Y-%m-%dT%H:%M:%S")?;
    let local_timezone = Local;
    let mapped_time = local_timezone.from_local_datetime(&naive_datetime);
    let local_datetime = mapped_time
        .single()
        .ok_or(anyhow!("no local time found in {time_string}"))?;
    Ok(local_datetime)
}

fn local_datetime_to_javascrip_time(local_datetime: DateTime<Local>) -> String {
    local_datetime.format("%Y-%m-%dT%H:%M:%S").to_string()
}

#[component]
fn TimeSet() -> impl IntoView {
    // Note. It has to have Some() to match the type. Seems like the documentation is currently
    // wrong.
    let value = RwSignal::new(Some(Local::now().time()));
    if let Ok(start_time) = get_start_time() {
        let local_time = Local.from_utc_datetime(&start_time.naive_utc());
        value.set(Some(local_time.time()));
    }

    create_effect(move |_| {
        logging::log!("New TimeSet via timepicker to: {:?}", value.get());
    });

    view! {
        <Flex vertical=true>
            <TimePicker value={value} />
        </Flex>
    }
}

#[component]
fn InitialRunStartTime(start_time_rw_signal: RwSignal<DateTime<Utc>>) -> impl IntoView {
    let (start_time, set_start_time) = start_time_rw_signal.split();
    if let Ok(start) = get_start_time() {
        set_start_time.set(start);
    }

    let start_time_local = move || -> DateTime<Local> { DateTime::from(start_time.get()) };

    view! {
        <div>
        <label for="start-datetime">"開始時刻: "</label>
        <input
            name="start-datetime"
            type="datetime-local"
            prop:value=move || {
                let js_time = local_datetime_to_javascrip_time(start_time_local());
                logging::log!("prop:value = {}", js_time);
                js_time
            }

            step="1"
            on:input=move |ev| {
                logging::log!("input event fired!");
                let value = event_target_value(&ev);
                let local_datetime = javascript_time_to_local(&value);
                if let Ok(datetime) = local_datetime {
                    logging::log!("parsed datetime from input element: {:?}", datetime);
                    let datetime = datetime.to_utc();
                    set_start_time.set(datetime);
                    save_start_time(datetime);
                }
            }
        />
        </div>
    }
}

#[component]
fn interval(interval_rw_signal: RwSignal<TimeDelta>) -> impl IntoView {
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
            <input type="number" id="interval-per-day" name="interval-per-day" min="0"
                prop:value=move || interval.get().num_seconds()
                on:input= move |ev| {
                    let value = event_target_value(&ev);
                    if let Ok(seconds) =value.parse::<i64>() {
                        logging::log!("Parsed interval: {}", seconds);
                        let seconds = TimeDelta::seconds(seconds);
                        set_interval.set(seconds);
                        save_start_interval(seconds);
                    } else {
                        logging::error!("Failed to parse value to integer: {value}");
                    }
                } />
        </div>
    }
}

#[component]
fn StartTimeToday(
    iniitial_start_time: ReadSignal<DateTime<Utc>>,
    interval: ReadSignal<TimeDelta>,
) -> impl IntoView {
    fn todays_start_time(
        iniitial_start_time: DateTime<Local>,
        interval: TimeDelta,
    ) -> Option<DateTime<Local>> {
        let now = Local::now();
        let days_since_start = (now - iniitial_start_time).num_days();
        if days_since_start < 0 {
            return None;
        }

        let today_start_time_no_interval_offset =
            iniitial_start_time + TimeDelta::days(days_since_start);

        let days_since_start = i32::try_from(days_since_start).ok()?;
        let offset = interval * days_since_start;

        Some(today_start_time_no_interval_offset + offset)
    }

    view! {
        <div>
        "今日の開始時間は"
        {move || {
            let initial_start_time = iniitial_start_time.get();
            let interval = interval.get();

            let initial_start_time: DateTime<Local> = DateTime::from(initial_start_time);
            let start_local_time = todays_start_time(initial_start_time, interval);
            if let Some(start_local_time) = start_local_time {
                start_local_time.format("%H:%M:%S").to_string()
            } else {
                "不明".to_string()
            }

        }}
        </div>
    }
}

fn main() {
    let start_time_rw_signal = create_rw_signal(Utc::now());
    let interval_rw_signal = create_rw_signal(TimeDelta::zero());
    mount_to_body(move || {
        view! {
            <InitialRunStartTime start_time_rw_signal=start_time_rw_signal/>
            <Interval interval_rw_signal=interval_rw_signal/>
            <StartTimeToday
                iniitial_start_time=start_time_rw_signal.read_only()
                interval=interval_rw_signal.read_only()
            />
            <TimeSet />
        }
    });
}
