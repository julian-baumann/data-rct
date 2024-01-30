//
//  Stream.swift
//
//
//  Created by Julian Baumann on 08.01.24.
//

import Foundation
import DataRCTFFI

class L2CapStream: NSObject, StreamDelegate, NativeStreamDelegate {
    private let inputStream: InputStream
    private let outputStream: OutputStream
    
    init(inputStream: InputStream, outputStream: OutputStream) {
        self.inputStream = inputStream
        self.outputStream = outputStream
     
        super.init()

//        self.inputStream.delegate = self
//        self.outputStream.delegate = self
        
        self.inputStream.schedule(in: .main, forMode: RunLoop.Mode.default)
        self.outputStream.schedule(in: .main, forMode: RunLoop.Mode.default)
        
        self.inputStream.open()
        self.outputStream.open()
    }
    
    func read(bufferLength: UInt64) -> Data {
        print("Want to READ from L2CAP stream")
        
        let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: Int(bufferLength))
        defer { buffer.deallocate() }

        var data = Data()

        while inputStream.hasBytesAvailable {
            let numberOfBytesRead = inputStream.read(buffer, maxLength: Int(bufferLength))
            print("L2Cap Bytes read \(numberOfBytesRead)")

            if numberOfBytesRead < 0, let error = inputStream.streamError {
                print(error)
                break
            }

            if numberOfBytesRead > 0 {
                data.append(buffer, count: numberOfBytesRead)
            } else {
                break
            }
        }

        return data
    }
    
    func flush() {
        // TODO
    }
    
    func write(data: Data) -> UInt64 {
        print("Want to write L2CAP bytes")

        var bytesWritten = 0;

        let _ = data.withUnsafeBytes {
            bytesWritten = outputStream.write($0.bindMemory(to: UInt8.self).baseAddress!, maxLength: data.count)
            print("L2Cap Bytes written \(bytesWritten)")
        }

        print("Written \(bytesWritten) L2CAP bytes")
        
        return UInt64(bytesWritten)
    }
    
    func close() {
        outputStream.close()
        inputStream.close()
    }
}
