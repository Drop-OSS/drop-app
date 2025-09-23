#[macro_export]
macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().unwrap_or_else(|_| panic!("Failed to lock onto {}", stringify!($mutex)))
    };
}