use std::{sync::{OnceLock, Once, Arc, atomic}, cell::UnsafeCell, mem::MaybeUninit, marker::PhantomData, ptr::NonNull};

use crate::{Dissectible, impl_mirror_mock};


#[derive(Dissectible)]
struct MockOnceLock<T> {
    once: Once,
    value: UnsafeCell<MaybeUninit<T>>,
    _marker: PhantomData<T>,
}
impl_mirror_mock!(with <T>: MockOnceLock<T> => OnceLock<T>);

#[repr(C)]
/// This is a dummy copy of the internal type ArcInner<T>, used as a field in Arc<T>, which is shadowed by the public dummy::ArcInner<T> and can be unsized
struct ArcInner<T> where T: ?Sized {
    strong: atomic::AtomicUsize,
    weak: atomic::AtomicUsize,
    data: T,
}

#[derive(Dissectible)]
pub struct MockArc<T> {
    ptr: NonNull<ArcInner<T>>,
    phantom: PhantomData<ArcInner<T>>,
}
impl_mirror_mock!(with <T>: MockArc<T> => Arc<T>);

pub mod dummy {
    use std::sync::atomic;

    use struct_scalpel_proc_macro::Dissectible;

    #[derive(Dissectible)]
    #[repr(C)]
    /// This is a dummy copy of the internal type ArcInner<T>, used as a field in Arc<T>
    pub struct ArcInner<T> {
        strong: atomic::AtomicUsize,
        weak: atomic::AtomicUsize,
        data: T,
    }
}