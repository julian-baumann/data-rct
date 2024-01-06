//
//  File.swift
//  
//
//  Created by Julian Baumann on 05.01.24.
//

import Foundation
import CoreBluetooth

struct InvalidStateError: Error {}

class BLEServer: NSObject, BleServerImplementationDelegate, CBPeripheralManagerDelegate {
    private let peripheralManager: CBPeripheralManager
    private let internalHandler: InternalNearbyServer
    private let nearbyServerDelegate: NearbyServerDelegate

    private var isPoweredOn = false
    public var state: BluetoothState

    init(handler: InternalNearbyServer, delegate: NearbyServerDelegate) {
        nearbyServerDelegate = delegate
        internalHandler = handler
        peripheralManager = CBPeripheralManager()
        state = BluetoothState(from: peripheralManager.state)
        
        super.init()
        peripheralManager.delegate = self
    }
    
    func peripheralManagerDidUpdateState(_ peripheral: CBPeripheralManager) {
        state = BluetoothState(from: peripheral.state)
        nearbyServerDelegate.nearbyServerDidUpdateState(state: state)
    }
    
    public func ensureValidState() throws {
        if state != .poweredOn {
            throw InvalidStateError()
        }
    }
    
    func addService() {
        let service = CBMutableService(type: ServiceUUID, primary: true)
        let characteristic = CBMutableCharacteristic(
            type: CharacteristicUUID,
            properties: [.read],
            value: nil,
            permissions: CBAttributePermissions.readable
        )
        
        let descriptor = CBMutableDescriptor(
            type: CBUUID(string: "DBF73DC3-450F-47BF-B79D-EC952A913E5B"),
            value: "Test".data(using: String.Encoding.utf8)
        )

        service.characteristics = [characteristic]
        
        peripheralManager.add(service)

        peripheralManager.startAdvertising([
            CBAdvertisementDataLocalNameKey: "Apple Device",
            CBAdvertisementDataServiceUUIDsKey: [ServiceUUID]
        ])
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didAdd service: CBService, error: Error?) {
        if error != nil {
            print(error!)
        }
    }
    
//    func peripheralManager(_ peripheral: CBPeripheralManager, central: CBCentral, didSubscribeTo characteristic: CBCharacteristic) {
//        let data = "Hello".data(using: String.Encoding.utf8)!
//        peripheral.updateValue(data, for: characteristic as! CBMutableCharacteristic, onSubscribedCentrals: [central])
//    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveRead request: CBATTRequest) {
        print("read requested")
        request.value = internalHandler.getAdvertisementData()
        peripheral.respond(to: request, withResult: CBATTError.success)
    }
    
    func peripheralManagerDidStartAdvertising(_ peripheral: CBPeripheralManager, error: Error?) {
        if error != nil {
            print(error!)
        }
    }
    
    func startServer() {
        addService()
    }
    
    func stopServer() {
        peripheralManager.stopAdvertising()
        peripheralManager.removeAllServices()
    }
    
    func read() -> Data {
        fatalError("Unimplemented")
    }
    
    func write(data: Data) {
        fatalError("Unimplemented")
    }
}
