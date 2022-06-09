use cfg_if::cfg_if;

const STATIC_PATH: &str = "static";
#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&format!(
        "{}/{}/",
        location.origin().unwrap(),
        option_env!("RES_PATH").unwrap_or(STATIC_PATH),
    ))
        .unwrap();
    base.join(file_name).unwrap()
}

pub fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            log::info!("URL: {:?}", path);
            let data = reqwest::get(url)?
                .bytes()?
                .to_vec();
        } else {
            let path = std::path::Path::new(option_env!("OUT_DIR").unwrap_or("."))
                .join(STATIC_PATH)
                .join(file_name);
            log::info!("Texture path: {:?}", path);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}