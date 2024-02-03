//
//  NearbyServer.swift
//
//
//  Created by Julian Baumann on 05.01.24.
//

import Foundation
import CoreBluetooth
import PhotosUI

public enum BluetoothState: Int {
    case unknown = 0
    case resetting = 1
    case unsupported = 2
    case unauthorized = 3
    case poweredOff = 4
    case poweredOn = 5
    
    init(from peripheralState: CBManagerState) {
        self = {
            switch peripheralState {
            case .unknown:
                return .unknown
            case .resetting:
                return .resetting
            case .unsupported:
                return .unsupported
            case .unauthorized:
                return .unauthorized
            case .poweredOff:
                return .poweredOff
            case .poweredOn:
                return .poweredOn
            @unknown default:
                fatalError("Unknown CBPeripheralManager.state")
            }
        }()
    }
}

public enum DeviceType: Int32 {
    case unknown
    case mobile
    case tablet
    case desktop
    case tv
    case car
    case watch
    case embedded
}

public protocol NearbyServerDelegate: NearbyConnectionDelegate {
    func nearbyServerDidUpdateState(state: BluetoothState)
}

public class NearbyServer {
    private let internalHandler: InternalNearbyServer
    private let bleServer: BLEPeripheralManager
    public var state: BluetoothState { get { bleServer.state } }
    
    public init(myDevice: Device, storage: String, delegate: NearbyServerDelegate) {
        internalHandler = InternalNearbyServer(myDevice: myDevice, fileStorage: storage, delegate: delegate)
        bleServer = BLEPeripheralManager(handler: internalHandler, delegate: delegate)

        internalHandler.addBleImplementation(bleImplementation: bleServer)
        internalHandler.addL2CapClient(delegate: L2CAPClient(internalHandler: internalHandler))
    }
    
    public func changeDevice(_ newDevice: Device) {
        internalHandler.changeDevice(newDevice: newDevice)
    }
    
    public func start() async throws {
        try bleServer.ensureValidState()

        await internalHandler.start()
    }
    
    @available(macOS 13.0, *)
    @available(iOS 14.0, *)
    public func sendFile(to device: Device, url: String, progress: SendProgressDelegate?) async throws {
        try await internalHandler.sendFile(receiver: device, filePath: url, progressDelegate: progress)
    }
    
    public func stop() async throws {
        try bleServer.ensureValidState()

        await internalHandler.stop()
    }
}
