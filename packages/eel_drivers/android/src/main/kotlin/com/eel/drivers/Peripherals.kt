package com.eel.drivers

import android.app.Activity
import android.content.Context
import android.hardware.usb.UsbConstants
import android.hardware.usb.UsbDevice
import android.util.Log
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel

/**
 * USB thermal printers: any device exposing a printer-class (7) interface.
 * printTestPage sends a minimal ESC/POS page (init, text, cut) over the
 * first bulk-OUT endpoint. Nothing is sent without attached hardware.
 */
internal class PrinterHandler : DriverHandler {
  override fun handle(context: Context?, activity: Activity?, call: MethodCall, result: MethodChannel.Result) {
    if (context == null) return result.error("NO_CONTEXT", "no context", null)
    when (call.method) {
      "probeDetails" -> {
        val found = Usb.withClass(context, UsbConstants.USB_CLASS_PRINTER)
        val reason = if (found.isEmpty()) "no-usb-printer-attached"
        else if (found.none { Usb.usable(context, it) }) "usb-permission-requested"
        else "ready"
        result.success(Usb.detail(found, context, reason))
      }
      "isAvailable" -> {
        val found = Usb.withClass(context, UsbConstants.USB_CLASS_PRINTER)
        result.success(found.any { Usb.usable(context, it) })
      }
      "printTestPage" -> result.success(printTestPage(context))
      else -> result.notImplemented()
    }
  }

  private fun printTestPage(context: Context): Boolean {
    val manager = Usb.manager(context) ?: return false
    val device = Usb.withClass(context, UsbConstants.USB_CLASS_PRINTER)
      .firstOrNull { Usb.usable(context, it) } ?: return false
    var connection: android.hardware.usb.UsbDeviceConnection? = null
    return try {
      connection = manager.openDevice(device) ?: return false
      for (i in 0 until device.interfaceCount) {
        val iface = device.getInterface(i)
        if (iface.interfaceClass != UsbConstants.USB_CLASS_PRINTER) continue
        if (!connection.claimInterface(iface, true)) continue
        try {
          for (e in 0 until iface.endpointCount) {
            val ep = iface.getEndpoint(e)
            if (ep.direction == UsbConstants.USB_DIR_OUT &&
              ep.type == UsbConstants.USB_ENDPOINT_XFER_BULK
            ) {
              val page = escposTestPage()
              val sent = connection.bulkTransfer(ep, page, page.size, 5000)
              return sent == page.size
            }
          }
        } finally {
          connection.releaseInterface(iface)
        }
      }
      false
    } catch (e: Exception) {
      Log.w("EelDrivers", "print failed: ${e.message}")
      false
    } finally {
      try { connection?.close() } catch (_: Exception) {}
    }
  }

  private fun escposTestPage(): ByteArray {
    val out = mutableListOf<Byte>()
    fun bytes(vararg b: Int) = b.forEach { out += it.toByte() }
    fun text(s: String) = out.addAll(s.toByteArray(Charsets.UTF_8).toList())
    bytes(0x1B, 0x40) // init
    text("EEL PRINTER TEST\n")
    text("factory validation page\n")
    text("\n\n\n")
    bytes(0x1D, 0x56, 0x00) // cut
    return out.toByteArray()
  }
}

/**
 * Document scanner slot. No vendor scanner SDK is bundled, so availability
 * is driven by a known VID/PID table (currently empty — first vendor
 * integration fills it). HID-class devices are enumerated in probe details
 * for factory visibility.
 */
internal class ScannerHandler : DriverHandler {
  /** Known scanner USB identities. Empty until the first vendor plugin lands. */
  private val known: List<Pair<Int, Int>> = emptyList()

  override fun handle(context: Context?, activity: Activity?, call: MethodCall, result: MethodChannel.Result) {
    if (context == null) return result.error("NO_CONTEXT", "no context", null)
    when (call.method) {
      "probeDetails" -> {
        val found = known.flatMap { (vid, pid) -> Usb.withVidPid(context, vid, pid) }
        val hid = Usb.withClass(context, UsbConstants.USB_CLASS_HID)
        val detail = Usb.detail(found, context, "no-supported-scanner-attached").toMutableMap()
        detail["hidDevices"] = Usb.all(context).filter { d ->
          @Suppress("UNCHECKED_CAST")
          (d["interfaces"] as List<Map<String, Any?>>).any { it["class"] == UsbConstants.USB_CLASS_HID }
        }
        detail["knownTable"] = known.map { mapOf("vid" to it.first, "pid" to it.second) }
        result.success(detail)
      }
      "isAvailable" -> {
        val found = known.flatMap { (vid, pid) -> Usb.withVidPid(context, vid, pid) }
        result.success(found.any { Usb.usable(context, it) })
      }
      "scanDocument" -> result.success(
        mapOf("success" to false, "error" to "scanner-vendor-plugin-not-bundled"),
      )
      else -> result.notImplemented()
    }
  }
}
