mod extensions;
mod scripts;
use godot::classes::{
    Control, FileDialog, IControl, Label, LineEdit, OptionButton, RichTextLabel, VBoxContainer,
};
use godot::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, RwLock};
use yomichan_rs::Ptr;
use yomichan_rs::Yomichan;

// --- 1. GLOBAL STATE MANAGER ---
// Option so the app can start even if Yomichan fails to load initially.
static YOMICHAN_GLOBAL: LazyLock<Ptr<Option<Yomichan>>> = LazyLock::new(|| {
    let user_path = godot::classes::Os::singleton()
        .get_user_data_dir()
        .to_string();
    let path = PathBuf::from(user_path);

    godot_print!("Initializing Yomichan at: {:?}", path);

    match Yomichan::new(path.clone()) {
        Ok(mut y) => {
            // Default config
            let _ = y.set_language("es");
            let _ = y.update_options();
            Ptr::from(Some(y))
        }
        Err(e) => {
            godot_warn!("Yomichan init failed {e}");
            Ptr::from(None)
        }
    }
});

struct YomichanRustDictionary;
#[gdextension]
unsafe impl ExtensionLibrary for YomichanRustDictionary {}

// --- 2. SCREEN 1: SEARCH ---
#[derive(GodotClass)]
#[class(base=Control)]
pub struct SearchScreen {
    #[export]
    input: Option<Gd<LineEdit>>,
    #[export]
    results_container: Option<Gd<VBoxContainer>>, // Use a VBox inside a ScrollContainer
    #[base]
    base: Base<Control>,
}

#[godot_api]
impl IControl for SearchScreen {
    fn init(base: Base<Control>) -> Self {
        Self {
            input: None,
            results_container: None,
            base,
        }
    }

    fn ready(&mut self) {
        // Auto-connect the text_submitted signal if input is assigned
        let callback = self.base().callable("perform_search");
        if let Some(input) = &mut self.input {
            input.connect("text_submitted", &callback);
        }
    }
}

#[godot_api]
impl SearchScreen {
    #[func]
    fn perform_search(&mut self, text: GString) {
        let Some(container) = &mut self.results_container else {
            return;
        };

        // Clear previous results
        for mut child in container.get_children().iter_shared() {
            child.queue_free();
        }

        // Access Global State
        let mut lock = YOMICHAN_GLOBAL.write().unwrap();

        if let Some(yomichan) = lock.as_mut() {
            let results = yomichan.search(&text.to_string());

            // Render results (Simplified for now)
            let mut label = RichTextLabel::new_alloc();
            label.set_text(&format!("{:#?}", results)); // Pretty print debug for now
            label.set_fit_content(true); // Auto-expand height
            container.add_child(&label);
        } else {
            let mut label = Label::new_alloc();
            label.set_text("Dictionary backend not loaded. Check 'Dictionaries' tab.");
            container.add_child(&label);
        }
    }
}

// --- 3. SCREEN 2: DICTIONARIES ---

// --- 4. SCREEN 3: SETTINGS ---
#[derive(GodotClass)]
#[class(base=Control)]
pub struct SettingsScreen {
    #[base]
    base: Base<Control>,
}

#[godot_api]
impl IControl for SettingsScreen {
    fn init(base: Base<Control>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl SettingsScreen {
    #[func]
    fn set_lang_es(&mut self) {
        self.change_lang("es");
    }

    #[func]
    fn set_lang_en(&mut self) {
        self.change_lang("en");
    }

    fn change_lang(&self, lang_code: &str) {
        let mut lock = YOMICHAN_GLOBAL.write().unwrap();
        if let Some(y) = lock.as_mut() {
            let _ = y.set_language(lang_code);
            let _ = y.update_options();
            godot_print!("Language changed to: {}", lang_code);
        }
    }
}
