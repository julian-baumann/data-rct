//
//  NearbyServer.swift
//
//
//  Created by Julian Baumann on 05.01.24.
//

import Foundation
import CoreBluetooth

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

public protocol NearbyServerDelegate {
    func nearbyServerDidUpdateState(state: BluetoothState)
}

public class NearbyServer {
    private let internalHandler: InternalNearbyServer
    private let bleServer: BLEServer
    public var state: BluetoothState { get { bleServer.state } }
    
    public init(myDevice: Device, delegate: NearbyServerDelegate) throws {
        internalHandler = try InternalNearbyServer(myDevice: myDevice)
        bleServer = BLEServer(handler: internalHandler, delegate: delegate)

        internalHandler.addBleImplementation(implementation: bleServer)
    }
    
    public func changeDevice(_ newDevice: Device) {
        internalHandler.changeDevice(device: newDevice)
    }
    
    public func start() throws {
        try bleServer.ensureValidState()

        internalHandler.start()
    }
    
    public func stop() throws {
        try bleServer.ensureValidState()

        internalHandler.stop()
    }
}
