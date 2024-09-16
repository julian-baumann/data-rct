//
//  File.swift
//  
//
//  Created by Julian Baumann on 30.01.24.
//

import Foundation
import CoreBluetooth

struct ConnectionDetails {
    let connectionId: String
    let PSM: CBL2CAPPSM
}

public class L2CAPClient: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate, L2CapDelegate {
    private let centralManager = CBCentralManager()
    private let internalHandler: InternalNearbyServer
    private var connections: [CBPeripheral: ConnectionDetails] = [:]
    private var streams: [L2CapStream] = []
    
    init(internalHandler: InternalNearbyServer) {
        self.internalHandler = internalHandler

        super.init()
        
        centralManager.delegate = self
    }
    
    public func centralManagerDidUpdateState(_ central: CBCentralManager) {
        // Update state
    }

    public func peripheral(_ peripheral: CBPeripheral, didOpen channel: CBL2CAPChannel?, error: Error?) {
        guard let channel else {
            return
        }
        
        let connectionDetails = connections[peripheral]

        guard let connectionDetails else {
            return
        }
        
        let l2capStream = L2CapStream(channel: channel)
        streams.append(l2capStream)

        Task {
            internalHandler.handleIncomingBleConnection(connectionId: connectionDetails.connectionId, nativeStream: l2capStream)
        }
    }
    
    public func openL2capConnection(connectionId: String, peripheralUuid: String, psm: UInt32) {
        let peripherals = centralManager.retrievePeripherals(withIdentifiers: [
            UUID(uuidString: peripheralUuid)!
        ])

        guard let peripheral = peripherals.first else {
            print("ERROR: Couldn't locate peripheral")
            return
        }

        connections[peripheral] = ConnectionDetails(connectionId: connectionId, PSM: CBL2CAPPSM(psm))
        peripheral.delegate = self

        centralManager.connect(peripheral)
    }
    
    public func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        let connectionDetails = connections[peripheral]
        
        guard let connectionDetails else {
            return
        }
        
        peripheral.openL2CAPChannel(connectionDetails.PSM)
    }
}
