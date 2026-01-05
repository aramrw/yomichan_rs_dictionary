use godot::classes::{Control, IControl, Label, LineEdit, RichTextLabel, VBoxContainer};
use godot::prelude::*;
use crate::YOMICHAN_GLOBAL; // Import global from lib.rs

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
            godot_error!("Results container not linked.");
            return;
        };

        for mut child in container.get_children().iter_shared() {
            child.queue_free();
        }

        let mut lock = YOMICHAN_GLOBAL.write();

        if let Some(yomichan) = lock.as_mut() {
            if let Some(results) = yomichan.search(&text.to_string()) {
                let mut result_text = String::new();

                for segment in results {
                    if let Some(segment_results) = segment.results {
                        for entry in &segment_results.dictionary_entries {
                            result_text.push_str(&format!("{}\n", entry.get_headword_text_joined()));
                            
                            // FIXED: Using .definitions here
                            for (i, def) in entry.definitions.iter().enumerate() {
                                result_text.push_str(&format!("{}. {:?}\n", i + 1, def));
                            }
                            result_text.push_str("\n");
                        }
                    }
                }

                if result_text.is_empty() {
                    result_text = "nothing found".to_string();
                }

                let mut label = RichTextLabel::new_alloc();
                label.set_text(&result_text);
                label.set_fit_content(true);
                container.add_child(&label);
            } else {
                let mut label = Label::new_alloc();
                label.set_text("No results found.");
                container.add_child(&label);
            }
        } else {
            let mut label = Label::new_alloc();
            label.set_text("Dictionary backend not loaded. Check 'Dictionaries' tab.");
            container.add_child(&label);
        }
    }
}
