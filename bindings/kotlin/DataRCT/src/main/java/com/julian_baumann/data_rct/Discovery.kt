package com.julian_baumann.data_rct

import android.content.Context
import com.julian_baumann.data_rct.bluetoothLowEnergy.BleDiscovery

interface DiscoveryDelegate: DeviceListUpdateDelegate

class Discovery(context: Context, delegate: DiscoveryDelegate) {
    private val internal: InternalDiscovery = InternalDiscovery(delegate)
    private val bleImplementation: BleDiscovery = BleDiscovery(context, internal)

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
