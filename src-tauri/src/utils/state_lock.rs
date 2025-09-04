#[macro_export]
macro_rules! state_lock {
    ($state:expr) => {
        $state.lock().expect("Failed to lock onto state")
    };
}