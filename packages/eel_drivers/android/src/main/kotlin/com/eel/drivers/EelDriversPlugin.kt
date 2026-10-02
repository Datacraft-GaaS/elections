/*
 * Copyright (c) 2026 Nyimbi Odero
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package com.eel.drivers

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.hardware.usb.UsbConstants
import android.hardware.usb.UsbDevice
import android.hardware.usb.UsbManager
import android.util.Log
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.embedding.engine.plugins.activity.ActivityAware
import io.flutter.embedding.engine.plugins.activity.ActivityPluginBinding
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel

/** Shared USB helpers: enumeration, permission, presence probes. */
internal object Usb {
  const val ACTION_PERMISSION = "com.eel.drivers.USB_PERMISSION"

  fun manager(context: Context): UsbManager? =
    context.getSystemService(Context.USB_SERVICE) as? UsbManager

  fun describe(device: UsbDevice, manager: UsbManager): Map<String, Any?> {
    val ifaces = (0 until device.interfaceCount).map { i ->
      val c = device.getInterface(i).interfaceClass
      mapOf("index" to i, "class" to c)
    }
    return mapOf(
      "vid" to device.vendorId,
      "pid" to device.productId,
      "class" to device.deviceClass,
      "productName" to (device.productName ?: ""),
      "hasPermission" to manager.hasPermission(device),
      "interfaces" to ifaces,
    )
  }

  fun all(context: Context): List<Map<String, Any?>> {
    val m = manager(context) ?: return emptyList()
    return m.deviceList.values.map { describe(it, m) }
  }

  /** Devices exposing an interface (or device) of [usbClass]. */
  fun withClass(context: Context, usbClass: Int): List<UsbDevice> {
    val m = manager(context) ?: return emptyList()
    return m.deviceList.values.filter { d ->
      d.deviceClass == usbClass ||
        (0 until d.interfaceCount).any { d.getInterface(it).interfaceClass == usbClass }
    }
  }

  fun withVidPid(context: Context, vid: Int, pid: Int): List<UsbDevice> {
    val m = manager(context) ?: return emptyList()
    return m.deviceList.values.filter { it.vendorId == vid && it.productId == pid }
  }

  /**
   * True when [device] may be used now. When present but permission is
   * missing, fires the permission request (result lands on a later call)
   * and returns false — callers surface reason `usb-permission-requested`.
   */
  fun usable(context: Context, device: UsbDevice): Boolean {
    val m = manager(context) ?: return false
    if (m.hasPermission(device)) return true
    return try {
      val intent = PendingIntent.getBroadcast(
        context, 0, Intent(ACTION_PERMISSION), PendingIntent.FLAG_MUTABLE,
      )
      m.requestPermission(device, intent)
      false
    } catch (e: Exception) {
      Log.w("EelDrivers", "usb permission request failed: ${e.message}")
      false
    }
  }

  fun detail(devices: List<UsbDevice>, context: Context, reason: String): Map<String, Any?> {
    val m = manager(context)
    return mapOf(
      "present" to devices.isNotEmpty(),
      "reason" to reason,
      "devices" to (m?.let { mgr -> devices.map { describe(it, mgr) } } ?: emptyList<Any>()),
    )
  }
}

/**
 * eel_drivers plugin: registers every hardware channel. Each channel also
 * serves `probeDetails` -> `{present, reason, devices}` so callers can tell
 * "no hardware attached" (SKIP) apart from "operation failed" (FAIL).
 */
class EelDriversPlugin : FlutterPlugin, ActivityAware, MethodChannel.MethodCallHandler {
  private var context: Context? = null
  private var activity: android.app.Activity? = null
  private val channels = mutableListOf<MethodChannel>()

  private val iris = IrisHandler()
  private val fingerprint = FingerprintHandler()
  private val printer = PrinterHandler()
  private val sms = SmsHandler()
  private val scanner = ScannerHandler()
  private val camera = CameraHandler()

  override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
    context = binding.applicationContext
    val names = mapOf(
      "eel.bvi/iris" to iris,
      "eel/iris" to iris,
      "eel/fingerprint" to fingerprint,
      "eel/bvi/printer" to printer,
      "eel/modem/sms" to sms,
      "eel/bvi/document_scanner" to scanner,
      "eel/bvi/camera" to camera,
    )
    for ((name, handler) in names) {
      val channel = MethodChannel(binding.binaryMessenger, name)
      channel.setMethodCallHandler { call, result ->
        handler.handle(context, activity, call, result)
      }
      channels += channel
    }
  }

  override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
    channels.forEach { it.setMethodCallHandler(null) }
    channels.clear()
    context = null
  }

  override fun onAttachedToActivity(binding: ActivityPluginBinding) { activity = binding.activity }
  override fun onDetachedFromActivityForConfigChanges() { activity = null }
  override fun onReattachedToActivityForConfigChanges(binding: ActivityPluginBinding) { activity = binding.activity }
  override fun onDetachedFromActivity() { activity = null }

  override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
    result.notImplemented()
  }
}

/** Common handler contract: pure functions of (context, activity, call). */
internal interface DriverHandler {
  fun handle(
    context: Context?,
    activity: android.app.Activity?,
    call: MethodCall,
    result: MethodChannel.Result,
  )
}
