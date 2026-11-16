use std::alloc::{AllocError, Allocator};
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use crate::{ErrorCode, Stream, hip_describe_err};

use crate::bindings::hipError_t;
use crate::bindings::{hipDeviceSynchronize, hipFree, hipMalloc, hipMallocManaged, hipMemAdvise, hipMemAdvise_v2, hipMemAttachGlobal, hipMemAttachHost, hipMemLocation, hipMemPrefetchAsync_v2, hipModuleLoadData};



pub mod ptr {
    use crate::bindings::{hipError_t, hipMemcpy, hipMemcpyKind};
    use crate::ErrorCode;

    pub unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize, kind: hipMemcpyKind) -> Result<(), ErrorCode> {
        let err = unsafe { hipMemcpy(dst as _, src as _, count * size_of::<T>(), kind) };
        if err != hipError_t::hipSuccess {
            return Err(err)
        }

        return Ok(())
    }
}







#[derive(Clone, Copy, Default)]
pub struct HIPAllocator;
impl HIPAllocator {
    pub fn new() -> HIPAllocator {
        HIPAllocator
    }
}

unsafe impl Allocator for HIPAllocator {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let mut ptr: *mut u8 = null_mut();
        let result = unsafe { hipMalloc(&raw mut ptr as _, layout.size()) };

        if result != hipError_t::hipSuccess {
            println!("Failed to allocate with hipMallocManaged: {:#?}", hip_describe_err(result))
        }

        if ptr.is_null() {
            return Err(AllocError)
        }

        let slice = unsafe { std::slice::from_raw_parts(ptr, layout.size()) };
        Ok(NonNull::from_ref(slice))
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        _ = unsafe { hipFree(ptr.as_ptr() as _) }
    }
}

#[derive(Clone, Copy, Default)]
pub struct HIPManagedAllocator();
impl HIPManagedAllocator {
    pub fn new() -> HIPManagedAllocator {
        HIPManagedAllocator()
    }
}

unsafe impl Allocator for HIPManagedAllocator {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let mut ptr: *mut u8 = null_mut();
        let result = unsafe { hipMallocManaged(&raw mut ptr as _, layout.size(), hipMemAttachHost) };

        if result != hipError_t::hipSuccess {
            println!("Failed to allocate with hipMallocManaged: {:#?}", hip_describe_err(result))
        }

        if ptr.is_null() {
            return Err(AllocError)
        }

        let slice = unsafe { std::slice::from_raw_parts(ptr, layout.size()) };
        Ok(NonNull::from_ref(slice))
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        _ = unsafe { hipFree(ptr.as_ptr() as _) }
    }
}

///////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////// Advise ///////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

pub use crate::bindings::hipMemoryAdvise as HIPMemoryAdvice;
pub trait HIPMemAdvise {
    fn advise(&self, advice: HIPMemoryAdvice, location: HIPMemoryLocation);
}

impl<T> HIPMemAdvise for *const T {
    fn advise(&self, advice: HIPMemoryAdvice, location: HIPMemoryLocation) {
        unsafe {
            hipMemAdvise_v2(*self as _, size_of::<T>(), advice, location);
        }
    }
}

impl<T> HIPMemAdvise for *mut T {
    fn advise(&self, advice: HIPMemoryAdvice, location: HIPMemoryLocation) {
        unsafe {
            hipMemAdvise_v2(*self as _, size_of::<T>(), advice, location);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////// Prefetch /////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////

pub use crate::bindings::hipMemLocation as HIPMemoryLocation;
pub trait HIPMemPrefetch<T: ?Sized> {
    fn prefetch(&self, location: HIPMemoryLocation) -> Result<(), ErrorCode>;
    fn prefetch_ext(&self, location: HIPMemoryLocation, stream: Stream) -> Result<Stream, ErrorCode>;
}

pub trait HIPMemPrefetchRaw {
    fn prefetch(&self, location: HIPMemoryLocation) -> Result<(), ErrorCode>;
    fn prefetch_ext(&self, location: HIPMemoryLocation, stream: Stream) -> Result<Stream, ErrorCode>;
}

pub trait HIPMemPrefetchSize {
    fn prefetch_size(&self, location: HIPMemoryLocation, size: usize) -> Result<(), ErrorCode>;
    fn prefetch_size_ext(&self, location: HIPMemoryLocation, size: usize, stream: Stream) -> Result<Stream, ErrorCode>;
}

impl<T: ?Sized, U: Deref<Target = T>>  HIPMemPrefetch<T> for U {
    fn prefetch(&self, location: HIPMemoryLocation) -> Result<(), ErrorCode> {
        self.prefetch_ext(location, Stream(null_mut())).map(|_| {})
    }

    fn prefetch_ext(&self, location: HIPMemoryLocation, stream: Stream) -> Result<Stream, ErrorCode> {
        let err = unsafe { hipMemPrefetchAsync_v2(&**self as *const T as _, core::mem::size_of_val(&**self), location, 0, stream.0) };
        
        if err != hipError_t::hipSuccess {
            return Err(err)
        }

        return Ok(stream)
    }
}

// impl<T: ?Sized> HIPMemPrefetch for &mut T {
//     fn prefetch(&self, location: HIPMemoryLocation) -> Result<(), ErrorCode> {
//         self.prefetch_ext(location, Stream(null_mut())).map(|_| {})
//     }

//     fn prefetch_ext(&self, location: HIPMemoryLocation, stream: Stream) -> Result<Stream, ErrorCode> {
//         let err = unsafe { hipMemPrefetchAsync_v2(*self as *const T as _, core::mem::size_of_val(*self), location, 0, stream.0) };
        
//         if err != hipError_t::hipSuccess {
//             return Err(err)
//         }

//         return Ok(stream)
//     }
// }

impl<T: Sized> HIPMemPrefetchRaw for *const T {
    fn prefetch(&self, location: HIPMemoryLocation) -> Result<(), ErrorCode> {
        self.prefetch_ext(location, Stream(null_mut())).map(|_| {})
    }

    fn prefetch_ext(&self, location: HIPMemoryLocation, stream: Stream) -> Result<Stream, ErrorCode> {
        let err = unsafe { hipMemPrefetchAsync_v2(*self as *const T as _, core::mem::size_of::<T>(), location, 0, stream.0) };
        
        if err != hipError_t::hipSuccess {
            return Err(err)
        }

        return Ok(stream)
    }
}

impl<T: Sized> HIPMemPrefetchRaw for *mut T {
    fn prefetch(&self, location: HIPMemoryLocation) -> Result<(), ErrorCode> {
        self.prefetch_ext(location, Stream(null_mut())).map(|_| {})
    }

    fn prefetch_ext(&self, location: HIPMemoryLocation, stream: Stream) -> Result<Stream, ErrorCode> {
        let err = unsafe { hipMemPrefetchAsync_v2(*self as *const T as _, core::mem::size_of::<T>(), location, 0, stream.0) };
        
        if err != hipError_t::hipSuccess {
            return Err(err)
        }

        return Ok(stream)
    }
}

impl<T: ?Sized> HIPMemPrefetchSize for *const T {
    fn prefetch_size(&self, location: HIPMemoryLocation, size: usize) -> Result<(), ErrorCode> {
        self.prefetch_size_ext(location, size, Stream(null_mut())).map(|_| {})
    }

    fn prefetch_size_ext(&self, location: HIPMemoryLocation, size: usize, stream: Stream) -> Result<Stream, ErrorCode> {
        let err = unsafe { hipMemPrefetchAsync_v2(*self as *const T as _, size, location, 0, stream.0) };
        
        if err != hipError_t::hipSuccess {
            return Err(err)
        }

        return Ok(stream)
    }
}

impl<T: ?Sized> HIPMemPrefetchSize for *mut T {
    fn prefetch_size(&self, location: HIPMemoryLocation, size: usize) -> Result<(), ErrorCode> {
        self.prefetch_size_ext(location, size, Stream(null_mut())).map(|_| {})
    }

    fn prefetch_size_ext(&self, location: HIPMemoryLocation, size: usize, stream: Stream) -> Result<Stream, ErrorCode> {
        let err = unsafe { hipMemPrefetchAsync_v2(*self as *const T as _, size, location, 0, stream.0) };
        
        if err != hipError_t::hipSuccess {
            return Err(err)
        }

        return Ok(stream)
    }
}