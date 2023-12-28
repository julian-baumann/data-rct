use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use objc_foundation::{INSString, NSString};
use objc::runtime::{BOOL, NO, YES};

pub trait IntoBool {
    fn into_bool(self) -> bool;
}

impl IntoBool for BOOL {
    fn into_bool(self) -> bool {
        match self {
            YES => true,
            NO => false
        }
    }
}

impl IntoBool for *mut Object {
    fn into_bool(self) -> bool {
        let nil = 0 as *mut Object;
        nil != self
    }
}

pub trait IntoObjcBool {
    fn into_objc_bool(self) -> BOOL;
}

impl IntoObjcBool for bool {
    fn into_objc_bool(self) -> BOOL {
        if self {
            YES
        } else {
            NO
        }
    }
}

pub trait IntoCBUUID {
    fn into_cbuuid(self) -> *mut Object;
}

impl IntoCBUUID for String {
    fn into_cbuuid(self) -> *mut Object {
        let uuid = self;
        let cls = class!(CBUUID);
        unsafe {
            let obj: *mut Object = msg_send![cls, alloc];
            msg_send![obj, initWithString: NSString::from_str(&uuid)]
        }
    }
}
