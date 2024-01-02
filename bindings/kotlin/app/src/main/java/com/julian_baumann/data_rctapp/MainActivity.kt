package com.julian_baumann.data_rctapp

import android.Manifest
import android.content.pm.PackageManager
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
import androidx.core.app.ActivityCompat
import com.julian_baumann.data_rct.BleAdvertisement
import com.julian_baumann.data_rct.Device
import com.julian_baumann.data_rct.Discovery
import com.julian_baumann.data_rct.DiscoveryDelegate
import com.julian_baumann.data_rctapp.ui.theme.DataRCTTheme

class MainActivity : ComponentActivity(), DiscoveryDelegate {
    private val devices = mutableStateListOf<Device>()

    init {
        val device = Device(id = "", name = "Android Device", deviceType = 0)

        registerForActivityResult(ActivityResultContracts.RequestPermission()) { isGranted: Boolean ->
            if (isGranted) {
                println("Permission granted!")

                val advertisement = BleAdvertisement(baseContext, device)
                advertisement.start()
            }
        }

        ActivityCompat.requestPermissions(this, arrayOf(Manifest.permission.BLUETOOTH_ADVERTISE), 0)

//        val discovery = Discovery(delegate = this)
//        discovery.start()
    }

    override fun deviceAdded(value: Device) {
        println("Device discovered")
        devices.add(value)
    }

    override fun deviceRemoved(deviceId: String) {
        println("Device was removed")
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            DataRCTTheme {
                // A surface container using the 'background' color from the theme
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    Greeting("Android")
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
