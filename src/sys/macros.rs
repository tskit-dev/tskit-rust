#![macro_use]

macro_rules! impl_tskteardown {
    ($tsk: ty, $teardown: expr) => {
        impl super::TskTeardown for $tsk {
            unsafe fn teardown(&mut self) -> i32 {
                $teardown(self as _)
            }
        }
    };
}
