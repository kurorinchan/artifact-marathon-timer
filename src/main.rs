use anyhow::anyhow;
use anyhow::Result;
use chrono::FixedOffset;
use chrono::Local;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::{DateTime, TimeDelta, Utc};
use gloo_storage::Storage;
use leptos::*;
use web_sys::js_sys::Date;

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

fn get_local_time_offset() -> i32 {
    //let now_fixed_offset: DateTime<FixedOffset> = Local::now().into();

    //now_fixed_offset.offset().local_minus_utc()
    Local::now().offset().local_minus_utc()
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
fn StartTime() -> impl IntoView {
    let (start_time, set_start_time) = create_signal(Utc::now());
    if let Ok(start) = get_start_time() {
        set_start_time.set(start);
    }

    let start_time_local = move || -> DateTime<Local> { DateTime::from(start_time.get()) };

    view! {
        <input
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
    }
}

fn main() {
    mount_to_body(|| view! { <StartTime/> });
}
