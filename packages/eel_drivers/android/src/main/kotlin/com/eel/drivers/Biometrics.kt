package com.eel.drivers

import android.app.Activity
import android.content.Context
import android.hardware.usb.UsbConstants
import android.hardware.usb.UsbDevice
import android.util.Base64
import android.util.Log
import android.view.TextureView
import com.antiris.AntUSBCamera.AuthCallback
import com.antiris.AntUSBCamera.DeviceConnectCallback
import com.antiris.AntUSBCamera.USBControl
import com.antiris.AntUSBCamera.iso.IrisIso
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.nio.charset.StandardCharsets

/**
 * AntIris iris scanner over USB (UVC video-class devices). Ported from the
 * reference `AntIrisPlugin` with two contract fixes the harness needs:
 * `init` returns Bool (was Map), and every entry point degrades to
 * `{isValid:false}` / `false` when no hardware is attached instead of
 * throwing. Vendor jars + .so ship inside this plugin (see ../libs,
 * ../jniLibs) — no device-side install required.
 */
internal class IrisHandler : DriverHandler {
  private var usbControl: USBControl? = null

  /** USB video-class devices — what UVC iris cameras present as. */
  private fun candidates(context: Context): List<UsbDevice> =
    Usb.withClass(context, UsbConstants.USB_CLASS_VIDEO)

  override fun handle(context: Context?, activity: Activity?, call: MethodCall, result: MethodChannel.Result) {
    if (context == null) return result.error("NO_CONTEXT", "no context", null)
    when (call.method) {
      "probeDetails" -> {
        val found = candidates(context)
        val reason = if (found.isEmpty()) "no-uvc-device-attached"
        else if (found.none { Usb.usable(context, it) }) "usb-permission-requested"
        else "ready"
        result.success(Usb.detail(found, context, reason))
      }
      "init" -> result.success(init(context, activity))
      "captureIris" -> {
        val userName = call.argument<String>("userName") ?: "user"
        val eyeIndex = call.argument<Int>("eyeIndex") ?: 0
        val irexType = call.argument<Int>("irexType") ?: 7
        val timeoutMs = call.argument<Int>("timeoutMs") ?: 30_000
        result.success(capture(userName, eyeIndex, irexType, timeoutMs))
      }
      "stopCapture" -> {
        try { usbControl?.antStopIrexCapture(true) } catch (_: Exception) {}
        result.success("stopped")
      }
      "release" -> {
        try { usbControl?.release() } catch (_: Exception) {}
        usbControl = null
        result.success("released")
      }
      else -> result.notImplemented()
    }
  }

  private fun init(context: Context, activity: Activity?): Boolean {
    val device = candidates(context).firstOrNull { Usb.usable(context, it) }
      ?: return false
    val act = activity ?: return false
    return try {
      val uc = USBControl.getInstance()
      val view = TextureView(act)
      uc.init(
        act, view,
        object : DeviceConnectCallback {
          override fun onAttachDev(d: UsbDevice) { Log.i("EelDrivers", "iris attach") }
          override fun onDettachDev(d: UsbDevice) { Log.i("EelDrivers", "iris detach") }
          override fun onConnectDev(d: UsbDevice, c: Boolean) { Log.i("EelDrivers", "iris connect=$c") }
          override fun onDisConnectDev(d: UsbDevice) {}
          override fun onPreview(p: Boolean) { Log.i("EelDrivers", "iris preview=$p") }
        },
        true,
      )
      usbControl = uc
      uc.isCameraOpened
    } catch (e: Exception) {
      Log.w("EelDrivers", "iris init failed: ${e.message}")
      false
    }
  }

  private fun capture(userName: String, eyeIndex: Int, irexType: Int, timeoutMs: Int): Map<String, Any?> {
    val uc = usbControl
    if (uc == null || !uc.isCameraOpened) {
      return mapOf("isValid" to false, "error" to "iris-not-initialized")
    }
    return try {
      if (uc.isIrexCapturing) {
        return mapOf("isValid" to false, "error" to "already-capturing")
      }
      var outcome: Map<String, Any?> = mapOf("isValid" to false, "error" to "capture-timeout")
      val done = java.util.concurrent.CountDownLatch(1)
      uc.antIrisIrexCapture(
        userName, eyeIndex, irexType, false, timeoutMs.toLong(),
        object : com.antiris.AntUSBCamera.IrexCaptureCallback {
          override fun onSuccess(eye: Int, name: String?, temp: ByteArray?) {
            outcome = try {
              val image = uc.getK7Image(eye)
              val w = uc.getK7ImgWidth(eye)
              val h = uc.getK7ImgHeight(eye)
              val q = uc.getk7Quality(eye)
              val jp2Size = IntArray(1)
              val jp2Data = ByteArray(w * h)
              uc.grayToJp2(
                image, w, h, jp2Data, jp2Size, irexType,
                USBControl.ANT_COMPRESSION_MODE_LOSSY.toInt(),
              )
              val finalJp2 = jp2Data.copyOf(jp2Size[0])
              val iso = IrisIso.generate(
                0, finalJp2, w, h,
                uc.getCameraSerialNumber()?.toByteArray(StandardCharsets.UTF_8) ?: ByteArray(0),
                q, irexType, USBControl.ANT_IMAGE_FORMAT_JPEG2000,
                USBControl.ANT_COMPRESSION_MODE_LOSSY.toInt(), 8,
              )
              mapOf(
                "isValid" to true,
                "imageBase64" to Base64.encodeToString(image, Base64.NO_WRAP),
                "isoBase64" to Base64.encodeToString(iso, Base64.NO_WRAP),
                "width" to w,
                "height" to h,
                "quality" to q,
                "irexType" to irexType,
              )
            } catch (e: Exception) {
              mapOf("isValid" to false, "error" to (e.message ?: "encode-failed"))
            }
            done.countDown()
          }
          override fun onError(msg: String?) {
            outcome = mapOf("isValid" to false, "error" to (msg ?: "unknown"))
            done.countDown()
          }
          override fun onProcess(eyeNum: Int, progress: Int, distance: Int, opening: Int) {
            Log.d("EelDrivers", "iris progress=$progress distance=${distance}cm")
          }
        },
      )
      done.await((timeoutMs + 5000).toLong(), java.util.concurrent.TimeUnit.MILLISECONDS)
      outcome
    } catch (e: Exception) {
      mapOf("isValid" to false, "error" to (e.message ?: "unknown"))
    }
  }
}

/**
 * FAP60 fingerprint slot. The vendor SDK jars (`cn.fingerup.*`,
 * `com.ZHIANG.*`) are NOT available, so capture can never succeed here:
 * `init` is false with reason `fap60-sdk-missing`, and presence tracks the
 * documented FAP60 USB identity (VID 0x0507 / PID 0x0581) so the factory can
 * see the reader even before the SDK lands. Drop the jars into
 * `android/libs` and implement capture when the vendor delivers.
 */
internal class FingerprintHandler : DriverHandler {
  private val fap60Vid = 0x0507
  private val fap60Pid = 0x0581

  override fun handle(context: Context?, activity: Activity?, call: MethodCall, result: MethodChannel.Result) {
    if (context == null) return result.error("NO_CONTEXT", "no context", null)
    when (call.method) {
      "probeDetails" -> {
        val found = Usb.withVidPid(context, fap60Vid, fap60Pid)
        val reason = if (found.isEmpty()) "no-fap60-attached" else "fap60-sdk-missing"
        val detail = Usb.detail(found, context, reason).toMutableMap()
        detail["sdkBundled"] = false
        result.success(detail)
      }
      "init" -> result.success(false)
      "captureFingerprint" -> result.success(
        mapOf("isValid" to false, "quality" to 0, "error" to "fap60-sdk-missing"),
      )
      else -> result.notImplemented()
    }
  }
}
