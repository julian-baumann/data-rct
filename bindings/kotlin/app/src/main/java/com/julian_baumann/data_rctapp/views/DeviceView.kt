package com.julian_baumann.data_rctapp.views

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import com.julian_baumann.data_rct.Device


@Composable
fun DeviceItem(device: Device) {
    // Replace with your desired item layout
    Text("Device: ${device.name}")
}
