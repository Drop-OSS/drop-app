
#[macro_export]
macro_rules! offline {
    ($var:expr, $func1:expr, $func2:expr, $( $arg:expr ),* ) => {

        if crate::borrow_db_checked().settings.force_offline || $var.lock().unwrap().status == crate::AppStatus::Offline {
            $func1( $( $arg ), *)
        } else {
            $func2( $( $arg ), *)
        }
    }
}