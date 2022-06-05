use ray_tracing::{image_mode, run};

fn main() {
    env_logger::init();
    if let Some("once") = option_env!("MODE") {
        image_mode();
    } else {
        pollster::block_on(run());
    }
}
