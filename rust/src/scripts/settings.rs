use godot::classes::{Control, IControl};
use godot::prelude::*;
use crate::YOMICHAN_GLOBAL;

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
