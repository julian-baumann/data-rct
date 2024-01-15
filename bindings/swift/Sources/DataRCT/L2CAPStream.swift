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
    private var innerStream: NativeStream?
    
    init(inputStream: InputStream, outputStream: OutputStream) {
        self.inputStream = inputStream
        self.outputStream = outputStream
     
        super.init()

        self.innerStream = NativeStream(delegate: self)
        self.inputStream.delegate = self
    }

    func stream(_ aStream: Stream, handle eventCode: Stream.Event) {
        switch eventCode {
        case .hasBytesAvailable:
            readAvailableBytes(stream: aStream as! InputStream)
            break
        default:
            // Handle other cases like error, end of stream, etc.
            break
        }
    }
    
    private func readAvailableBytes(stream: InputStream) {
        let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: 1024)

        while stream.hasBytesAvailable {
            let numberOfBytesRead = inputStream.read(buffer, maxLength: 1024)

            if numberOfBytesRead < 0, let error = stream.streamError {
                print(error)
                break
            }
            
            guard let innerStream = innerStream else {
                continue
            }
            
            innerStream.fillBuffer(data: Data(bytes: buffer, count: numberOfBytesRead))
        }
    
        buffer.deallocate()
    }
    
    func write(data: Data) -> UInt64 {
        var bytesWritten = 0;

        let _ = data.withUnsafeBytes {
            bytesWritten = outputStream.write($0.bindMemory(to: UInt8.self).baseAddress!, maxLength: data.count)
        }
        
        return UInt64(bytesWritten)
    }
    
    func close() {
        outputStream.close()
        inputStream.close()
    }
}
