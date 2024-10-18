package com.julian_baumann.data_rct.bluetoothLowEnergy

import android.bluetooth.BluetoothSocket
import android.util.Log
import com.julian_baumann.data_rct.NativeStreamDelegate

class L2CAPStream(private val socket: BluetoothSocket): NativeStreamDelegate {
    override fun write(data: ByteArray): ULong {
        try {
            socket.outputStream.write(data)

            return data.size.toULong()
        } catch (exception: Exception) {
            Log.w("L2CAPStream write exception:", exception)

            return 0.toULong()
        }
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
        socket.close()
    }
}
