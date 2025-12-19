use std::ffi::c_void;

/// Core vsnprintf implementation - formats a string with varargs
///
/// # Safety
/// - `buffer` must point to valid memory of at least `count` bytes
/// - `format` must be a valid null-terminated C string
/// - `argptr` must point to valid sequential u64 arguments matching format specifiers
pub unsafe fn vsnprintf_core(
    buffer: *mut i8,
    count: usize,
    format: *const i8,
    argptr: *const u64,
) -> i32 {
    if buffer.is_null() || count == 0 {
        return -1;
    }

    let mut arg_ptr = argptr;
    let mut written: usize = 0;
    let max_write = count - 1;
    let mut i: usize = 0;
    let fmt_len = libc::strlen(format) as usize;

    while i < fmt_len && written < max_write {
        let c = *format.add(i) as u8;

        if c != b'%' {
            *buffer.add(written) = c as i8;
            written += 1;
            i += 1;
            continue;
        }

        i += 1;
        if i >= fmt_len {
            break;
        }

        // Skip flags: -, +, space, #, 0
        while i < fmt_len {
            let fc = *format.add(i) as u8;
            if fc == b'-' || fc == b'+' || fc == b' ' || fc == b'#' || fc == b'0' {
                i += 1;
            } else {
                break;
            }
        }

        // Skip width (digits or *)
        if i < fmt_len && *format.add(i) as u8 == b'*' {
            arg_ptr = arg_ptr.add(1); // consume width arg
            i += 1;
        } else {
            while i < fmt_len && (*format.add(i) as u8).is_ascii_digit() {
                i += 1;
            }
        }

        // Skip precision (.digits or .*)
        if i < fmt_len && *format.add(i) as u8 == b'.' {
            i += 1;
            if i < fmt_len && *format.add(i) as u8 == b'*' {
                arg_ptr = arg_ptr.add(1); // consume precision arg
                i += 1;
            } else {
                while i < fmt_len && (*format.add(i) as u8).is_ascii_digit() {
                    i += 1;
                }
            }
        }

        // Skip length modifiers: h, hh, l, ll, L, z, j, t, I, I32, I64
        while i < fmt_len {
            let lc = *format.add(i) as u8;
            if lc == b'h'
                || lc == b'l'
                || lc == b'L'
                || lc == b'z'
                || lc == b'j'
                || lc == b't'
                || lc == b'I'
            {
                i += 1;
            } else if lc.is_ascii_digit() && i > 0 && *format.add(i - 1) as u8 == b'I' {
                i += 1; // I32, I64
            } else {
                break;
            }
        }

        if i >= fmt_len {
            break;
        }

        let spec = *format.add(i) as u8;
        i += 1;

        match spec {
            b'%' => {
                *buffer.add(written) = b'%' as i8;
                written += 1;
            }
            b's' => {
                let s = *arg_ptr as *const i8;
                arg_ptr = arg_ptr.add(1);
                if !s.is_null() {
                    let slen = libc::strlen(s) as usize;
                    let copy_len = std::cmp::min(slen, max_write - written);
                    libc::memcpy(
                        buffer.add(written) as *mut c_void,
                        s as *const c_void,
                        copy_len,
                    );
                    written += copy_len;
                }
            }
            b'd' | b'i' => {
                let val = *arg_ptr as i64;
                arg_ptr = arg_ptr.add(1);
                let s = format!("{}", val);
                let copy_len = std::cmp::min(s.len(), max_write - written);
                libc::memcpy(
                    buffer.add(written) as *mut c_void,
                    s.as_ptr() as *const c_void,
                    copy_len,
                );
                written += copy_len;
            }
            b'u' => {
                let val = *arg_ptr;
                arg_ptr = arg_ptr.add(1);
                let s = format!("{}", val);
                let copy_len = std::cmp::min(s.len(), max_write - written);
                libc::memcpy(
                    buffer.add(written) as *mut c_void,
                    s.as_ptr() as *const c_void,
                    copy_len,
                );
                written += copy_len;
            }
            b'x' => {
                let val = *arg_ptr;
                arg_ptr = arg_ptr.add(1);
                let s = format!("{:x}", val);
                let copy_len = std::cmp::min(s.len(), max_write - written);
                libc::memcpy(
                    buffer.add(written) as *mut c_void,
                    s.as_ptr() as *const c_void,
                    copy_len,
                );
                written += copy_len;
            }
            b'X' => {
                let val = *arg_ptr;
                arg_ptr = arg_ptr.add(1);
                let s = format!("{:X}", val);
                let copy_len = std::cmp::min(s.len(), max_write - written);
                libc::memcpy(
                    buffer.add(written) as *mut c_void,
                    s.as_ptr() as *const c_void,
                    copy_len,
                );
                written += copy_len;
            }
            b'p' => {
                let val = *arg_ptr;
                arg_ptr = arg_ptr.add(1);
                let s = format!("{:016X}", val);
                let copy_len = std::cmp::min(s.len(), max_write - written);
                libc::memcpy(
                    buffer.add(written) as *mut c_void,
                    s.as_ptr() as *const c_void,
                    copy_len,
                );
                written += copy_len;
            }
            b'c' => {
                let val = (*arg_ptr & 0xFF) as u8;
                arg_ptr = arg_ptr.add(1);
                *buffer.add(written) = val as i8;
                written += 1;
            }
            b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => {
                let val = f64::from_bits(*arg_ptr);
                arg_ptr = arg_ptr.add(1);
                let s = format!("{}", val);
                let copy_len = std::cmp::min(s.len(), max_write - written);
                libc::memcpy(
                    buffer.add(written) as *mut c_void,
                    s.as_ptr() as *const c_void,
                    copy_len,
                );
                written += copy_len;
            }
            _ => {
                // Unknown specifier, skip
            }
        }
    }

    *buffer.add(written) = 0;
    written as i32
}
