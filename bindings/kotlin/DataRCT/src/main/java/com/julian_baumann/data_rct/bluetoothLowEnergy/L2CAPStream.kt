package com.julian_baumann.data_rct.bluetoothLowEnergy

import android.bluetooth.BluetoothSocket
import com.julian_baumann.data_rct.NativeStreamDelegate

class L2CAPStream(private val socket: BluetoothSocket): NativeStreamDelegate {
    override fun write(data: ByteArray): ULong {
        println("Want to write ${data.size}")
        socket.outputStream.write(data)
        socket.outputStream.flush()
        println("Written ${data.size}")

        return data.size.toULong()
    }

    override fun read(bufferLength: ULong): ByteArray {
        val buffer = ByteArray(bufferLength.toInt())
        val readBytes = socket.inputStream.read(buffer)

        if (readBytes <= 0) {
            return ByteArray(0)
        }

        return buffer.copyOfRange(0, readBytes)
    }

    override fun flush() {
        socket.outputStream.flush()
    }

    override fun disconnect() {
        println("Close called")
//        socket.close()
    }
}
