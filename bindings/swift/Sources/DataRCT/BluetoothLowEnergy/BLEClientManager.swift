//
//  BleClient.swift
//
//
//  Created by Julian Baumann on 06.01.24.
//

import Foundation
import CoreBluetooth

public enum OpenL2CAPErrors: Error {
    case PeripheralNotFound
}

public class BLEClientManager: NSObject, BleDiscoveryImplementationDelegate, CBCentralManagerDelegate, CBPeripheralDelegate {
    private let delegate: DiscoveryDelegate
    private let internalHandler: InternalDiscovery
    private let centralManager = CBCentralManager()
    private var state: BluetoothState = .unknown
    private var discoveredPeripherals: [CBPeripheral] = []

    init(delegate: DiscoveryDelegate, internalHandler: InternalDiscovery) {
        self.delegate = delegate
        self.internalHandler = internalHandler

        super.init()
        centralManager.delegate = self
    }

    public func ensureValidState() throws {
        if state != .poweredOn {
            throw InvalidStateError()
        }
    }

    public func centralManagerDidUpdateState(_ central: CBCentralManager) {
        state = BluetoothState(from: central.state)
        delegate.discoveryDidUpdateState(state: state)
    }

    public func startScanning() {
        if centralManager.isScanning {
            return
        }

        centralManager.scanForPeripherals(withServices: [ServiceUUID], options: [
            CBCentralManagerScanOptionAllowDuplicatesKey: true
        ])
    }

    public func stopScanning() {
        centralManager.stopScan()
        centralManager.delegate = nil
    }

    public func centralManager(_ central: CBCentralManager, didDiscover peripheral: CBPeripheral, advertisementData: [String : Any], rssi RSSI: NSNumber) {
        peripheral.delegate = self
        discoveredPeripherals.append(peripheral)
        central.connect(peripheral)
    }

    public func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        peripheral.discoverServices([ServiceUUID])
    }

    public func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        if let error = error {
            print(error)
            return
        }

        let service = peripheral.services?.first(where: { $0.uuid == ServiceUUID })

        guard let service = service else {
            return
        }

        peripheral.discoverCharacteristics([CharacteristicUUID], for: service)
    }

    public func peripheral(_ peripheral: CBPeripheral, didDiscoverCharacteristicsFor service: CBService, error: Error?) {
        if let error = error {
            print(error)
            return
        }

        let characteristic = service.characteristics?.first(where: { $0.uuid == CharacteristicUUID })

        if let characteristic = characteristic {
            peripheral.readValue(for: characteristic)
        }
    }

    public func peripheral(_ peripheral: CBPeripheral, didModifyServices invalidatedServices: [CBService]) {
        peripheral.discoverServices([ServiceUUID])
        for service in invalidatedServices {
            if discoveredPeripherals.contains(where: { $0 == service.peripheral } ) {
                discoveredPeripherals.removeAll(where: { $0 == service.peripheral })
            }
        }
    }

    public func peripheral(_ peripheral: CBPeripheral, didUpdateValueFor characteristic: CBCharacteristic, error: Error?) {
        if let error = error {
            print(error)
            return
        }

        let data = characteristic.value

        if let data = data {
            internalHandler.parseDiscoveryMessage(data: data, bleUuid: peripheral.identifier.uuidString)
            centralManager.cancelPeripheralConnection(peripheral)
        }
    }
}
