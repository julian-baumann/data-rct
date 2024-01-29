package com.julian_baumann.data_rct

import android.content.Context
import android.os.Environment
import com.julian_baumann.data_rct.bluetoothLowEnergy.BLEImplementation

class NearbyServer(context: Context, myDevice: Device, delegate: NearbyConnectionDelegate) {
    private val internal: InternalNearbyServer = InternalNearbyServer(myDevice, Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS).absolutePath, delegate)
    private val internalBleImplementation = BLEImplementation(context, internal)

    init {
        internal.addBleImplementation(internalBleImplementation)
    }

    suspend fun start() {
        internal.start()
    }

    suspend fun sendFile(receiver: Device, fileUrl: String, progressDelegate: ProgressDelegate?) {
        internal.sendFile(receiver, fileUrl, progressDelegate)
    }

    suspend fun stop() {
        internal.stop()
    }
}
