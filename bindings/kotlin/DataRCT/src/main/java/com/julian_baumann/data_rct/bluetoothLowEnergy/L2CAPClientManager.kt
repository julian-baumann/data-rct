package com.julian_baumann.data_rct.bluetoothLowEnergy

import android.annotation.SuppressLint
import com.julian_baumann.data_rct.InternalNearbyServer
import com.julian_baumann.data_rct.L2CapDelegate
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class L2CAPClientManager(private val internalHandler: InternalNearbyServer): L2CapDelegate {
    @SuppressLint("MissingPermission")
    override fun openL2capConnection(connectionId: String, peripheralUuid: String, psm: UInt) {
        val peripheral = BLECentralManager.discoveredPeripherals.find { device -> device.address == peripheralUuid }

        if (peripheral == null) {
            return
        }

        val socket = peripheral.createInsecureL2capChannel(psm.toInt())
        socket.connect()
        val stream = L2CAPStream(socket)

        Thread {
            CoroutineScope(Dispatchers.Main).launch {
                internalHandler.handleIncomingBleConnection(connectionId, stream)
            }
        }.start()
    }
}
