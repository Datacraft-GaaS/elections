/// FAP60 fingerprint scanner Kotlin plugin (real SDK integration, M01 M3).
/// Wraps FUFingerSDK (USB scanner) + FPAPI (algorithm: capture/template/compare).
/// Per SDK docs: VID=0x0507, PID=0x0581; image 1600x1500; quality score 0-100.
/// Hardware: FAP60 fingerprint scanner.
/// SDK source: /Users/nyimbiodero/src/pjs/eel/sdks/2.fap60-android/

package com.eel.biometric

import android.content.Context
import android.util.Log
import cn.fingerup.*
import com.ZHIANG.*
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.embedding.engine.plugins.MethodChannel
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class Fap60Plugin(private val context: Context) : FlutterPlugin, MethodChannel.MethodCallHandler {
  private val TAG = "Fap60Plugin"
  private val CHANNEL = "eel.bvr/fingerprint"
  private var sdk: FUFingerSDK? = null
  private var fpApi: FPAPI? = null
  private var deviceHandle: Long = 0

  fun register(flutterPluginBinding: io.flutter.embedding.engine.plugins.FlutterPlugin.FlutterPluginBinding, binaryMessenger: io.flutter.plugin.common.BinaryMessenger) {
    val channel = MethodChannel(binaryMessenger, CHANNEL)
    channel.setMethodCallHandler(this)
  }

  override fun onMethodCall(call: MethodCall, result: MethodChannel.Result) {
    when (call.method) {
      "init" -> {
        val lcdMode = call.argument<Boolean>("lcdMode") ?: true
        val ret = FUFingerSDK(context).apply { sdk = this }.LIVESCAN_Init(lcdMode)
        fpApi = FPAPI(context.activity as? android.app.Activity ?: context as? android.app.Activity ? context : null)
        result.success(if (ret == 0) "init_ok" else "init_fail:$ret")
      }
      "captureFingerprint" -> {
        val position = call.argument<String>("position") ?: "LEFT_INDEX"
        val timeoutMs = call.argument<Int>("timeoutMs") ?: 5000
        val rawBuffer = ByteArray(1600 * 1500)
        val bmpBuffer = ByteArray(1078 + 1600 * 1500)
        try {
          val ret = sdk?.captureFinger(timeoutMs, rawBuffer, bmpBuffer) ?: 0
          val quality = fpApi?.F_GetImageQuality(deviceHandle, rawBuffer) ?: 0
          val template = ByteArray(1024)
          val templateSize = fpApi?.F_CreateTemplate(deviceHandle, rawBuffer, template) ?: 0
          if (ret == 0 && quality > 50 && templateSize > 0) {
            result.success(mapOf(
              "templateBase64" to android.util.Base64.encodeToString(template, 0, templateSize, android.util.Base64.NO_WRAP),
              "qualityScore" to quality,
              "fingerPosition" to position,
              "isValid" to true
            ))
          } else {
            result.success(mapOf("isValid" to false, "qualityScore" to quality, "error" to "capture_failed:$ret"))
          }
        } catch (e: Exception) {
          result.error("FAP60_ERROR", e.message, null)
        }
      }
      "selfCheck" -> {
        val ret = sdk?.LIVESCAN_GetVersion() ?: 0
        val openRet = fpApi?.F_OpenDevice() ?: 0L
        result.success(openRet > 0 && ret == 100)
      }
      "close" -> {
        sdk?.LIVESCAN_Close()
        fpApi?.F_CloseDevice(deviceHandle)
        result.success("closed")
      }
      else -> result.notImplemented()
    }
  }

  override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
    val messenger = binding.binaryMessenger
    val channel = MethodChannel(messenger, CHANNEL)
    val plugin = Fap60Plugin(binding.applicationContext)
    channel.setMethodCallHandler(plugin)
  }

  override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {}
}
