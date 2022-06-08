use cfg_if::cfg_if;
use log::LevelFilter;

fn main() {
    env_logger::Builder::new().filter(Some("ray_tracing"), LevelFilter::Info).init();
    cfg_if! {
        if #[cfg(feature = "cli")] {
            use ray_tracing::cli::image_mode;
            image_mode();
        } else {
            use ray_tracing::run;
            pollster::block_on(run());
        }
    }
}
