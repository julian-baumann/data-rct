package com.julian_baumann.data_rctapp

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.julian_baumann.data_rct.ReceiveProgressDelegate
import com.julian_baumann.data_rct.ReceiveProgressState

class ReceiveProgress : ReceiveProgressDelegate {
    var state: ReceiveProgressState by mutableStateOf(ReceiveProgressState.Unknown)

    override fun progressChanged(progress: ReceiveProgressState) {
        state = progress
    }
}
