package com.julian_baumann.data_rctapp

import android.Manifest
import android.content.Context
import android.os.Build
import android.os.Bundle
import android.os.Environment
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.RequiresApi
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.preferencesDataStore
import com.julian_baumann.data_rct.*
import com.julian_baumann.data_rctapp.ui.theme.DataRCTTheme
import com.julian_baumann.data_rctapp.views.NameChangeDialog
import com.julian_baumann.data_rctapp.views.ReceiveContentView
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.util.*

val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "settings")

class MainActivity : ComponentActivity(), DiscoveryDelegate, NearbyConnectionDelegate {
    private val devices = mutableStateListOf<Device>()
    private var nearbyServer: NearbyServer? = null
    private var currentConnectionRequest: ConnectionRequest? = null
    private var showConnectionRequest by mutableStateOf(false)
    private var showReceivingSheet by mutableStateOf(false)
    private var receiveProgress: ReceiveProgress? = null
    private val userPreferencesManager = remember { UserPreferencesManager(baseContext) }

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

    @OptIn(ExperimentalMaterial3Api::class)
    @RequiresApi(Build.VERSION_CODES.S)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        bleScanPermissionActivity.launch(Manifest.permission.BLUETOOTH_SCAN)

        setContent {
            DataRCTTheme {
                // A surface container using the 'background' color from the theme
                StartView(userPreferencesManager)

                if (showReceivingSheet && receiveProgress != null && currentConnectionRequest != null) {
                    ModalBottomSheet(
                        onDismissRequest = {
                            showReceivingSheet = false
                        }
                    ) {
                        ReceiveContentView(receiveProgress!!, currentConnectionRequest)
                    }
                }

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
                                    receiveProgress = ReceiveProgress()
                                    currentConnectionRequest?.setProgressDelegate(receiveProgress!!)
                                    showReceivingSheet = true

                                    Thread {
                                        currentConnectionRequest?.accept()
                                    }.start()
                                }
                            ) {
                                Text("Accept")
                            }
                        },
                        dismissButton = {
                            TextButton(
                                onClick = {
                                    showConnectionRequest = false
                                    currentConnectionRequest?.decline()
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
fun StartView(userPreferencesManager: UserPreferencesManager) {
    val scrollBehavior = TopAppBarDefaults.pinnedScrollBehavior(rememberTopAppBarState())

    Scaffold(
        modifier = Modifier.nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            LargeTopAppBar(
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
                actions = {

//                    val visibility = remember { mutableStateOf(false) }
//                    val options = listOf("Visible", "Hidden")
//                    val icons = listOf(
//                        Icons.Filled.Visibility,
//                        Icons.Filled.VisibilityOff
//                    )
//
//                    SingleChoiceSegmentedButtonRow(
//                        modifier = Modifier.padding(20.dp)
//                    ) {
//                        options.forEachIndexed { index, label ->
//                            SegmentedButton(
//                                shape = SegmentedButtonDefaults.itemShape(index = index, count = options.size),
//                                icon = {
//                                    SegmentedButtonDefaults.Icon(active = index in visibility) {
//                                        Icon(
//                                            imageVector = icons[index],
//                                            contentDescription = null,
//                                            modifier = Modifier.size(SegmentedButtonDefaults.IconSize)
//                                        )
//                                    }
//                                },
//                                onClick = {
//                                },
//                                selected = true
//                            ) {
//                                Text(label)
//                            }
//                        }
//                    }
                },
                scrollBehavior = scrollBehavior
            )
        },
    ) { innerPadding ->
        Surface(
            modifier = Modifier.fillMaxSize().padding(innerPadding),
            color = MaterialTheme.colorScheme.background
        ) {
            Column(modifier = Modifier.padding(PaddingValues(top = 0.dp, start = 18.dp))) {
                NameChangeDialog(userPreferencesManager = userPreferencesManager)
            }

            Column(
                modifier = Modifier.fillMaxSize().padding(20.dp),
                verticalArrangement = Arrangement.Bottom,
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    text = "Share",
                    fontWeight = FontWeight.Bold,
                    fontSize = 15.sp,
                    modifier = Modifier
                        .padding(bottom = 10.dp)
                        .alpha(0.8F)
                )

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Button(
                        onClick = { /* TODO: Handle image or video sharing */ },
                        modifier = Modifier.weight(1f).height(60.dp),
                        colors = ButtonDefaults.buttonColors(MaterialTheme.colorScheme.secondary)
                    ) {
                        Text("Image or Video")
                    }

                    Spacer(modifier = Modifier.width(10.dp))

                    Button(
                        onClick = { /* TODO: Handle file sharing */ },
                        modifier = Modifier.weight(1f).height(60.dp),
                        colors = ButtonDefaults.buttonColors(MaterialTheme.colorScheme.secondary)
                    ) {
                        Text("File")
                    }
                }

                Spacer(modifier = Modifier.height(10.dp))

                FilledTonalButton(
                    onClick = { /* TODO: Handle file sharing */ },
                    modifier = Modifier.fillMaxWidth().height(60.dp)
                ) {
                    Text("Show received files")
                }
            }
        }
    }
}
