//
//  File.swift
//  
//
//  Created by Julian Baumann on 06.01.24.
//

import Foundation
import CoreBluetooth

let ServiceUUID = CBUUID.init(string: getBleServiceUuid())
let CharacteristicUUID = CBUUID.init(string: getBleCharacteristicUuid())
