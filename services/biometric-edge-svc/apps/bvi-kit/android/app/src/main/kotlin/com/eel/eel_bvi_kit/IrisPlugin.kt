package com.eel.eel_bvi_kit

import android.app.Activity
import android.util.Base64
import android.view.View
import com.antiris.AntUSBCamera.DeviceConnectCallback
import com.antiris.AntUSBCamera.EnrollCallback
import com.antiris.AntUSBCamera.IdentifyCallback
import com.antiris.AntUSBCamera.IrexCaptureCallback
import com.antiris.AntUSBCamera.USBControl
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

/**
 * Native bridge for the AntIris USB iris scanner (integration phase P2).
 *
 * Wraps `com.antiris.AntUSBCamera.USBControl` and speaks the `eel/iris`
 * MethodChannel consumed by `PlatformChannelIrisTransport`
 * (packages/kit-sdk-flutter/lib/src/iris_scanner.dart).
 *
 * The channel is bidirectional:
 *   Dart -> native:  connect | disconnect | enroll | identify | captureIso | stop
 *   native -> Dart:  connected | disconnected | error | enrolled | identified | captured
 */
class IrisPlugin(activity: Activity, engine: FlutterEngine) : DeviceConnectCallback {

    private val helper: USBControl = USBControl.getInstance()
    private val channel = MethodChannel(engine.dartExecutor.binaryMessenger, "eel/iris")

    init {
        // The SDK needs a View for its live preview. P3 replaces this with a
        // Flutter platform view (TextureView/SurfaceView); the content view is
        // a valid placeholder for the compile-verified scaffold.
        val preview: View = activity.findViewById(android.R.id.content)
        helper.init(activity, preview, this, false)

        channel.setMethodCallHandler { call, result ->
            when (call.method) {
                "connect" -> {
                    try {
                        helper.onOpen()
                        val id = helper.getCameraSerialNumber() ?: "antiris"
                        channel.invokeMethod("connected", id)
                        result.success(mapOf("device_id" to id))
                    } catch (e: Exception) {
                        result.error("CONNECT_FAILED", e.message, null)
                    }
                }
                "disconnect" -> {
                    helper.stopStream()
                    channel.invokeMethod("disconnected", null)
                    result.success(null)
                }
                "enroll" -> {
                    helper.antIrisEnroll("worker", null, eyeCode(call.argument<String>("eye")), 100_000L, enrollCallback)
                    result.success(null)
                }
                "identify" -> {
                    val eye = eyeCode(call.argument<String>("eye"))
                    // Enrolled gallery (base64 ISO-19794-6 templates) fetched
                    // from the ABIS gallery sync and passed down by Dart.
                    val templates = (call.argument<List<*>>("templates") ?: emptyList<Any>())
                        .mapNotNull { (it as? String)?.let { s -> Base64.decode(s, Base64.NO_WRAP) } }
                    val all = ByteArray(templates.sumOf { it.size })
                    var offset = 0
                    for (t in templates) {
                        t.copyInto(all, offset)
                        offset += t.size
                    }
                    helper.antIrisIdentify(eye, identifyCallback, all, templates.size)
                    result.success(null)
                }
                "captureIso" -> {
                    helper.antIrisIrexCapture("worker", eyeCode(call.argument<String>("eye")), 0, true, 100_000L, irexCallback)
                    result.success(null)
                }
                "stop" -> {
                    helper.antStopIdentify()
                    helper.antStopEnroll(false)
                    helper.antStopIrexCapture(false)
                    result.success(null)
                }
                else -> result.notImplemented()
            }
        }
    }

    private fun eyeCode(eye: String?): Int = when (eye) {
        "right" -> USBControl.RIGHT_EYE
        "both" -> USBControl.TWO_EYES
        else -> USBControl.LEFT_EYE
    }

    private fun eyeName(index: Int): String = when (index) {
        USBControl.RIGHT_EYE -> "right"
        USBControl.TWO_EYES -> "both"
        else -> "left"
    }

    private val enrollCallback = object : EnrollCallback {
        override fun onSuccess(eyeIndex: Int, userName: String, temp: ByteArray) {
            channel.invokeMethod("enrolled", mapOf(
                "eye" to eyeName(eyeIndex),
                "iso_base64" to Base64.encodeToString(temp, Base64.NO_WRAP),
            ))
        }

        override fun onError(msg: String) {
            channel.invokeMethod("error", msg)
        }

        override fun onProcess(eyeNum: Int, progress: Int, distance: Int, opening: Int) {
            // Progress events are optional for the minimal bridge.
        }
    }

    private val identifyCallback = object : IdentifyCallback {
        override fun onProcess(distance: Int, opening: Int) {}

        override fun onSuccess(distance: Int, uid: String) {
            channel.invokeMethod("identified", mapOf("user_id" to uid))
        }

        override fun onError(msg: String) {
            channel.invokeMethod("error", msg)
        }
    }

    private val irexCallback = object : IrexCaptureCallback {
        override fun onSuccess(eyeIndex: Int, userName: String, temp: ByteArray) {
            channel.invokeMethod("captured", mapOf(
                "eye" to eyeName(eyeIndex),
                "iso_base64" to Base64.encodeToString(temp, Base64.NO_WRAP),
            ))
        }

        override fun onError(msg: String) {
            channel.invokeMethod("error", msg)
        }

        override fun onProcess(eyeNum: Int, progress: Int, distance: Int, opening: Int) {}
    }

    override fun onConnectDev(device: android.hardware.usb.UsbDevice, isConnected: Boolean) {
        channel.invokeMethod(if (isConnected) "connected" else "disconnected", device.deviceName)
    }

    override fun onDisConnectDev(device: android.hardware.usb.UsbDevice) {
        channel.invokeMethod("disconnected", null)
    }

    override fun onAttachDev(device: android.hardware.usb.UsbDevice) {
        // USB attached; the connect handshake is driven by the `connect` call.
    }

    override fun onDettachDev(device: android.hardware.usb.UsbDevice) {
        channel.invokeMethod("disconnected", null)
    }

    override fun onPreview(isPreview: Boolean) {
        // Preview state is only surfaced through the platform view (P3).
    }
}
