use godot::classes::{
    Control, IControl, Label, LineEdit, OptionButton, RichTextLabel, VBoxContainer,
};
use godot::prelude::*;
use std::path::PathBuf;
//use std::sync::{Arc, LazyLock, RwLock};
use yomichan_rs::Yomichan;

use crate::YOMICHAN_GLOBAL;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct DictionariesScreen {
    #[export]
    status_label: Option<Gd<Label>>,
    #[export]
    import_path_input: Option<Gd<LineEdit>>,
    #[base]
    base: Base<Control>,
}

#[godot_api]
impl IControl for DictionariesScreen {
    fn init(base: Base<Control>) -> Self {
        Self {
            status_label: None,
            import_path_input: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.refresh_status();
    }
}

#[godot_api]
impl DictionariesScreen {
    #[func]
    fn refresh_status(&mut self) {
        let Some(label) = &mut self.status_label else {
            return;
        };

        let lock = YOMICHAN_GLOBAL.read();
        if let Some(y) = lock.as_ref() {
            let count = y.dictionary_summaries().map(|s| s.len()).unwrap_or(0);
            label.set_text(&format!("Status: Active\nDictionaries Loaded: {}", count));
        } else {
            label.set_text("Status: Not Initialized (Files missing?)");
        }
    }

    #[func]
    fn on_import_pressed(&mut self) {
        let path_str = self
            .import_path_input
            .as_ref()
            .map(|i| i.get_text().to_string())
            .unwrap_or_default();

        if path_str.is_empty() {
            godot_print!("No path provided");
            return;
        }

        // This requires write access to the global state
        let mut ycd = YOMICHAN_GLOBAL.write();

        // If yomichan is None, try to re-init it first or create new
        if ycd.is_none() {
            let user_path = PathBuf::from(
                godot::classes::Os::singleton()
                    .get_user_data_dir()
                    .to_string(),
            );
            *ycd = Yomichan::new(user_path).ok();
        }

        let Some(ycd) = ycd.take() else {
            return;
        };

        match ycd.import_dictionaries(&[&path_str]) {
            Ok(_) => godot_print!("Import Success!"),
            Err(e) => godot_warn!("Import Failed: {}", e),
        }

        // Drop lock before refreshing UI to avoid deadlocks
        drop(ycd);
        self.refresh_status();
    }
}
