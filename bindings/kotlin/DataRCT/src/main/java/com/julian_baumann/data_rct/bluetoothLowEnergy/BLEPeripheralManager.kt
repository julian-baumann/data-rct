package com.julian_baumann.data_rct.bluetoothLowEnergy

import android.Manifest
import android.bluetooth.*
import android.bluetooth.le.*
import android.content.Context
import android.content.pm.PackageManager
import android.os.ParcelUuid
import android.util.Log
import androidx.core.app.ActivityCompat
import com.julian_baumann.data_rct.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.util.*


class BlePermissionNotGrantedException : Exception()
val discoveryServiceUUID: UUID = UUID.fromString(getBleServiceUuid())
val discoveryCharacteristicUUID: UUID = UUID.fromString(getBleCharacteristicUuid())

internal class BLEPeripheralManager(private val context: Context, private val internalNearbyServer: InternalNearbyServer) : BleServerImplementationDelegate {
    private val bluetoothManager: BluetoothManager by lazy {
        context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    }

    private var bluetoothGattServer: BluetoothGattServer? = null
    private var bluetoothL2CAPServer: BluetoothServerSocket? = null
    private var l2CAPThread: Thread? = null

    private fun createService(): BluetoothGattService {
        val service = BluetoothGattService(discoveryServiceUUID, BluetoothGattService.SERVICE_TYPE_PRIMARY)
        val characteristic = BluetoothGattCharacteristic(discoveryCharacteristicUUID, BluetoothGattCharacteristic.PROPERTY_READ, BluetoothGattCharacteristic.PERMISSION_READ)

        service.addCharacteristic(characteristic)

        return service
    }

    private val gattServerCallback = object : BluetoothGattServerCallback() {
        override fun onCharacteristicReadRequest(
            device: BluetoothDevice?,
            requestId: Int,
            offset: Int,
            characteristic: BluetoothGattCharacteristic?
        ) {
            if (ActivityCompat.checkSelfPermission(context, Manifest.permission.BLUETOOTH_ADVERTISE) != PackageManager.PERMISSION_GRANTED) {
                throw BlePermissionNotGrantedException()
            }

            CoroutineScope(Dispatchers.Main).launch {
                val data = internalNearbyServer.getAdvertisementData()

                bluetoothGattServer?.sendResponse(device,
                    requestId,
                    BluetoothGatt.GATT_SUCCESS,
                    0,
                    data
                )
            }
        }
    }

    private val advertiseCallback = object : AdvertiseCallback() {
        override fun onStartSuccess(settingsInEffect: AdvertiseSettings) {
            Log.i("BLE", "LE Advertise Started.")
        }

        override fun onStartFailure(errorCode: Int) {
            Log.w("BLE", "LE Advertise Failed: $errorCode")
        }
    }

    private fun startGattServer() {
        if (ActivityCompat.checkSelfPermission(context, Manifest.permission.BLUETOOTH_ADVERTISE) != PackageManager.PERMISSION_GRANTED) {
            throw BlePermissionNotGrantedException()
        }

        bluetoothL2CAPServer = bluetoothManager.adapter.listenUsingInsecureL2capChannel()

        l2CAPThread = Thread {
            val psm = bluetoothL2CAPServer!!.psm.toUInt()
            internalNearbyServer.setBleConnectionDetails(BluetoothLeConnectionInfo("", psm))

            while (true) {
                val connection = bluetoothL2CAPServer!!.accept()
                val stream = L2CAPStream(connection)

                CoroutineScope(Dispatchers.Main).launch {
                    internalNearbyServer.handleIncomingConnection(stream)
                }
            }
        }

        l2CAPThread!!.start()

        bluetoothGattServer = bluetoothManager.openGattServer(context, gattServerCallback)
        bluetoothGattServer?.addService(createService())
            ?: Log.w("BLE", "Unable to create GATT server")
    }

    private fun stopGattServer() {
        if (ActivityCompat.checkSelfPermission(context, Manifest.permission.BLUETOOTH_ADVERTISE) != PackageManager.PERMISSION_GRANTED) {
            throw BlePermissionNotGrantedException()
        }

        bluetoothGattServer?.close()
    }

    private fun startAdvertising() {
        val bluetoothLeAdvertiser: BluetoothLeAdvertiser? = bluetoothManager.adapter.bluetoothLeAdvertiser

        bluetoothLeAdvertiser?.let {

            val settings = AdvertiseSettings.Builder()
                .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
                .setConnectable(true)
                .setTimeout(0)
                .build()

            val data = AdvertiseData.Builder()
                .setIncludeDeviceName(false)
                .setIncludeTxPowerLevel(false)
                .addServiceUuid(ParcelUuid(discoveryServiceUUID))
                .build()

            if (ActivityCompat.checkSelfPermission(context, Manifest.permission.BLUETOOTH_ADVERTISE) != PackageManager.PERMISSION_GRANTED) {
                throw BlePermissionNotGrantedException()
            }

            it.startAdvertising(settings, data, advertiseCallback)
        } ?: Log.w("BLE", "Failed to create advertiser")
    }

    private fun stopAdvertising() {
        if (ActivityCompat.checkSelfPermission(context, Manifest.permission.BLUETOOTH_ADVERTISE) != PackageManager.PERMISSION_GRANTED) {
            throw BlePermissionNotGrantedException()
        }

        val bluetoothLeAdvertiser: BluetoothLeAdvertiser? = bluetoothManager.adapter.bluetoothLeAdvertiser
        bluetoothLeAdvertiser?.stopAdvertising(advertiseCallback) ?: Log.w("BLE", "Failed to create advertiser")
    }

    override fun startServer() {
        if (!bluetoothManager.adapter.isEnabled) {
            Log.d("BLE", "Bluetooth is currently disabled...enabling")
        } else {
            Log.d("BLE", "Bluetooth enabled...starting services")
            startGattServer()
            startAdvertising()
        }
    }

    override fun stopServer() {
        stopAdvertising()
        stopGattServer()
    }
}
