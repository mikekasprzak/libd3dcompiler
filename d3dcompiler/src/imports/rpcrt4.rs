use super::*;

import_fn! {
    fn UuidCreate(Uuid: *mut [u8; 16]) -> i32 {
        trace_call!("rpcrt4!UuidCreate");
        let uuid = uuid::Uuid::new_v4();
        (*Uuid).copy_from_slice(uuid.as_bytes());
        0 // RPC_S_OK
    }
}
