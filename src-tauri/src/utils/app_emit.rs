#[macro_export]
macro_rules! app_emit {
    ($app:expr, $event:expr, $p:expr) => {
        match $app.emit($event, $p) {
            Ok(_) => (),
            Err(e) => {
                log::error!("Failed to emit event {} with error {}", $event, e);
            }
        };
    };
}