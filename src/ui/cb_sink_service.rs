use anyhow::{anyhow, Result};
use cursive::CbSink;
use std::sync::OnceLock;

pub struct CbSinkService {}

static INSTANCE: OnceLock<CbSink> = OnceLock::new();

impl CbSinkService {
    pub fn instance() -> Result<&'static CbSink> {
        INSTANCE
            .get()
            .ok_or_else(|| anyhow!("Error! No CbSink instance to get yet!"))
    }

    pub fn set_instance(cb_sink: CbSink) -> Result<()> {
        INSTANCE.set(cb_sink).map_err(|_| {
            anyhow!("Error setting the global CbSink instance! Shit's probably fucked!")
        })?;
        Ok(())
    }
}
