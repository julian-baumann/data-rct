package com.julian_baumann.data_rctapp.views

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.julian_baumann.data_rctapp.UserPreferencesManager
import kotlinx.coroutines.launch

@Composable
fun NameChangeDialog(userPreferencesManager: UserPreferencesManager) {
    val scope = rememberCoroutineScope()
    var showDialog by remember { mutableStateOf(false) }
    var userName by remember { mutableStateOf("") }
    var saveButtonEnabled by remember { mutableStateOf(false) }

    // Listen to userNameFlow to update userName state
    LaunchedEffect(key1 = true) {
        userPreferencesManager.deviceNameFlow.collect { name ->
            userName = name ?: ""
            saveButtonEnabled = userName.length >= 3

            if (userName.length < 3) {
                showDialog = true
            }
        }
    }

    Row(verticalAlignment = Alignment.CenterVertically) {
        Text("Device name:")
        TextButton(onClick = { showDialog = true }) {
            Text(userName)
        }
    }

    if (showDialog) {
        AlertDialog(
            onDismissRequest = {
            },
            title = { Text(text = "Name this device") },
            text = {
                Column {
                    Text("Nearby devices will discover this device using this name. Must be at least three characters long.", modifier = Modifier.padding(
                        PaddingValues(bottom = 20.dp)
                    ))
                    TextField(
                        value = userName,
                        singleLine = true,
                        onValueChange = { newName ->
                            userName = newName
                            saveButtonEnabled = userName.length >= 3
                        },
                        label = { Text("Device Name") }
                    )
                }
            },
            confirmButton = {
                Button(
                    enabled = saveButtonEnabled,
                    onClick = {
                        scope.launch {
                            userPreferencesManager.saveDeviceName(userName)
                            showDialog = false
                        }
                    }
                ) {
                    Text("Save")
                }
            }
        )
    }
}
