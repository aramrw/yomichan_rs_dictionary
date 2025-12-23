use godot::classes::FileDialog;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=FileDialog, init)]
pub struct AddDictionaryButton {
    #[base]
    base: Base<FileDialog>,
}
