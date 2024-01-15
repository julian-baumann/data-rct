package com.julian_baumann.data_rctapp

import android.Manifest
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Phone
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
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
        bleAdvertisePermissionActivity.launch(Manifest.permission.BLUETOOTH_ADVERTISE)
    }

    private val accessLocationPermissionActivity = registerForActivityResult(ActivityResultContracts.RequestPermission()) { isGranted: Boolean ->
        accessLocationPermissionGranted = isGranted
        startServer()
    }

    private val bleAdvertisePermissionActivity = registerForActivityResult(ActivityResultContracts.RequestPermission()) { isGranted: Boolean ->
        bluetoothAdvertisePermissionGranted = isGranted
        accessLocationPermissionActivity.launch(Manifest.permission.ACCESS_FINE_LOCATION)
    }

    private val bleScanPermissionActivity = registerForActivityResult(ActivityResultContracts.RequestPermission()) { isGranted: Boolean ->
        bluetoothScanPermissionGranted = isGranted
        bleConnectPermissionActivity.launch(Manifest.permission.BLUETOOTH_CONNECT)
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

//        advertisement = NearbyServer(baseContext, device)
//        advertisement?.start()

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

        bleScanPermissionActivity.launch(Manifest.permission.BLUETOOTH_SCAN)

        setContent {
            DataRCTTheme {
                // A surface container using the 'background' color from the theme
                StartView(devices)
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun StartView(devices: List<Device>) {
    val scrollBehavior = TopAppBarDefaults.pinnedScrollBehavior(rememberTopAppBarState())

    Scaffold(
        modifier = Modifier.nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            MediumTopAppBar(
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    titleContentColor = MaterialTheme.colorScheme.primary,
                ),
                title = {
                    Text(
                        "DataRCT",
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                },
                scrollBehavior = scrollBehavior
            )
        },
    ) { innerPadding ->
        Surface(
            modifier = Modifier.fillMaxSize().padding(innerPadding),
            color = MaterialTheme.colorScheme.background
        ) {
            DeviceList(devices)
        }
    }
}

@Composable
fun DeviceList(devices: List<Device>) {
    LazyColumn(
        modifier = Modifier
            .padding(Dp(10F))
    ) {
        items(devices) { device ->
            ListItem(
                headlineContent = { Text(device.name) },
                leadingContent = {
                    Icon(
                        Icons.Default.Phone,
                        contentDescription = "Phone",
                    )
                }
            )

            Divider()
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
