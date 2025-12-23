use godot::classes::{Control, IControl, InputEvent, Label, LineEdit, Os, VBoxContainer};
use godot::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, RwLock}; // Use Arc<RwLock> for statics
use yomichan_rs::Yomichan;

struct ProairGame;

#[gdextension]
unsafe impl ExtensionLibrary for ProairGame {}

// We wrap in Option so the game doesn't crash if files are missing
static YCD: LazyLock<Arc<RwLock<Option<Yomichan>>>> = LazyLock::new(|| {
    // 1. GET GODOT USER PATH (Works on Android/iOS/PC)
    let user_dir = Os::singleton().get_user_data_dir().to_string();
    let path = PathBuf::from(user_dir);

    godot_print!("Attempting to load dictionary from: {:?}", path);

    // 2. SAFETY CHECK (No unwrap)
    match Yomichan::new(path.clone()) {
        Ok(mut y) => {
            // Log errors instead of crashing if settings fail
            if let Err(e) = y.set_language("es") {
                godot_warn!("Failed to set lang: {}", e);
            }
            if let Err(e) = y.update_options() {
                godot_warn!("Failed to update options: {}", e);
            }
            Arc::new(RwLock::new(Some(y)))
        }
        Err(e) => {
            godot_error!(
                "Failed to initialize Yomichan: {}. Ensure files are in {:?}",
                e,
                path
            );
            Arc::new(RwLock::new(None))
        }
    }
});

#[derive(GodotClass)]
#[class(base=Control)] // Removed init, usually better to use default for Control
pub struct SearchState {
    #[export]
    search_input: Option<Gd<LineEdit>>,
    #[export]
    search_results: Option<Gd<VBoxContainer>>,
    #[base]
    base: Base<Control>,
}

#[godot_api]
impl IControl for SearchState {
    fn init(base: Base<Control>) -> Self {
        let mut search_input = LineEdit::new_alloc();
        search_input.set_text("testing");
        Self {
            search_input: Some(search_input),
            search_results: None,
            base,
        }
    }

    fn ready(&mut self) {
        let on_submit = self.base().callable("search");

        if let Some(input) = &mut self.search_input {
            input.connect("text_submitted", &on_submit);
            input.grab_focus();
        } else {
            godot_error!("Search Input is not linked in the Godot Editor!");
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
        godot_error!("its all corrupted!!! game over!!!!");
        let Some(results) = &mut self.search_results else {
            godot_error!("search_results container is missing!");
            return;
        };

        // 3. CLEAR PREVIOUS RESULTS
        for mut child in results.get_children().iter_shared() {
            child.queue_free();
        }

        let mut label_query = Label::new_alloc();
        let mut label_def = Label::new_alloc();

        label_query.set_text(&format!("Query: '{}'", text));
        results.add_child(&label_query);

        // 4. PERFORM SAFE SEARCH
        let mut lock = YCD.write().unwrap();

        if let Some(yomichan) = lock.as_mut() {
            let search_res = yomichan.search(&text.to_string());

            // Format the output specifically for reading
            let output_text = format!("{:#?}", search_res);
            label_def.set_text(&output_text);
            label_def.set_autowrap_mode(godot::classes::text_server::AutowrapMode::WORD_SMART);
        } else {
            label_def.set_text(
                "Dictionary not found.\nPlease copy dictionary files to the user data folder.",
            );
        }

        // 5. ACTUALLY ADD THE DEFINITION TO TREE
        results.add_child(&label_def);
    }
}
