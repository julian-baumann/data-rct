//
//  Stream.swift
//
//
//  Created by Julian Baumann on 08.01.24.
//

import Foundation
import DataRCTFFI
import CoreBluetooth

class L2CapStream: NSObject, StreamDelegate, NativeStreamDelegate {
    private var channel: CBL2CAPChannel?
    
    init(channel: CBL2CAPChannel) {
        self.channel = channel
     
        super.init()

        self.channel!.inputStream.open()
        self.channel!.outputStream.open()
    }
    
    func read(bufferLength: UInt64) -> Data {
        let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: Int(bufferLength))
        defer { buffer.deallocate() }

        var data = Data()

        let numberOfBytesRead = channel!.inputStream.read(buffer, maxLength: Int(bufferLength))

        if numberOfBytesRead > 0 {
            data.append(buffer, count: numberOfBytesRead)
        }

        return data
    }
    
    func flush() {
        // TODO
    }
    
    func write(data: Data) -> UInt64 {
        var bytesWritten = 0;

        let _ = data.withUnsafeBytes {
            bytesWritten = channel!.outputStream.write($0.bindMemory(to: UInt8.self).baseAddress!, maxLength: data.count)
        }
        
        return UInt64(bytesWritten)
    }
    
    func disconnect() {
        channel!.outputStream.close()
        channel!.inputStream.close()
        channel = nil
    }
}
