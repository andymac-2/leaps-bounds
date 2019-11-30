use crate::Context2D;

pub fn interpolate(start: f64, end: f64, proportion: f64) -> f64 {
    assert!(proportion >= 0.0 && proportion <= 1.0);
    start + ((end - start) * proportion)
}

pub fn with_saved_context<F: FnOnce()>(context: &Context2D, func: F) {
    context.save();
    func();
    context.restore();
}

pub fn clamp(value: f64, lower: f64, upper: f64) -> f64 {
    assert!(lower < upper);
    if value > upper {
        return upper;
    }
    if value < lower {
        return lower;
    }
    value
}

pub fn get_storage () -> web_sys::Storage {
    let window = web_sys::window().unwrap();
    window.local_storage().unwrap().unwrap()
}