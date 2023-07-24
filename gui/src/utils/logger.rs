use std::path::PathBuf;

use lazy_static::lazy_static;
use log::{error, info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;

use mgit::utils::logger;
use mgit::utils::style_message::StyleMessage;

pub(crate) static GUI_LOGGER: GuiLogger = GuiLogger {};
lazy_static! {
    pub(crate) static ref LOG_DIR: PathBuf = PathBuf::from("log");
}

#[derive(Clone, Default)]
pub struct GuiLogger;

impl logger::Log for GuiLogger {
    fn info(&self, message: StyleMessage) {
        info!("{}", message)
    }

    fn error(&self, message: StyleMessage) {
        error!("{}", message)
    }
}

pub(crate) fn init_log() {
    logger::set_logger(&GUI_LOGGER);
    let regular_name = "regular_out".to_string();
    let regular_out = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}{n}")))
        .build(LOG_DIR.join(&regular_name))
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build(&regular_name, Box::new(regular_out)))
        .logger(Logger::builder().build(&regular_name, LevelFilter::Info))
        .build(
            Root::builder()
                .appender(&regular_name)
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}

// pub(crate) struct LoggerBuilderWrapper {
//     config_builder: ConfigBuilder,
//     loggers: Vec<String>,
//     out_dir: PathBuf,
// }
//
// impl LoggerBuilderWrapper {
//     pub(crate) fn new() -> Self {
//         let out_dir = PathBuf::from("log");
//         let regular_name = "regular_out".to_string();
//         let regular_out = FileAppender::builder()
//             .encoder(Box::new(PatternEncoder::new("{l} - {m}{n}")))
//             .build(out_dir.join(&regular_name))
//             .unwrap();
//         Self {
//             config_builder: Config::builder()
//                 .appender(Appender::builder().build(&regular_name, Box::new(regular_out)))
//                 .logger(Logger::builder().build(&regular_name, LevelFilter::Info)),
//             loggers: vec![regular_name],
//             out_dir,
//         }
//     }
//
//     pub(crate) fn add_appender(&mut self, name: impl AsRef<str>) {
//         self.loggers.push(name.as_ref().to_string());
//         let name = format!("{}.log", name.as_ref());
//         let appender = FileAppender::builder()
//             .encoder(Box::new(PatternEncoder::new("{l} - {m}{n}")))
//             .build(self.out_dir.join(&name))
//             .unwrap();
//         self.config_builder = self
//             .config_builder
//             .appender(Appender::builder().build(&name, Box::new(appender)))
//             .logger(Logger::builder().build(&name, LevelFilter::Info));
//     }
//
//     pub(crate) fn build(&mut self) {
//         let root_builder = self
//             .loggers
//             .iter()
//             .fold(Root::builder(), |builder, logger| {
//                 builder.appender(logger);
//                 builder
//             });
//
//         let config = self
//             .config_builder
//             .build(root_builder.build(LevelFilter::Info))
//             .unwrap();
//
//         log4rs::init_config(config).unwrap();
//     }
// }
