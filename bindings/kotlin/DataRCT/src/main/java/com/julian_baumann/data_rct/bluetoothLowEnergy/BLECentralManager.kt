package com.julian_baumann.data_rct.bluetoothLowEnergy

import android.Manifest
import android.annotation.SuppressLint
import android.bluetooth.*
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.content.pm.PackageManager
import android.os.ParcelUuid
import androidx.core.app.ActivityCompat
import com.julian_baumann.data_rct.BleDiscoveryImplementationDelegate
import com.julian_baumann.data_rct.InternalDiscovery
import kotlinx.coroutines.*
import java.util.*


@SuppressLint("MissingPermission")
class BluetoothGattCallbackImplementation(private val internal: InternalDiscovery, private var discoveredPeripherals: MutableList<BluetoothDevice>) : BluetoothGattCallback() {
    override fun onConnectionStateChange(gatt: BluetoothGatt, status: Int, newState: Int) {
        if (newState == BluetoothProfile.STATE_CONNECTED) {
            gatt.requestMtu(150)
        } else if (newState == BluetoothProfile.STATE_DISCONNECTED) {
            discoveredPeripherals.remove(gatt.device)
        }
    }

    override fun onMtuChanged(gatt: BluetoothGatt?, mtu: Int, status: Int) {
        if (status == BluetoothGatt.GATT_SUCCESS) {
            gatt?.discoverServices()
        }
    }

    override fun onServicesDiscovered(gatt: BluetoothGatt, status: Int) {
        if (status == BluetoothGatt.GATT_SUCCESS) {
            getDeviceInfo(gatt)
        }
    }

    @SuppressLint("MissingPermission")
    private fun getDeviceInfo(gatt: BluetoothGatt) {
        val service = gatt.getService(discoveryServiceUUID)

        service?.let {
            val characteristic = it.getCharacteristic(discoveryCharacteristicUUID)
            gatt.readCharacteristic(characteristic)
        }
    }

    override fun onCharacteristicChanged(
        gatt: BluetoothGatt,
        characteristic: BluetoothGattCharacteristic,
        value: ByteArray
    ) {
        super.onCharacteristicChanged(gatt, characteristic, value)
        internal.parseDiscoveryMessage(value, gatt.device.address)
    }

    // Still needed for older Android versions (< 13)
    @Deprecated("Deprecated")
    override fun onCharacteristicRead(
        gatt: BluetoothGatt?,
        characteristic: BluetoothGattCharacteristic?,
        status: Int
    ) {
        super.onCharacteristicRead(gatt, characteristic, status)

        if (gatt != null && characteristic != null && characteristic.value != null) {
            handleCharacteristicData(characteristic.value, status, gatt)
        }
    }

    override fun onCharacteristicRead(
        gatt: BluetoothGatt,
        characteristic: BluetoothGattCharacteristic,
        value: ByteArray,
        status: Int
    ) {
        super.onCharacteristicRead(gatt, characteristic, value, status)
        handleCharacteristicData(characteristic.value, status, gatt)
    }

    private fun handleCharacteristicData(data: ByteArray, status: Int, gatt: BluetoothGatt) {
        if (status == BluetoothGatt.GATT_SUCCESS) {
            internal.parseDiscoveryMessage(data, gatt.device.address)
            gatt.disconnect()
//            isBusy = false
//            discoveredPeripherals.remove(gatt.device)
        }
    }

    private fun subscribeToCharacteristic(gatt: BluetoothGatt, characteristic: BluetoothGattCharacteristic) {
        if (characteristic.properties and BluetoothGattCharacteristic.PROPERTY_NOTIFY != 0) {
            characteristic.writeType = BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT

            gatt.setCharacteristicNotification(characteristic, true)
            val uuid = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")
            val descriptor = characteristic.getDescriptor(uuid)
            descriptor.setValue(BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)
            gatt.writeDescriptor(descriptor)
        }
    }
}

class BLECentralManager(private val context: Context, private val internal: InternalDiscovery) : BleDiscoveryImplementationDelegate {
    private val bluetoothAdapter: BluetoothManager by lazy {
        context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    }

    private var scanJob: Job? = null
    private val scanIntervalMillis = 3000L
    private val pauseBetweenScans = 1000L

    companion object {
        var discoveredPeripherals = mutableListOf<BluetoothDevice>()
    }

    override fun startScanning() {
        discoveredPeripherals.clear()

        if (ActivityCompat.checkSelfPermission(context, Manifest.permission.BLUETOOTH_SCAN) != PackageManager.PERMISSION_GRANTED) {
            throw BlePermissionNotGrantedException()
        }

        val scanFilter: List<ScanFilter> = listOf(
            ScanFilter.Builder()
                .setServiceUuid(ParcelUuid(discoveryServiceUUID))
                .build()
        )

        val settings = ScanSettings.Builder()
            .setLegacy(false)
            .setPhy(ScanSettings.PHY_LE_ALL_SUPPORTED)
            .setNumOfMatches(ScanSettings.MATCH_NUM_ONE_ADVERTISEMENT)
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
            .setMatchMode(ScanSettings.MATCH_MODE_AGGRESSIVE)
            .setReportDelay(0L)
            .build()

        scanJob = CoroutineScope(Dispatchers.IO).launch {
            while (isActive) {
                bluetoothAdapter.adapter.bluetoothLeScanner.startScan(null, settings, leScanCallback)
                delay(scanIntervalMillis)
                discoveredPeripherals.clear()
                bluetoothAdapter.adapter.bluetoothLeScanner.stopScan(leScanCallback)
                delay(pauseBetweenScans)
            }
        }
    }

    override fun stopScanning() {
        if (ActivityCompat.checkSelfPermission(context, Manifest.permission.BLUETOOTH_SCAN) != PackageManager.PERMISSION_GRANTED) {
            throw BlePermissionNotGrantedException()
        }

        scanJob?.cancel()
        bluetoothAdapter.adapter.bluetoothLeScanner.stopScan(leScanCallback)
    }

    @SuppressLint("MissingPermission")
    private val leScanCallback: ScanCallback = object : ScanCallback() {
        fun addDevice(device: BluetoothDevice) {
            if (!discoveredPeripherals.contains(device)) {
                discoveredPeripherals.add(device)

                device.connectGatt(
                    context,
                    false,
                    BluetoothGattCallbackImplementation(internal, discoveredPeripherals),
                    BluetoothDevice.TRANSPORT_LE,
                    BluetoothDevice.PHY_LE_2M_MASK
                )
            }
        }


        override fun onScanResult(callbackType: Int, result: ScanResult) {
            addDevice(result.device)
        }

        override fun onBatchScanResults(results: List<ScanResult>) {
            results.forEach { result ->
                addDevice(result.device)
            }
        }
    }
}
