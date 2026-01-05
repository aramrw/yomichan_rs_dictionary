use godot::prelude::*;
use std::path::PathBuf;
use std::sync::LazyLock;
use yomichan_rs::parking_lot::RwLock; // using parking_lot directly
use yomichan_rs::Yomichan;
use godot::classes::Os;

// 1. Declare Modules
pub mod scripts;
pub mod extensions;

// 2. Entry Point
struct YomichanRustDictionary;

#[gdextension]
unsafe impl ExtensionLibrary for YomichanRustDictionary {}

// 3. Global State (Public so other modules can use it)
pub static YOMICHAN_GLOBAL: LazyLock<RwLock<Option<Yomichan>>> = LazyLock::new(|| {
    let user_path = Os::singleton()
        .get_user_data_dir()
        .to_string();
    let path = PathBuf::from(user_path);

    godot_print!("Initializing Yomichan at: {:?}", path);

    match Yomichan::new(path.clone()) {
        Ok(mut y) => {
            let _ = y.set_language("es");
            let _ = y.update_options();
            RwLock::new(Some(y))
        }
        Err(e) => {
            godot_warn!("Yomichan init failed (ignore if first run): {}", e);
            RwLock::new(None)
        }
    }
});
