extern crate jni;
extern crate tokenizers;

/// Takes a long ptr argument and reinterpret it as (&mut T) instance
#[inline]
pub unsafe fn reinterpret_cast<T>(ptr: i64) -> &'static mut T{
    &mut *(ptr as *mut T)
}