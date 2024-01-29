package com.julian_baumann.data_rctapp.views

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Phone
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import com.julian_baumann.data_rct.Device


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
