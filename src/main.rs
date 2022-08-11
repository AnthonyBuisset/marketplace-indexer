use dotenv::dotenv;
use slog::{o, Drain, Logger};

fn get_root_logger() -> Logger {
	let drain = match std::env::var("LOGS") {
		Ok(logs) if logs == *"terminal" => slog_async::Async::default(slog_envlogger::new(
			slog_term::CompactFormat::new(slog_term::TermDecorator::new().stderr().build())
				.build()
				.fuse(),
		)),
		_ => slog_async::Async::default(slog_envlogger::new(
			slog_json::Json::new(std::io::stdout()).add_default_keys().build().fuse(),
		)),
	};
	slog_stdlog::init().unwrap();
	slog::Logger::root(drain.fuse(), o!("version" => env!("CARGO_PKG_VERSION")))
}

#[tokio::main]
async fn main() {
	dotenv().ok();
	let root_logger = get_root_logger();
	let _global_logger_guard = slog_scope::set_global_logger(root_logger);
}
