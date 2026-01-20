use std::path::PathBuf;
use std::sync::OnceLock;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub(crate) fn log_dir() -> &'static PathBuf {
    static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();
    LOG_DIR.get_or_init(|| {
        home::home_dir()
            .expect("Failed to get home directory")
            .join(".mgit")
            .join("logs")
    })
}

pub(crate) fn init_log() -> WorkerGuard {
    let log_path = log_dir();

    std::fs::create_dir_all(log_path).expect("Failed to create log directory");

    let log_file = log_path.join("log.txt");
    let _ = std::fs::File::create(&log_file);
    let file_appender = tracing_appender::rolling::never(log_path, "log.txt");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_file(true)
                .with_line_number(true),
        )
        .with(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    tracing::info!(
        log_file = log_file.to_string_lossy().as_ref(),
        "logger_initialized"
    );

    guard
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
