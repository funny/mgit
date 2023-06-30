use crate::utils::style_message::StyleMessage;

static mut LOGGER: &dyn Log = &NopLogger;

pub(crate) fn info(message: impl Into<StyleMessage>) {
    unsafe {
        LOGGER.info(message.into());
    }
}

pub(crate) fn error(message: impl Into<StyleMessage>) {
    unsafe {
        LOGGER.error(message.into());
    }
}

pub fn set_logger(logger: &'static dyn Log) {
    unsafe {
        LOGGER = logger;
    }
}

pub trait Log: Send + Sync {
    fn info(&self, message: StyleMessage);

    fn error(&self, message: StyleMessage);
}

pub(crate) struct NopLogger;

impl Log for NopLogger {
    fn info(&self, _: StyleMessage) {}

    fn error(&self, _: StyleMessage) {}
}
