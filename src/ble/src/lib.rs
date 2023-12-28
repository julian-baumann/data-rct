use crate::platforms::apple::Discovery;

mod platforms;


trait CorePeripheral {
    fn init();
}



#[test]
pub fn test() {
    let peripheral = Discovery::new();
    while !peripheral.is_powered_on() {}

    peripheral.start_discovering_devices();

    while !peripheral.is_discovering() {}

    println!("Is advertising");

    loop {}

    println!("is advertising: {:}", peripheral.is_discovering())
}
