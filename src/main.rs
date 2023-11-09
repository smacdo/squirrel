use tracing_subscriber::{filter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};

fn main() {
    tracing_log::LogTracer::init().expect("failed to initialize LogTracer");

    let stdout_subscriber = tracing_subscriber::fmt().pretty().finish();
    tracing::subscriber::set_global_default(stdout_subscriber)
        .expect("failed to install stdout global tracing subscriber");

    // TODO: Configure tracing to emit INFO+ for wgpu, and DEBUG+ for squirrel

    squirrel::squirrel_main();
}
