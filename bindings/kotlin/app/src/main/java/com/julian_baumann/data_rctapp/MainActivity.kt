package com.julian_baumann.data_rctapp

import android.Manifest
import android.os.Build
import android.os.Bundle
import android.os.Environment
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.RequiresApi
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.text.style.TextOverflow
import com.julian_baumann.data_rct.*
import com.julian_baumann.data_rctapp.ui.theme.DataRCTTheme
import com.julian_baumann.data_rctapp.views.DeviceList
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.util.*
import kotlin.math.log10
import kotlin.math.pow


class MainActivity : ComponentActivity(), DiscoveryDelegate, NearbyConnectionDelegate {
    private val devices = mutableStateListOf<Device>()
    private var nearbyServer: NearbyServer? = null
    private var discovery: Discovery? = null
    private var currentConnectionRequest: ConnectionRequest? = null
    private var showConnectionRequest by mutableStateOf(false)

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

        val test = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS)
        println(test)

        nearbyServer = NearbyServer(baseContext, device, this)

        CoroutineScope(Dispatchers.Main).launch {
            nearbyServer?.start()
        }
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

        CoroutineScope(Dispatchers.Main).launch {
            nearbyServer?.stop()
        }
    }

    override fun receivedConnectionRequest(request: ConnectionRequest) {
        currentConnectionRequest = request
        showConnectionRequest = true
    }

    private fun toHumanReadableSize(bytes: ULong?): String {
        if (bytes == 0UL || bytes == null) {
            return "0 B"
        }

        val units = arrayOf("B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB")
        val digitGroups = (log10(bytes.toDouble()) / log10(1024.0)).toInt()

        return String.format("%.2f %s", bytes.toDouble() / 1024.0.pow(digitGroups.toDouble()), units[digitGroups])
    }

    @RequiresApi(Build.VERSION_CODES.S)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        bleScanPermissionActivity.launch(Manifest.permission.BLUETOOTH_SCAN)

        setContent {
            DataRCTTheme {
                // A surface container using the 'background' color from the theme
                StartView(devices)

                if (showConnectionRequest && currentConnectionRequest != null) {
                    AlertDialog(
                        title = {
                            Text(text = "${currentConnectionRequest?.getSender()?.name} wants to send you a file")
                        },
                        text = {
                            Text(text = "${currentConnectionRequest?.getFileTransferIntent()?.fileName} (${toHumanReadableSize(currentConnectionRequest?.getFileTransferIntent()?.fileSize)})")
                        },
                        onDismissRequest = {
                            showConnectionRequest = false

                            CoroutineScope(Dispatchers.Main).launch {
                                currentConnectionRequest?.decline()
                            }
                        },
                        confirmButton = {
                            TextButton(
                                onClick = {
                                    showConnectionRequest = false

                                    CoroutineScope(Dispatchers.Main).launch {
                                        currentConnectionRequest?.accept()
                                    }
                                }
                            ) {
                                Text("Accept")
                            }
                        },
                        dismissButton = {
                            TextButton(
                                onClick = {
                                    showConnectionRequest = false

                                    CoroutineScope(Dispatchers.Main).launch {
                                        currentConnectionRequest?.decline()
                                    }
                                }
                            ) {
                                Text("Decline")
                            }
                        }
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun StartView(devices: List<Device>) {
    val scrollBehavior = TopAppBarDefaults.pinnedScrollBehavior(rememberTopAppBarState())

    val showConnectionRequest by remember { mutableStateOf(false) }

    Scaffold(
        modifier = Modifier.nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            MediumTopAppBar(
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.background,
                    titleContentColor = MaterialTheme.colorScheme.primary,
                ),
                title = {
                    Text(
                        "InterShare",
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
