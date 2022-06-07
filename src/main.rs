use cfg_if::cfg_if;

fn main() {
    env_logger::init();
    cfg_if! {
            if #[cfg(feature = "windowless")] {
                use ray_tracing::image_mode;
                image_mode();
            } else {
                use ray_tracing::run;
                pollster::block_on(run());
            }
        }
}
