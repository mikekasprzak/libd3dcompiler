use super::*;

#[repr(C)]
#[allow(clippy::upper_case_acronyms)]
pub struct CONTEXT {
    data: [u8; 1232], // Full x64 CONTEXT structure
}

#[repr(C)]
pub struct RUNTIME_FUNCTION {
    pub BeginAddress: u32,
    pub EndAddress: u32,
    pub UnwindData: u32,
}

import_fn! {
    fn RtlCaptureContext(context: *mut CONTEXT) {
        trace_call!("ntdll!RtlCaptureContext");
        if !context.is_null() {
            std::ptr::write_bytes(context, 0, 1);
        }
    }

    fn RtlLookupFunctionEntry(
        _pc: u64,
        _image_base: *mut u64,
        _history_table: *mut c_void,
    ) -> *mut RUNTIME_FUNCTION {
        trace_call!("ntdll!RtlLookupFunctionEntry", "pc=0x{:x}", _pc);
        panic!("ntdll!RtlLookupFunctionEntry not implemented");
    }

    fn RtlVirtualUnwind(
        _handler_type: u32,
        _image_base: u64,
        _control_pc: u64,
        _function_entry: *mut RUNTIME_FUNCTION,
        _context: *mut CONTEXT,
        _handler_data: *mut *mut c_void,
        _establisher_frame: *mut u64,
        _context_pointers: *mut c_void,
    ) -> *mut c_void {
        trace_call!("ntdll!RtlVirtualUnwind");
        panic!("ntdll!RtlVirtualUnwind not implemented");
    }

    fn RtlUnwindEx(
        _target_frame: *mut c_void,
        _target_ip: *mut c_void,
        _exception_record: *mut c_void,
        _return_value: *mut c_void,
        _context: *mut CONTEXT,
        _history_table: *mut c_void,
    ) {
        trace_call!("ntdll!RtlUnwindEx");
        std::process::abort();
    }
}
