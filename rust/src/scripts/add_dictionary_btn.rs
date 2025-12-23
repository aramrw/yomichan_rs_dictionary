use godot::classes::{
    Control, FileDialog, IControl, Label, LineEdit, RichTextLabel, VBoxContainer,
};
use godot::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, RwLock};
use yomichan_rs::Yomichan;

#[derive(GodotClass)]
#[class(base=FileDialog)]
pub struct AddDictionaryBtn {
    #[export]
    results_container: Option<Gd<VBoxContainer>>,
    #[base]
    base: Base<Control>,
}
