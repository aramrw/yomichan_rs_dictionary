use crate::YOMICHAN_GLOBAL;
use godot::classes::Os;
use godot::classes::{Control, FileDialog, IControl, Label};
use godot::prelude::*;
use std::path::PathBuf;
use yomichan_rs::Yomichan;

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

        let lock = YOMICHAN_GLOBAL.read();
        if let Some(y) = lock.as_ref() {
            let cp = y.options().read().get_current_profile().unwrap();
            let mut cp = cp.write();
            let dicts = cp.dictionaries_mut();
            let dicts_len = dicts.len();
            for dict in dicts {
                dict.1.enabled = true;
            }
            label.set_text(&format!("Status: Active\nDictionaries Loaded: {dicts_len}"));
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

        // Lazy init if null
        if lock.is_none() {
            let user_path = PathBuf::from(Os::singleton().get_user_data_dir().to_string());
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
