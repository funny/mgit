use mgit::utils::logger;
use mgit::utils::style_message::StyleMessage;

pub(crate) static TERM_LOGGER: TermLogger = TermLogger {};

#[derive(Clone, Default)]
pub struct TermLogger;

impl logger::Log for TermLogger {
    fn info(&self, message: StyleMessage) {
        println!("{}", message)
    }

    fn error(&self, message: StyleMessage) {
        eprintln!("{}", message)
    }
}
