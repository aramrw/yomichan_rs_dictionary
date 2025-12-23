use godot::classes::{
    Control, FileDialog, IControl, Label, LineEdit, RichTextLabel, VBoxContainer,
};
use godot::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, RwLock};
use yomichan_rs::Yomichan;

// --- 1. GLOBAL STATE MANAGER ---
// Single source of truth for the dictionary backend.
static YOMICHAN_GLOBAL: LazyLock<Arc<RwLock<Option<Yomichan>>>> = LazyLock::new(|| {
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
            Arc::new(RwLock::new(Some(y)))
        }
        Err(e) => {
            godot_warn!("Yomichan init failed (ignore if first run): {}", e);
            Arc::new(RwLock::new(None))
        }
    }
});

// --- 2. SCREEN 1: SEARCH ---
#[derive(GodotClass)]
#[class(base=Control)]
pub struct SearchScreen {
    #[export]
    input: Option<Gd<LineEdit>>,
    #[export]
    results_container: Option<Gd<VBoxContainer>>,
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
        // Auto-connect text_submitted signal
        let callback = self.base().callable("perform_search");
        if let Some(input) = &mut self.input {
            if !input.is_connected("text_submitted", &callback) {
                input.connect("text_submitted", &callback);
            }
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

            // Render results
            let mut label = RichTextLabel::new_alloc();
            label.set_text(&format!("{:#?}", results)); // Debug view
            label.set_fit_content(true);
            container.add_child(&label);
        } else {
            let mut label = Label::new_alloc();
            label.set_text("Dictionary backend not loaded. Check 'Dictionaries' tab.");
            container.add_child(&label);
        }
    }
}

// --- 3. SCREEN 2: DICTIONARIES ---
#[derive(GodotClass)]
#[class(base=Control)]
pub struct DictionariesScreen {
    #[export]
    status_label: Option<Gd<Label>>,
    #[export] // <--- LINK THIS IN GODOT
    file_dialog: Option<Gd<FileDialog>>,
    #[base]
    base: Base<Control>,
}

#[godot_api]
impl IControl for DictionariesScreen {
    fn init(base: Base<Control>) -> Self {
        Self {
            status_label: None,
            file_dialog: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.refresh_status();

        // Connect the file dialog "dir_selected" signal
        let on_dir_selected = self.base().callable("on_file_dialog_selected");
        if let Some(fd) = &mut self.file_dialog {
            if !fd.is_connected("dir_selected", &on_dir_selected) {
                fd.connect("dir_selected", &on_dir_selected);
            }
        }
    }
}

#[godot_api]
impl DictionariesScreen {
    #[func]
    fn refresh_status(&mut self) {
        let Some(label) = &mut self.status_label else {
            return;
        };

        let lock = YOMICHAN_GLOBAL.read().unwrap();
        if let Some(y) = lock.as_ref() {
            let count = y.dictionary_summaries().map(|s| s.len()).unwrap_or(0);
            label.set_text(&format!("Status: Active\nDictionaries Loaded: {}", count));
        } else {
            label.set_text("Status: Not Initialized (Files missing?)");
        }
    }

    // Called by the "Add Dictionary" Button
    #[func]
    fn on_add_btn_pressed(&mut self) {
        if let Some(fd) = &mut self.file_dialog {
            fd.popup_centered();
        } else {
            godot_error!("FileDialog not linked in Inspector!");
        }
    }

    // Called when the user picks a folder
    #[func]
    fn on_file_dialog_selected(&mut self, path: GString) {
        let path_str = path.to_string();
        godot_print!("Importing dictionary from: {}", path_str);

        let mut lock = YOMICHAN_GLOBAL.write().unwrap();

        // If backend isn't loaded, try to load/create it
        if lock.is_none() {
            let user_path = PathBuf::from(
                godot::classes::Os::singleton()
                    .get_user_data_dir()
                    .to_string(),
            );
            *lock = Yomichan::new(user_path).ok();
        }

        if let Some(y) = lock.as_mut() {
            match y.import_dictionaries(&[&path_str]) {
                Ok(_) => godot_print!("Import Success!"),
                Err(e) => godot_warn!("Import Failed: {}", e),
            }
        }

        drop(lock); // Release lock before UI update
        self.refresh_status();
    }
}

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
