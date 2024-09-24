package com.julian_baumann.data_rct

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Environment
import android.util.Log
import com.julian_baumann.data_rct.bluetoothLowEnergy.BLEPeripheralManager
import com.julian_baumann.data_rct.bluetoothLowEnergy.L2CAPClientManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

class NearbyServer(context: Context, myDevice: Device, delegate: NearbyConnectionDelegate) {
    private val internal: InternalNearbyServer = InternalNearbyServer(myDevice, Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS).absolutePath, delegate)
    private val internalBleImplementation = BLEPeripheralManager(context, internal)
    private val internalL2CapClient = L2CAPClientManager(internal)
    private var currentIPAddress: String? = null
    private var connectivityManager: ConnectivityManager = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager

    private val networkCallback = object : ConnectivityManager.NetworkCallback() {
        override fun onAvailable(network: Network) {
            super.onAvailable(network)
            checkNetworkType()
        }

        override fun onLost(network: Network) {
            super.onLost(network)
            Log.d("NetworkMonitor", "No network connection")
            currentIPAddress = null
        }
    }

    private fun checkNetworkType() {
        val activeNetwork: Network? = connectivityManager.activeNetwork
        val networkCapabilities: NetworkCapabilities? = connectivityManager.getNetworkCapabilities(activeNetwork)

        if (networkCapabilities != null) {
            val ip = internal.getCurrentIp()

            when {
                networkCapabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> {
                    Log.d("NetworkMonitor", "Connected via Wi-Fi")
                    if (ip != currentIPAddress) {
                        currentIPAddress = ip
                        Log.d("NetworkMonitor", "Wi-Fi IP Address: $ip")

                        CoroutineScope(Dispatchers.Default).launch {
                            internal.restartServer()
                        }
                    }
                }
                networkCapabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> {
                    Log.d("NetworkMonitor", "Connected via Cellular Data")
                    if (ip != currentIPAddress) {
                        currentIPAddress = ip
                        Log.d("NetworkMonitor", "Cellular IP Address: $ip")

                        CoroutineScope(Dispatchers.Default).launch {
                            internal.restartServer()
                        }
                    }
                }
                else -> {
                    Log.d("NetworkMonitor", "Unknown network type")
                    currentIPAddress = null
                }
            }
        }
    }

    init {
        internal.addBleImplementation(internalBleImplementation)
        internal.addL2CapClient(internalL2CapClient)

        val request = NetworkRequest.Builder().build()
        connectivityManager.registerNetworkCallback(request, networkCallback)
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
