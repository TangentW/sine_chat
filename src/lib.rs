use log::{LevelFilter, Metadata, Record, SetLoggerError};

pub mod frame;
pub mod handler;
pub mod message;

pub async fn run_server(addr: impl tokio::net::ToSocketAddrs) -> anyhow::Result<()> {
    let _ = Logger::init();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let handler = handler::Handler::run();

    loop {
        guard::guard!(let Ok((stream, _addr)) = listener.accept().await else { continue });
        handler.connect(stream);
    }
}

// Logger

struct Logger;

static LOGGER: Logger = Logger;

impl Logger {
    fn init() -> Result<(), SetLoggerError> {
        log::set_logger(&LOGGER).map(|_| log::set_max_level(LevelFilter::Info))
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!("[{}] {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
