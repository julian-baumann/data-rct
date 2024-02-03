package com.julian_baumann.data_rctapp.views

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.LineBreak
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.julian_baumann.data_rct.ConnectionRequest
import com.julian_baumann.data_rct.ReceiveProgressState
import com.julian_baumann.data_rctapp.R
import com.julian_baumann.data_rctapp.ReceiveProgress
import com.julian_baumann.data_rctapp.toHumanReadableSize

@Composable
fun ReceiveContentView(progress: ReceiveProgress, connectionRequest: ConnectionRequest?) {
    Column(
        modifier = Modifier
            .padding(16.dp)
            .fillMaxWidth(),
        horizontalAlignment = Alignment.Start
    ) {
        Text(
            text = "Receiving file",
            fontWeight = FontWeight.Bold,
            fontSize = 25.sp,
            modifier = Modifier.padding(bottom = 8.dp)
        )

        Text(
            text = "File: ${connectionRequest?.getFileTransferIntent()?.fileName ?: "Unknown file"}",
            style = TextStyle(
                lineBreak = LineBreak.Paragraph
            )
        )

        Text(
            text = "Size: ${toHumanReadableSize(connectionRequest?.getFileTransferIntent()?.fileSize)}",
            style = TextStyle(
                lineBreak = LineBreak.Paragraph
            )
        )

        when (val state = progress.state) {
            is ReceiveProgressState.Receiving -> {
                LinearProgressIndicator(
                    progress = state.progress.toFloat(),
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(10.dp)
                        .padding(PaddingValues(bottom = 50.dp, top = 20.dp))
                )
            }
            ReceiveProgressState.Finished -> {
                Text(
                    text = "Done",
                    fontSize = 20.sp,
                    style = TextStyle(
                        color = Color.Green,
                        lineBreak = LineBreak.Paragraph
                    ),
                    textAlign = TextAlign.Center,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(PaddingValues(bottom = 50.dp, top = 20.dp))
                )
            }
            else -> {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally) {
                    CircularProgressIndicator(
                        modifier = Modifier.padding(PaddingValues(bottom = 45.dp, top = 20.dp)),
                        color = MaterialTheme.colorScheme.secondary,
                        trackColor = MaterialTheme.colorScheme.surfaceVariant,
                    )
                }
            }
        }
    }
}
