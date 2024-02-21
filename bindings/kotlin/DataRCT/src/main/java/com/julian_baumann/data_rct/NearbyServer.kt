package com.julian_baumann.data_rct

import android.content.Context
import android.os.Environment
import com.julian_baumann.data_rct.bluetoothLowEnergy.BLEPeripheralManager
import com.julian_baumann.data_rct.bluetoothLowEnergy.L2CAPClientManager

class NearbyServer(context: Context, myDevice: Device, delegate: NearbyConnectionDelegate) {
    private val internal: InternalNearbyServer = InternalNearbyServer(myDevice, Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS).absolutePath, delegate)
    private val internalBleImplementation = BLEPeripheralManager(context, internal)
    private val internalL2CapClient = L2CAPClientManager(internal)

    init {
        internal.addBleImplementation(internalBleImplementation)
        internal.addL2CapClient(internalL2CapClient)
    }

    suspend fun start() {
        internal.start()
    }

    fun changeDevice(newDevice: Device) {
        internal.changeDevice(newDevice)
    }

    suspend fun sendFile(receiver: Device, fileUrl: String, progressDelegate: SendProgressDelegate?) {
        internal.sendFile(receiver, fileUrl, progressDelegate)
    }

    suspend fun stop() {
        internal.stop()
    }
}
