use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, TimeDelta, Utc};
// Loaded for LocalStorage methods.
use gloo_storage::{errors::StorageError, Storage as AnyNameThatDoesNotConflictAAA};
use protobuf::Message;

use super::protos::storage::StorageMessage;

const STORAGE_KEY: &str = "storage_message_proto";

pub struct Storage {
    message: StorageMessage,
}

impl Storage {
    pub fn new() -> Self {
        let message = StorageMessage::new();
        let mut ret = Self { message };
        // It's ok to ignore first load from storage. It might not be there.
        let _ = ret.load_from_storage();
        ret
    }

    // TODO: Probably don't need this to be a function.
    fn load_from_storage(&mut self) -> Result<()> {
        let raw_storage_data: Result<Vec<u8>, _> = gloo_storage::LocalStorage::get(STORAGE_KEY);
        match raw_storage_data {
            Ok(data) => {
                let message: StorageMessage =
                    Message::parse_from_bytes(&data).context("Failed to deserialize storage")?;
                self.message = message;
                Ok(())
            }
            Err(error) => {
                if let StorageError::KeyNotFound(_) = error {
                    // The storage has never been set. So write a new one
                    // and mark it as "loaded".
                    self.save()?;
                    return Ok(());
                };
                Err(anyhow!(error))
            }
        }
    }

    pub fn get_start_interval(&self) -> Option<TimeDelta> {
        let interval = self.message.interval_seconds?;
        Some(TimeDelta::seconds(interval))
    }

    pub fn get_start_time(&self) -> Option<DateTime<Utc>> {
        let start_time_rfc3339 = self.message.start_time_rfc3339.as_ref()?;

        DateTime::parse_from_rfc3339(start_time_rfc3339)
            .map_or(None, |start_time| Some(start_time.with_timezone(&Utc)))
    }

    pub fn set_start_time(&mut self, start_time: DateTime<Utc>) -> Result<()> {
        self.message.start_time_rfc3339 = Some(start_time.to_rfc3339());
        self.save().context("Failed to save start time")
    }

    pub fn set_start_interval(&mut self, interval: TimeDelta) -> Result<()> {
        self.message.interval_seconds = Some(interval.num_seconds());
        self.save().context("Failed to save start interval")
    }

    fn save(&self) -> Result<()> {
        let message = self
            .message
            .write_to_bytes()
            .context("Failed to serialize data for storage.")?;
        gloo_storage::LocalStorage::set(STORAGE_KEY, message).context("Failed to save to storage")
    }
}
