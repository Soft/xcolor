/// A simple trait which makes it easy to ensure a given type is odd.
pub trait EnsureOdd {
    fn ensure_odd(self) -> Self;
}

macro_rules! impl_ensure_odd {
    ($type:ident) => {
        impl EnsureOdd for $type {
            /// Returns `self` if `self` is odd, otherwise returns `self + 1`
            fn ensure_odd(self) -> Self {
                if self % 2 == 0 {
                    self + 1
                } else {
                    self
                }
            }
        }
    };
}

impl_ensure_odd!(u16);
impl_ensure_odd!(u32);
impl_ensure_odd!(isize);
impl_ensure_odd!(usize);
