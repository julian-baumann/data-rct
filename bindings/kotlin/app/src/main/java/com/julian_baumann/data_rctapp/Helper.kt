package com.julian_baumann.data_rctapp

import kotlin.math.log10
import kotlin.math.pow

fun toHumanReadableSize(bytes: ULong?): String {
    if (bytes == 0UL || bytes == null) {
        return "0 B"
    }

    val units = arrayOf("B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB")
    val digitGroups = (log10(bytes.toDouble()) / log10(1024.0)).toInt()

    return String.format("%.2f %s", bytes.toDouble() / 1024.0.pow(digitGroups.toDouble()), units[digitGroups])
}
