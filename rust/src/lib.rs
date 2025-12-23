mod scripts;
use godot::classes::{
    Control, FileDialog, IControl, Label, LineEdit, RichTextLabel, VBoxContainer,
};
use godot::prelude::*;
use parking_lot::RwLock; // using parking_lot
use std::path::PathBuf;
use std::sync::LazyLock;
use yomichan_rs::{parking_lot, Yomichan};

// --- 0. ENTRY POINT ---
struct YomichanRustDictionary;
#[gdextension]
unsafe impl ExtensionLibrary for YomichanRustDictionary {}

// --- 1. GLOBAL STATE MANAGER ---
static YOMICHAN_GLOBAL: LazyLock<RwLock<Option<Yomichan>>> = LazyLock::new(|| {
    let user_path = godot::classes::Os::singleton()
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

        for mut child in container.get_children().iter_shared() {
            child.queue_free();
        }

        // parking_lot: write() returns the guard directly, no unwrap()
        let mut lock = YOMICHAN_GLOBAL.write();

        if let Some(yomichan) = lock.as_mut() {
            let results = yomichan.search(&text.to_string());

            let mut label = RichTextLabel::new_alloc();
            label.set_text(&format!("{:#?}", results));
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
    #[export]
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

        // parking_lot: read() returns the guard directly
        let lock = YOMICHAN_GLOBAL.read();
        
        if let Some(y) = lock.as_ref() {
            let count = y.dictionary_summaries().map(|s| s.len()).unwrap_or(0);
            label.set_text(&format!("Status: Active\nDictionaries Loaded: {}", count));
        } else {
            label.set_text("Status: Not Initialized (Files missing?)");
        }
    }

    #[func]
    fn on_add_btn_pressed(&mut self) {
        if let Some(fd) = &mut self.file_dialog {
            fd.popup_centered();
        } else {
            godot_error!("FileDialog not linked in Inspector!");
        }
    }

    #[func]
    fn on_file_dialog_selected(&mut self, path: GString) {
        let path_str = path.to_string();
        godot_print!("Importing dictionary from: {}", path_str);

        let mut lock = YOMICHAN_GLOBAL.write();

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

        drop(lock); 
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
        let mut lock = YOMICHAN_GLOBAL.write();
        if let Some(y) = lock.as_mut() {
            let _ = y.set_language(lang_code);
            let _ = y.update_options();
            godot_print!("Language changed to: {}", lang_code);
        }
    }
}
