
#[macro_export]
macro_rules! offline {
    ($var:expr, $func1:expr, $func2:expr, $( $arg:expr ),* ) => {
        if borrow_db_checked().settings.offline || state.{
            $func1( $( $arg ), *)
        } else {
            $func2( $( $arg ), *)
        }
    };
}