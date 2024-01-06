package com.julian_baumann.data_rct.bluetoothLowEnergy

import android.Manifest
import android.bluetooth.*
import android.bluetooth.le.*
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.ParcelUuid
import android.util.Log
import androidx.core.app.ActivityCompat
import com.julian_baumann.data_rct.BleServerImplementationDelegate
import com.julian_baumann.data_rct.InternalNearbyServer
import java.util.*


class BlePermissionNotGrantedException : Exception()
val discoveryServiceUUID: UUID = UUID.fromString("68D60EB2-8AAA-4D72-8851-BD6D64E169B7")
val discoveryCharacteristicUUID: UUID = UUID.fromString("0BEBF3FE-9A5E-4ED1-8157-76281B3F0DA5")

internal class BLEImplementation(private val context: Context, private val internalNearbyServer: InternalNearbyServer) : BleServerImplementationDelegate {
    private val bluetoothManager: BluetoothManager by lazy {
        context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    }

    private var bluetoothGattServer: BluetoothGattServer? = null

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

            val data = internalNearbyServer.getAdvertisementData()

            bluetoothGattServer?.sendResponse(device,
                requestId,
                BluetoothGatt.GATT_SUCCESS,
                0,
                data
            )
        }
    }

    private val bluetoothReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            val state = intent.getIntExtra(BluetoothAdapter.EXTRA_STATE, BluetoothAdapter.STATE_OFF)

            when (state) {
                BluetoothAdapter.STATE_ON -> {
                    startAdvertising()
                    startGattServer()
                }
                BluetoothAdapter.STATE_OFF -> {
                    stopGattServer()
                    stopAdvertising()
                }
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

        bluetoothManager.adapter.listenUsingInsecureL2capChannel()

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
                .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_MEDIUM)
                .build()

            val data = AdvertiseData.Builder()
                .setIncludeDeviceName(true)
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
            startAdvertising()
            startGattServer()
        }
    }

    override fun stopServer() {
        stopAdvertising()
        stopGattServer()
    }
}
