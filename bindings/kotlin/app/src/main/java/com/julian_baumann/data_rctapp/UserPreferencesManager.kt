package com.julian_baumann.data_rctapp

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

class UserPreferencesManager(private val context: Context) {
    companion object {
        private val DEVICE_NAME_KEY = stringPreferencesKey("device_name")
        private val DEVICE_ID_KEY = stringPreferencesKey("device_id")
    }

    val deviceNameFlow: Flow<String?> = context.dataStore.data
        .map { preferences ->
            preferences[DEVICE_NAME_KEY]
        }

    val deviceIdFlow: Flow<String?> = context.dataStore.data
        .map { preferences ->
            preferences[DEVICE_ID_KEY]
        }

    suspend fun saveDeviceId(id: String) {
        context.dataStore.edit { preferences ->
            preferences[DEVICE_ID_KEY] = id
        }
    }

    suspend fun saveDeviceName(name: String) {
        context.dataStore.edit { preferences ->
            preferences[DEVICE_NAME_KEY] = name
        }
    }
}
