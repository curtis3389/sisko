use cursive::CbSink;
use std::sync::OnceLock;

pub struct CbSinkService {}

static INSTANCE: OnceLock<CbSink> = OnceLock::new();

impl CbSinkService {
    pub fn instance() -> &'static CbSink {
        INSTANCE.get().unwrap()
    }

    pub fn set_instance(cb_sink: CbSink) {
        INSTANCE.set(cb_sink).unwrap();
    }
}
