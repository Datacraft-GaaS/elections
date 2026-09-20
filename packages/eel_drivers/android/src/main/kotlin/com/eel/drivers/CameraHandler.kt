package com.eel.drivers

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.pm.PackageManager
import android.graphics.ImageFormat
import android.hardware.camera2.CameraCharacteristics
import android.hardware.camera2.CameraDevice
import android.hardware.camera2.CameraManager
import android.hardware.camera2.CaptureRequest
import android.media.ImageReader
import android.os.Handler
import android.os.HandlerThread
import android.util.Base64
import android.util.Log
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

/**
 * Camera2 still-image capture. No third-party deps. Presence = at least one
 * camera id; capture opens the (back-preferred) camera, grabs one JPEG via
 * ImageReader, closes everything. Bounded by a 10 s latch — never hangs.
 */
internal class CameraHandler : DriverHandler {
  override fun handle(context: Context?, activity: Activity?, call: MethodCall, result: MethodChannel.Result) {
    if (context == null) return result.error("NO_CONTEXT", "no context", null)
    when (call.method) {
      "probeDetails" -> {
        val ids = cameraIds(context)
        result.success(
          mapOf(
            "present" to ids.isNotEmpty(),
            "reason" to if (ids.isEmpty()) "no-camera-device" else "ready",
            "cameraIds" to ids,
            "devices" to emptyList<Any>(),
          ),
        )
      }
      "isAvailable" -> result.success(cameraIds(context).isNotEmpty())
      "captureImage" -> {
        val width = (call.argument<Int>("width") ?: 640).coerceIn(160, 1920)
        val height = (call.argument<Int>("height") ?: 480).coerceIn(120, 1080)
        result.success(capture(context, width, height))
      }
      else -> result.notImplemented()
    }
  }

  private fun manager(context: Context): CameraManager? =
    context.getSystemService(Context.CAMERA_SERVICE) as? CameraManager

  private fun cameraIds(context: Context): List<String> {
    return try {
      manager(context)?.cameraIdList?.toList() ?: emptyList()
    } catch (e: Exception) {
      Log.w("EelDrivers", "camera enumeration failed: ${e.message}")
      emptyList()
    }
  }

  private fun pickId(context: Context): String? {
    val mgr = manager(context) ?: return null
    return try {
      val ids = mgr.cameraIdList
      ids.firstOrNull {
        mgr.getCameraCharacteristics(it).get(CameraCharacteristics.LENS_FACING) ==
          CameraCharacteristics.LENS_FACING_BACK
      } ?: ids.firstOrNull()
    } catch (e: Exception) {
      null
    }
  }

  private fun capture(context: Context, width: Int, height: Int): Map<String, Any?> {
    if (context.checkSelfPermission(Manifest.permission.CAMERA) !=
      PackageManager.PERMISSION_GRANTED
    ) {
      return mapOf("success" to false, "error" to "camera-permission-denied")
    }
    val id = pickId(context) ?: return mapOf("success" to false, "error" to "no-camera-device")
    val thread = HandlerThread("EelDriversCamera").also { it.start() }
    val handler = Handler(thread.looper)
    val reader = ImageReader.newInstance(width, height, ImageFormat.JPEG, 2)
    var device: CameraDevice? = null
    var session: android.hardware.camera2.CameraCaptureSession? = null
    return try {
      val opened = CountDownLatch(1)
      var openError: String? = null
      try {
        manager(context)?.openCamera(
          id,
          object : CameraDevice.StateCallback() {
            override fun onOpened(camera: CameraDevice) {
              device = camera
              opened.countDown()
            }
            override fun onDisconnected(camera: CameraDevice) {
              openError = "camera-disconnected"
              opened.countDown()
            }
            override fun onError(camera: CameraDevice, error: Int) {
              openError = "camera-error-$error"
              opened.countDown()
            }
          },
          handler,
        )
      } catch (e: SecurityException) {
        return mapOf("success" to false, "error" to "camera-permission-denied")
      }
      if (!opened.await(10, TimeUnit.SECONDS) || device == null) {
        return mapOf("success" to false, "error" to (openError ?: "camera-open-timeout"))
      }
      val captured = CountDownLatch(1)
      var jpeg: ByteArray? = null
      var captureError: String? = null
      reader.setOnImageAvailableListener({ r ->
        try {
          val image = r.acquireLatestImage() ?: return@setOnImageAvailableListener
          try {
            val buffer = image.planes[0].buffer
            val bytes = ByteArray(buffer.remaining())
            buffer.get(bytes)
            jpeg = bytes
          } finally {
            image.close()
          }
        } catch (e: Exception) {
          captureError = e.message
        } finally {
          captured.countDown()
        }
      }, handler)
      val dev = device!!
      val sessLatch = CountDownLatch(1)
      var sessError: String? = null
      @Suppress("DEPRECATION")
      dev.createCaptureSession(
        listOf(reader.surface),
        object : android.hardware.camera2.CameraCaptureSession.StateCallback() {
          override fun onConfigured(s: android.hardware.camera2.CameraCaptureSession) {
            session = s
            sessLatch.countDown()
          }
          override fun onConfigureFailed(s: android.hardware.camera2.CameraCaptureSession) {
            sessError = "session-configure-failed"
            sessLatch.countDown()
          }
        },
        handler,
      )
      if (!sessLatch.await(10, TimeUnit.SECONDS) || session == null) {
        return mapOf("success" to false, "error" to (sessError ?: "session-timeout"))
      }
      val request = dev.createCaptureRequest(CameraDevice.TEMPLATE_STILL_CAPTURE)
        .also { it.addTarget(reader.surface) }.build()
      val shot = CountDownLatch(1)
      session!!.capture(
        request,
        object : android.hardware.camera2.CameraCaptureSession.CaptureCallback() {
          override fun onCaptureCompleted(
            s: android.hardware.camera2.CameraCaptureSession,
            r: CaptureRequest,
            res: android.hardware.camera2.TotalCaptureResult,
          ) { shot.countDown() }
          override fun onCaptureFailed(
            s: android.hardware.camera2.CameraCaptureSession,
            r: CaptureRequest,
            f: android.hardware.camera2.CaptureFailure,
          ) {
            captureError = "capture-failed-reason-${f.reason}"
            shot.countDown()
          }
        },
        handler,
      )
      shot.await(10, TimeUnit.SECONDS)
      captured.await(3, TimeUnit.SECONDS)
      val bytes = jpeg
      if (bytes != null) {
        mapOf(
          "success" to true,
          "imageBytes" to Base64.encodeToString(bytes, Base64.NO_WRAP),
          "width" to width,
          "height" to height,
        )
      } else {
        mapOf("success" to false, "error" to (captureError ?: "no-image"))
      }
    } catch (e: Exception) {
      mapOf("success" to false, "error" to (e.message ?: "unknown"))
    } finally {
      try { session?.close() } catch (_: Exception) {}
      try { device?.close() } catch (_: Exception) {}
      try { reader.close() } catch (_: Exception) {}
      thread.quitSafely()
    }
  }
}
