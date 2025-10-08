#[macro_export]
macro_rules! send {
    ($download_manager:expr, $signal:expr) => {
        $download_manager.send($signal).unwrap_or_else(|_| panic!("Failed to send signal {} to the download manager", stringify!(signal)))
    };
}