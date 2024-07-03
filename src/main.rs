use tracing::info;

fn main() {
    pollster::block_on(squirrel::run_main())
}
