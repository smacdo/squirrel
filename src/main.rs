use tracing_log::env_logger;

fn main() {
    // Initialize logging.
    env_logger::init();

    squirrel::run_game_loop();
}
