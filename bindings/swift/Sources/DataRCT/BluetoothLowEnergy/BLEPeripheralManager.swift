//
//  File.swift
//  
//
//  Created by Julian Baumann on 05.01.24.
//

import Foundation
import CoreBluetooth

struct InvalidStateError: Error {}

class BLEPeripheralManager: NSObject, BleServerImplementationDelegate, CBPeripheralManagerDelegate {
    private let peripheralManager: CBPeripheralManager
    private let internalHandler: InternalNearbyServer
    private let nearbyServerDelegate: NearbyServerDelegate
    private var streams: [L2CapStream] = []

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
    
    func startL2CapServer() {
        peripheralManager.publishL2CAPChannel(withEncryption: false)
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didPublishL2CAPChannel PSM: CBL2CAPPSM, error: Error?) {
        print("L2CAP Channel PSM: \(PSM)")
        internalHandler.setBleConnectionDetails(bleDetails: BluetoothLeConnectionInfo(uuid: "", psm: UInt32(PSM)))
        addService()
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didOpen channel: CBL2CAPChannel?, error: Error?) {
        print("L2CAP Channel was opened")
        
        guard let channel else {
            return
        }
        
        let l2capStream = L2CapStream(channel: channel)
        streams.append(l2capStream)

        Task {
            internalHandler.handleIncomingConnection(nativeStreamHandle: l2capStream)
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

        service.characteristics = [characteristic]
        
        peripheralManager.add(service)

        peripheralManager.startAdvertising([
            CBAdvertisementDataServiceUUIDsKey: [ServiceUUID]
        ])
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didAdd service: CBService, error: Error?) {
        if error != nil {
            print(error!)
        }
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveRead request: CBATTRequest) {
        Task {
            request.value = await internalHandler.getAdvertisementData()
            peripheral.respond(to: request, withResult: CBATTError.success)
        }
    }
    
    func peripheralManagerDidStartAdvertising(_ peripheral: CBPeripheralManager, error: Error?) {
        if error != nil {
            print(error!)
        }
    }
    
    func startServer() {
        startL2CapServer()
    }
    
    func stopServer() {
        peripheralManager.stopAdvertising()
        peripheralManager.removeAllServices()
    }
}
