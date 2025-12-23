use godot::obj::Gd;

trait InternalExtGodot {
    fn has_method_ok() {

    }
}

impl<T: godot::prelude::GodotClass> InternalExtGodot for Gd<T> {}
