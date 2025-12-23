use std::path::PathBuf;
use std::sync::LazyLock;

// dictionary_ui.rs (Modified)
use better_default::Default;
use godot::classes::*;
use godot::prelude::*;

use dirs::data_dir;
use yomichan_rs;
use yomichan_rs::parking_lot;
use yomichan_rs::Yomichan;

static YCD: LazyLock<parking_lot::RwLock<Yomichan>> = LazyLock::new(|| {
    let ycd = parking_lot::RwLock::new(Yomichan::new(data_dir().unwrap()).unwrap());
    ycd.write().set_language("es").unwrap();
    ycd.write().update_options().unwrap();
    ycd
});

#[derive(Clone)]
struct AppPaths {
    data: PathBuf,
    log: PathBuf,
}
impl Default for AppPaths {
    fn default() -> Self {
        let data = data_dir().expect("could not get data dir");
        Self {
            log: data.clone().join("log"),
            data,
        }
    }
}

#[derive(Clone, Default)]
struct AppData {
    paths: AppPaths,
}

#[derive(GodotClass)]
#[class(base=Control, init)]
pub struct SearchState {
    #[export_group(name = "Search")]
    #[export]
    search_input: Option<Gd<LineEdit>>,
    #[export]
    search_results: Option<Gd<VBoxContainer>>,
    #[base]
    base: Base<Control>,
}

#[godot_api]
impl IControl for SearchState {
    fn ready(&mut self) {
        let on_submit = &self.base().callable("search");
        let Some(input) = &mut self.search_input else {
            panic!("missing search input reference");
        };
        let res = input.connect("text_submitted", on_submit);
        if res != godot::global::Error::OK {
            panic!("{res:?}");
        }
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if event.is_action_pressed("ui_cancel") {
            if let Some(input) = &mut self.search_input {
                input.grab_focus();
                input.select_all();
            }
        }
    }
}

#[godot_api]
impl SearchState {
    #[func]
    fn search(&mut self, text: GString) {
        let Some(results) = &mut self.search_results else {
            panic!("search_results is None!");
        };
        let mut label = Label::new_alloc();
        let mut def = Label::new_alloc();
        label.set_text(&format!("You searched for: '{text}'"));
        def.set_text(&format!("{:?}", YCD.write().search(&text.to_string())));
        results.add_child(&label);
    }
}
