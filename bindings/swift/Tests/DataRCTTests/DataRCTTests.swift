import XCTest
@testable import DataRCT


final class DataRCTTests: XCTestCase {
    override func setUp() async throws {
        executionTimeAllowance = 20
        
        let device = Device(
            id: "39FAC7A0-E581-4676-A9C5-0F6DC667567F",
            name: "Device A",
            deviceType: 0
        )
    
//        let discovery = try Discovery(myDevice: device, method: DiscoveryMethod.both, delegate: nil)
//        discovery.advertise()
    }
    
    func testExample() throws {
        let device = Device(
            id: "B53CCB62-7DAB-4403-9FEB-F336834DB41F",
            name: "Device B",
            deviceType: 1
        )
    
//        let discovery = try Discovery(myDevice: device, method: DiscoveryMethod.both, delegate: nil)
//        discovery.startSearch()
//
//        while (true) {
//            let devices = discovery.getDevices();
//            if (devices.count > 0) {
//                XCTAssert(devices.first?.id == "39FAC7A0-E581-4676-A9C5-0F6DC667567F")
//                break;
//            }
//        }
    }
}
