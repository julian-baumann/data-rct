package com.julian_baumann.data_rctapp

import android.Manifest
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import com.julian_baumann.data_rct.Device
import com.julian_baumann.data_rct.Discovery
import com.julian_baumann.data_rct.DiscoveryDelegate
import com.julian_baumann.data_rct.NearbyServer
import com.julian_baumann.data_rctapp.ui.theme.DataRCTTheme
import java.util.*

class MainActivity : ComponentActivity(), DiscoveryDelegate {
    private val devices = mutableStateListOf<Device>()
    private var advertisement: NearbyServer? = null
    private var discovery: Discovery? = null

    private var bluetoothConnectPermissionGranted = false
    private var bluetoothAdvertisePermissionGranted = false
    private var bluetoothScanPermissionGranted = false
    private var accessLocationPermissionGranted = false

    private val bleConnectPermissionActivity = registerForActivityResult(ActivityResultContracts.RequestPermission()) { isGranted: Boolean ->
        bluetoothConnectPermissionGranted = isGranted
        startServer()
    }

    private val accessLocationPermissionActivity = registerForActivityResult(ActivityResultContracts.RequestPermission()) { isGranted: Boolean ->
        accessLocationPermissionGranted = isGranted
        startServer()
    }

    private val bleAdvertisePermissionActivity = registerForActivityResult(ActivityResultContracts.RequestPermission()) { isGranted: Boolean ->
        bluetoothAdvertisePermissionGranted = isGranted
        startServer()
    }

    private val bleScanPermissionActivity = registerForActivityResult(ActivityResultContracts.RequestPermission()) { isGranted: Boolean ->
        bluetoothScanPermissionGranted = isGranted
        startServer()
    }

    private fun startServer() {
        if (!bluetoothConnectPermissionGranted
            || !bluetoothAdvertisePermissionGranted
            || !bluetoothScanPermissionGranted
            || !accessLocationPermissionGranted) {
            return
        }

        val device = Device(
            id = UUID.randomUUID().toString(),
            name = "Android Device",
            deviceType = 0
        )

        advertisement = NearbyServer(baseContext, device)
        advertisement?.start()

        discovery = Discovery(baseContext, this)
        discovery?.startScanning()
    }

    override fun deviceAdded(value: Device) {
        println("Device discovered")
        devices.add(value)
    }

    override fun deviceRemoved(deviceId: String) {
        println("Device was removed")
    }

    override fun onStop() {
        super.onStop()
        advertisement?.stop()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        bleConnectPermissionActivity.launch(Manifest.permission.BLUETOOTH_CONNECT)
        bleAdvertisePermissionActivity.launch(Manifest.permission.BLUETOOTH_ADVERTISE)
        accessLocationPermissionActivity.launch(Manifest.permission.ACCESS_FINE_LOCATION)
        bleScanPermissionActivity.launch(Manifest.permission.BLUETOOTH_SCAN)

        setContent {
            DataRCTTheme {
                // A surface container using the 'background' color from the theme
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    DeviceList(devices)
                }
            }
        }
    }
}

@Composable
fun DeviceList(devices: List<Device>) {
    LazyColumn {
        items(devices) { device ->
            DeviceItem(device)
        }
    }
}

@Composable
fun DeviceItem(device: Device) {
    // Replace with your desired item layout
    Text("Device: ${device.name}")
}


@Composable
fun Greeting(name: String, modifier: Modifier = Modifier) {
    Text(
        text = "Hello $name!",
        modifier = modifier
    )
}

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    DataRCTTheme {
        Greeting("Android")
    }
}
