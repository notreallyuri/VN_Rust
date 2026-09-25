use crate::run_json;

#[unsafe(no_mangle)]
pub extern "C" fn novn_alloc(len: usize) -> *mut u8 {
    let buffer = vec![0u8; len].into_boxed_slice();
    Box::into_raw(buffer).cast::<u8>()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn novn_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    drop(unsafe { Box::from_raw(slice) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn novn_run(ptr: *const u8, len: usize) -> *mut u8 {
    let request = unsafe { std::slice::from_raw_parts(ptr, len) };
    let answer = run_json(std::str::from_utf8(request).unwrap_or("{}"));

    let bytes = answer.as_bytes();
    let mut framed = Vec::with_capacity(4 + bytes.len());
    framed.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    framed.extend_from_slice(bytes);

    Box::into_raw(framed.into_boxed_slice()).cast::<u8>()
}
