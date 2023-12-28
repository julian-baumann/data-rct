use std::{ffi::CString, sync::{Once}};
use objc::{class, declare::ClassDecl, msg_send, runtime::{BOOL, Class, NO, Object, Protocol, Sel, YES}, sel, sel_impl};
use objc_foundation::{INSArray, INSDictionary, INSString, NSArray, NSDictionary, NSObject, NSString, };
use objc_id::{Id, Shared};
use crate::{DISCOVERY_CHARACTERISTIC_UUID, DISCOVERY_SERVICE_UUID};

use crate::platforms::apple::converter::{IntoBool, IntoCBUUID};
use crate::platforms::apple::events::{peripheral_manager_did_add_service_error, peripheral_manager_did_receive_read_request, peripheral_manager_did_receive_write_requests, peripheral_manager_did_start_advertising_error};
use crate::platforms::apple::ffi::{CBAttributePermissions, CBCharacteristicProperties, dispatch_queue_create, DISPATCH_QUEUE_SERIAL, nil};

use super::{constants::{PERIPHERAL_MANAGER_DELEGATE_CLASS_NAME, PERIPHERAL_MANAGER_IVAR, POWERED_ON_IVAR}, events::{peripheral_manager_did_update_state}, ffi::{CBAdvertisementDataServiceUUIDsKey}};

static REGISTER_DELEGATE_CLASS: Once = Once::new();

#[derive(Debug)]
pub struct PeripheralManager {
    peripheral_manager_delegate: Id<Object, Shared>,
}

impl PeripheralManager {
    pub fn new() -> Self {
        REGISTER_DELEGATE_CLASS.call_once(|| {
            let mut decl =
                ClassDecl::new(PERIPHERAL_MANAGER_DELEGATE_CLASS_NAME, class!(NSObject)).unwrap();
            decl.add_protocol(Protocol::get("CBPeripheralManagerDelegate").unwrap());

            decl.add_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR);
            decl.add_ivar::<BOOL>(POWERED_ON_IVAR);

            unsafe {
                decl.add_method(
                    sel!(init),
                    init as extern "C" fn(&mut Object, Sel) -> *mut Object,
                );
                decl.add_method(
                    sel!(peripheralManagerDidUpdateState:),
                    peripheral_manager_did_update_state
                        as extern "C" fn(&mut Object, Sel, *mut Object),
                );
                decl.add_method(
                    sel!(peripheralManagerDidStartAdvertising:error:),
                    peripheral_manager_did_start_advertising_error
                        as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object),
                );
                decl.add_method(
                    sel!(peripheralManager:didAddService:error:),
                    peripheral_manager_did_add_service_error
                        as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object, *mut Object),
                );
                decl.add_method(
                    sel!(peripheralManager:didReceiveReadRequest:),
                    peripheral_manager_did_receive_read_request
                        as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object),
                );
                decl.add_method(
                    sel!(peripheralManager:didReceiveWriteRequests:),
                    peripheral_manager_did_receive_write_requests
                        as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object),
                );
            }

            decl.register();
        });

        let peripheral_manager_delegate = unsafe {
            let cls = Class::get(PERIPHERAL_MANAGER_DELEGATE_CLASS_NAME).unwrap();
            let mut obj: *mut Object = msg_send![cls, alloc];
            obj = msg_send![obj, init];
            Id::from_ptr(obj).share()
        };

        PeripheralManager {
            peripheral_manager_delegate,
        }
    }

    pub fn is_powered(self: &Self) -> bool {
        unsafe {
            let powered_on = *self
                .peripheral_manager_delegate
                .get_ivar::<BOOL>(POWERED_ON_IVAR);
            powered_on.into_bool()
        }
    }

    pub fn start_advertising(self: &Self) {
        let peripheral_manager = unsafe {
            *self
                .peripheral_manager_delegate
                .get_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR)
        };

        let service_uuid = DISCOVERY_SERVICE_UUID
            .to_string()
            .into_cbuuid();

        let mut keys: Vec<&NSString> = vec![];
        let mut objects: Vec<Id<NSObject>> = vec![];

        unsafe {
            keys.push(&*(CBAdvertisementDataServiceUUIDsKey as *mut NSString));
            objects.push(Id::from_retained_ptr(msg_send![
                NSArray::from_vec(vec![Id::<NSObject>::from_retained_ptr(service_uuid as *mut NSObject)]),
                copy
            ]));
        }

        let advertising_data = NSDictionary::from_keys_and_objects(keys.as_slice(), objects);
        unsafe {
            let _: Result<(), ()> =
                msg_send![peripheral_manager, startAdvertising: advertising_data];
        }
    }

    fn create_characteristic(&self) -> Id<NSObject> {
        unsafe {
            let cls = class!(CBMutableCharacteristic);
            let obj: *mut Object = msg_send![cls, alloc];
            let init_with_type = DISCOVERY_CHARACTERISTIC_UUID
                .to_string()
                .into_cbuuid();

            let properties: u16 =
                (CBCharacteristicProperties::CBCharacteristicPropertyRead as u16)
                | (CBCharacteristicProperties::CBCharacteristicPropertyWrite as u16);

            let permissions: u8 =
                (CBAttributePermissions::CBAttributePermissionsReadable as u8)
                | (CBAttributePermissions::CBAttributePermissionsWriteable as u8);

            let mutable_characteristic: *mut Object = msg_send![
                obj,
                initWithType: init_with_type
                properties: properties
                value: nil
                permissions: permissions
            ];

            Id::from_ptr(mutable_characteristic as *mut NSObject)
        }
    }

    pub fn configure_service(&self) {
        unsafe {
            let cls = class!(CBMutableService);
            let obj: *mut Object = msg_send![cls, alloc];

            let service: *mut Object = msg_send![
                obj,
                initWithType:DISCOVERY_SERVICE_UUID.to_string().into_cbuuid()
                primary:YES
            ];

            let characteristic = self.create_characteristic();
            let _: Result<(), ()> = msg_send![service, setValue:NSArray::from_vec(vec![characteristic])
                                 forKey:NSString::from_str("characteristics")];

            let peripheral_manager = *self
                .peripheral_manager_delegate
                .get_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR);

            let _: Result<(), ()> = msg_send![peripheral_manager, addService: service];
        }
    }

    pub fn stop_advertising(self: &Self) {
        unsafe {
            let peripheral_manager = *self
                .peripheral_manager_delegate
                .get_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR);
            let _: Result<(), ()> = msg_send![peripheral_manager, stopAdvertising];
        }
    }

    pub fn is_advertising(self: &Self) -> bool {
        unsafe {
            let peripheral_manager = *self
                .peripheral_manager_delegate
                .get_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR);
            let response: *mut Object = msg_send![peripheral_manager, isAdvertising];
            response.into_bool()
        }
    }
}

impl Default for PeripheralManager {
    fn default() -> Self {
        PeripheralManager::new()
    }
}

extern "C" fn init(delegate: &mut Object, _cmd: Sel) -> *mut Object {
    unsafe {
        let cls = class!(CBPeripheralManager);
        let mut obj: *mut Object = msg_send![cls, alloc];

        #[allow(clippy::cast_ptr_alignment)]
        let init_with_delegate = delegate as *mut Object as *mut *mut Object;

        let label = CString::new("CBqueue").unwrap();
        let queue = dispatch_queue_create(label.as_ptr(), DISPATCH_QUEUE_SERIAL);

        obj = msg_send![obj, initWithDelegate:init_with_delegate
                                        queue:queue];

        delegate.set_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR, obj);
        delegate.set_ivar::<BOOL>(POWERED_ON_IVAR, NO);

        delegate
    }
}
