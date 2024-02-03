package com.julian_baumann.data_rct

import android.content.Context
import com.julian_baumann.data_rct.bluetoothLowEnergy.BLECentralManager

interface DiscoveryDelegate: DeviceListUpdateDelegate

class Discovery(context: Context, delegate: DiscoveryDelegate) {
    private val internal: InternalDiscovery = InternalDiscovery(delegate)
    private val bleImplementation: BLECentralManager = BLECentralManager(context, internal)

    init {
        internal.addBleImplementation(bleImplementation)
    }

    fun startScanning() {
        internal.start()
    }

    fun stopScanning() {
        internal.stop()
    }
}
