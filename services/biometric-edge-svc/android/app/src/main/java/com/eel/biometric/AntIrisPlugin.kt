/// AntIris iris scanner Kotlin plugin (real SDK integration, M05 M3).
/// Wraps USBControl (AntUSBCamera.jar) for USB connection + capture.
/// SDK source: ./sdks/AntIris_Android_SDK/
/// Hardware: AntIris iris scanner (K1/K2/K3/K7 capture modes).

package com.eel.biometric

import android.app.Activity
import android.content.Context
import android.util.Log
import android.view.TextureView
import com.antiris.AntUSBCamera.*
import com.antiris.AntUSBCamera.iso.IrisIso
import com.antiris.AntUSBCamera.utils.Log as AntLog
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.embedding.engine.plugins.activity.ActivityAware
import io.flutter.embedding.engine.plugins.activity.ActivityPluginBinding
import io.flutter.plugin.common.MethodChannel
import java.nio.charset.StandardCharsets

class AntIrisPlugin : FlutterPlugin, ActivityAware, MethodChannel.MethodCallHandler {
  private val TAG = "AntIrisPlugin"
  private val CHANNEL = "eel.bvi/iris"
  private var activity: Activity? = null
  private var usbControl: USBControl? = null
  private var captureView: TextureView? = null
  private var context: Context? = null

  override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
    when (call.method) {
      "init" -> {
        val act = activity ?: return result.error("NO_ACTIVITY", "Activity required for iris scanner preview", null)
        captureView = TextureView(act)
        usbControl = USBControl.getInstance()
        val callbacks = object : DeviceConnectCallback {
          override fun onAttachDev(device: android.hardware.usb.UsbDevice) {
            Log.i(TAG, "onAttachDev")
          }
          override fun onDettachDev(device: android.hardware.usb.UsbDevice) {
            Log.i(TAG, "onDettachDev")
          }
          override fun onConnectDev(device: android.hardware.usb.UsbDevice, isConnected: Boolean) {
            Log.i(TAG, "onConnectDev connected=$isConnected")
          }
          override fun onDisConnectDev(device: android.hardware.usb.UsbDevice) {}
          override fun onPreview(previewing: Boolean) {
            Log.i(TAG, "onPreview=$previewing")
          }
        }
        val authCallback = object : AuthCallback {
          override fun onAuthListenerCallback(isSuccess: Boolean) {
            Log.i(TAG, "auth=$isSuccess")
          }
        }
        usbControl?.init(act, captureView, callbacks, true)
        result.success(mapOf("initialized" to true, "cameraOpened" to (usbControl?.isCameraOpened ?: false)))
      }
      "captureIris" -> {
        val userName = call.argument<String>("userName") ?: "user"
        val eyeIndex = call.argument<Int>("eyeIndex") ?: 0 // 0=left, 1=right
        val irexType = call.argument<Int>("irexType") ?: 7
        val timeoutMs = call.argument<Int>("timeoutMs") ?: 30_000
        val uc = usbControl ?: return result.error("NOT_INITIALIZED", "Call init() first", null)
        if (!uc.isCameraOpened) return result.error("CAMERA_NOT_OPEN", "AntIris camera not open", null)
        if (uc.isIrexCapturing) {
          uc.antStopIrexCapture(true)
          return result.error("ALREADY_CAPTURING", "Capture already in progress", null)
        }
        uc.antIrisIrexCapture(userName, eyeIndex, irexType, false, timeoutMs, object : IrexCaptureCallback {
          override fun onSuccess(eyeIndex: Int, userName: String?, temp: ByteArray?) {
            val k7Image = uc.getK7Image(eyeIndex)
            val w = uc.getK7ImgWidth(eyeIndex)
            val h = uc.getK7ImgHeight(eyeIndex)
            val q = uc.getk7Quality(eyeIndex)
            val jp2Size = IntArray(1)
            val jp2Data = ByteArray(w * h)
            uc.grayToJp2(k7Image, w, h, jp2Data, jp2Size, irexType, uc.ANT_COMPRESSION_MODE_LOSSY)
            val finalJp2 = ByteArray(jp2Size[0])
            System.arraycopy(jp2Data, 0, finalJp2, 0, jp2Size[0])
            val iso = IrisIso.generate(
              0, finalJp2, w, h,
              uc.cameraSerialNumber?.toByteArray(StandardCharsets.UTF_8) ?: ByteArray(0),
              q, irexType, uc.ANT_IMAGE_FORMAT_JPEG2000,
              uc.ANT_COMPRESSION_MODE_LOSSY, 8
            )
            result.success(mapOf(
              "isValid" to true,
              "imageBase64" to android.util.Base64.encodeToString(k7Image, android.util.Base64.NO_WRAP),
              "isoBase64" to android.util.Base64.encodeToString(iso, android.util.Base64.NO_WRAP),
              "width" to w,
              "height" to h,
              "quality" to q,
              "irexType" to irexType
            ))
          }
          override fun onError(msg: String?) {
            result.success(mapOf("isValid" to false, "error" to (msg ?: "unknown")))
          }
          override fun onProcess(eyeNum: Int, progress: Int, distance: Int, opening: Int) {
            Log.d(TAG, "capture progress=$progress distance=${distance}cm")
          }
        })
      }
      "stopCapture" -> {
        usbControl?.antStopIrexCapture(true)
        result.success("stopped")
      }
      "selfCheck" -> {
        val opened = usbControl?.isCameraOpened ?: false
        result.success(opened)
      }
      "release" -> {
        usbControl?.release()
        result.success("released")
      }
      else -> result.notImplemented()
    }
  }

  override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
    context = binding.applicationContext
    val channel = MethodChannel(binding.binaryMessenger, CHANNEL)
    channel.setMethodCallHandler(this)
  }

  override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {}

  override fun onAttachedToActivity(binding: ActivityPluginBinding) { activity = binding.activity }
  override fun onDetachedFromActivityForConfigChanges() { activity = null }
  override fun onReattachedToActivityForConfigChanges(binding: ActivityPluginBinding) { activity = binding.activity }
  override fun onDetachedFromActivity() { activity = null }
}
