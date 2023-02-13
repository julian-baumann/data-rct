import XCTest
@testable import DataRCT

final class DataRCTTests: XCTestCase {
    func testExample() throws {
        // This is an example of a functional test case.
        // Use XCTAssert and related functions to verify your tests produce the correct
        // results.
        
        let device = DeviceInfo(id: "12", name: "Device 1", port: 4242, deviceType: "computer", ipAddress: "192.168.42.42")
    
        let discovery = try Discovery(myDevice: device, method: DiscoveryMethod.both)
        discovery.advertise()
    }
}
