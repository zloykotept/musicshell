#[macro_export]
macro_rules! get_action {
    ($name:expr) => {
        $crate::workspace::Action::$name
    };
}
