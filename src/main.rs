use ray_tracing::run;

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
