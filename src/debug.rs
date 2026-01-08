#[cfg(feature = "debug_draw")]
#[macro_export]
macro_rules! debug_draw {
    ($expr:expr) => {
        $expr
    };
}

#[cfg(not(feature = "debug_draw"))]
#[macro_export]
macro_rules! debug_draw {
    ($expr:expr) => {};
}
