use wasm_bindgen::prelude::wasm_bindgen;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}



// Serializable replacement for Vec<T> that implements From/Into for the element type
pub struct SerVec<T>(Vec<T>);
impl<T, S> From<Vec<T>> for SerVec<S> where T: Into<S> {
    fn from(val: Vec<T>) -> Self {
        SerVec(val.into_iter().map(Into::into).collect())
    }
}
impl<T, S> From<SerVec<S>> for Vec<T> where S: Into<T> {
    fn from(val: SerVec<S>) -> Self {
        val.0.into_iter().map(Into::into).collect()
    }
}
impl<T> serde::Serialize for SerVec<T> where T: serde::Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        self.0.serialize(serializer)
    }
}
impl<'de, T> serde::Deserialize<'de> for SerVec<T> where T: serde::Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        Vec::deserialize(deserializer).map(|x| SerVec(x))
    }
}


// Print to browser console
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (utils::log(&format_args!($($t)*).to_string()))
}
