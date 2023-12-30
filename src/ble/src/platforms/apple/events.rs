use objc::{msg_send, sel, sel_impl};
use objc::runtime::{BOOL, NO, Object, Sel, YES};
use objc_foundation::{INSArray, INSData, INSString, NSArray, NSData, NSObject, NSString};
use protocol::discovery::Device;
use crate::platforms::apple::constants::POWERED_ON_IVAR;
use crate::platforms::apple::converter::IntoBool;
use crate::platforms::apple::ffi::{CBATTError, CBManagerState};
use protocol::prost::Message;
use crate::platforms::apple::{add_new_device, DISCOVERED_DEVICES, DISCOVERY_DELEGATE, get_discovered_devices_mutex};

pub extern "C" fn peripheral_manager_did_update_state(
    delegate: &mut Object,
    _cmd: Sel,
    peripheral: *mut Object,
) {
    println!("peripheral_manager_did_update_state");

    unsafe {
        let state: CBManagerState = msg_send![peripheral, state];
        match state {
            CBManagerState::CBManagerStateUnknown => {
                println!("CBManagerStateUnknown");
            }
            CBManagerState::CBManagerStateResetting => {
                println!("CBManagerStateResetting");
            }
            CBManagerState::CBManagerStateUnsupported => {
                println!("CBManagerStateUnsupported");
            }
            CBManagerState::CBManagerStateUnauthorized => {
                println!("CBManagerStateUnauthorized");
            }
            CBManagerState::CBManagerStatePoweredOff => {
                println!("CBManagerStatePoweredOff");
                delegate.set_ivar::<BOOL>(POWERED_ON_IVAR, NO);
            }
            CBManagerState::CBManagerStatePoweredOn => {
                println!("CBManagerStatePoweredOn");
                delegate.set_ivar(POWERED_ON_IVAR, YES);
            }
        };
    }
}

pub extern "C" fn peripheral_manager_did_start_advertising_error(
    _delegate: &mut Object,
    _cmd: Sel,
    _peripheral: *mut Object,
    error: *mut Object,
) {
    if error.is_null() {
        return;
    }

    println!("peripheral_manager_did_start_advertising_error");
    println!("Type: {:?}", error);
    if error.into_bool() {
        let localized_description: *mut Object = unsafe { msg_send![error, localizedDescription] };
        let string = localized_description as *mut NSString;
        println!("{:?}", unsafe { (*string).as_str() });
    }
}


pub extern "C" fn peripheral_manager_did_add_service_error(
    _delegate: &mut Object,
    _cmd: Sel,
    _peripheral: *mut Object,
    _service: *mut Object,
    error: *mut Object,
) {
    println!("peripheral_manager_did_add_service_error");
    if error.into_bool() {
        let localized_description: *mut Object = unsafe { msg_send![error, localizedDescription] };
        let string = localized_description as *mut NSString;
        println!("{:?}", unsafe { (*string).as_str() });
    }
}

pub extern "C" fn peripheral_manager_did_receive_read_request(
    _delegate: &mut Object,
    _cmd: Sel,
    peripheral: *mut Object,
    request: *mut Object,
) {
    unsafe {
        let _: Result<(), ()> = msg_send![peripheral, respondToRequest:request
                                    withResult:CBATTError::CBATTErrorSuccess];
    }
}

pub extern "C" fn peripheral_manager_did_receive_write_requests(
    _delegate: &mut Object,
    _cmd: Sel,
    peripheral: *mut Object,
    requests: *mut Object,
) {
    unsafe {
        for request in (*(requests as *mut NSArray<NSObject>)).to_vec() {
            let value : *mut NSData = msg_send![request, value];
            let value = (*value).bytes();

            let device = Device::decode_length_delimited(value);

            if let Ok(device) = device {
                let device_added = add_new_device(device.clone());

                if device_added {
                    if let Some(discovery_delegate) = &DISCOVERY_DELEGATE {
                        discovery_delegate.lock().unwrap().device_added(device.clone());
                    }
                }
            }

            let _: Result<(), ()> = msg_send![peripheral, respondToRequest:request
                                        withResult:CBATTError::CBATTErrorSuccess];
        }
    }
}
