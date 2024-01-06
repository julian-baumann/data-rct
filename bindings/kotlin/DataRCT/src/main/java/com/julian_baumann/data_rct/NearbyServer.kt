package com.julian_baumann.data_rct

import android.content.Context
import com.julian_baumann.data_rct.bluetoothLowEnergy.BLEImplementation

class NearbyServer(context: Context, myDevice: Device) {
    private val internal: InternalNearbyServer = InternalNearbyServer(myDevice)
    private val internalBleImplementation = BLEImplementation(context, internal)

    init {
        internal.addBleImplementation(internalBleImplementation)
    }

    fun start() {
        internal.start()
    }

    fun stop() {
        internal.stop()
    }
}
