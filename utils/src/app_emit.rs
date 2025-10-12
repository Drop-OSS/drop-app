#[macro_export]
macro_rules! app_emit {
    ($app:expr, $event:expr, $p:expr) => {
        ::tauri::Emitter::emit($app, $event, $p)
            .expect(&format!("Failed to emit event {}", $event));
    };
}
