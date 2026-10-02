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

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.pm.PackageManager
import android.telephony.SmsManager
import android.telephony.TelephonyManager
import android.util.Log
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel

/**
 * SMS modem via platform SmsManager. Presence = SIM in READY state.
 * sendTestSms targets the device's own line number only (when known) and
 * never fires without a ready SIM — no surprise charges, no blind sends.
 */
internal class SmsHandler : DriverHandler {
  override fun handle(context: Context?, activity: Activity?, call: MethodCall, result: MethodChannel.Result) {
    if (context == null) return result.error("NO_CONTEXT", "no context", null)
    when (call.method) {
      "probeDetails" -> {
        val ready = testReady(context)
        val tm = context.getSystemService(Context.TELEPHONY_SERVICE) as? TelephonyManager
        val state = tm?.simState ?: TelephonyManager.SIM_STATE_UNKNOWN
        result.success(
          mapOf(
            "present" to ready,
            "reason" to if (ready) "ready" else "sim-not-ready",
            "simState" to state,
            "hasLineNumber" to !lineNumber(context).isNullOrBlank(),
            "devices" to emptyList<Any>(),
          ),
        )
      }
      "isAvailable" -> result.success(testReady(context))
      "sendTestSms" -> result.success(sendTestSms(context))
      else -> result.notImplemented()
    }
  }

  private fun lineNumber(context: Context): String? {
    if (!hasPhonePermission(context)) return null
    return try {
      (context.getSystemService(Context.TELEPHONY_SERVICE) as? TelephonyManager)?.line1Number
    } catch (e: SecurityException) {
      Log.w("EelDrivers", "line1Number denied: ${e.message}")
      null
    }
  }

  /**
   * Test presence: SIM ready AND own number known AND permission held. A
   * modem without an addressable number (emulator quirk: READY with no SIM)
   * reports not-present so callers SKIP instead of FAIL.
   */
  private fun testReady(context: Context): Boolean {
    val tm = context.getSystemService(Context.TELEPHONY_SERVICE) as? TelephonyManager
      ?: return false
    if (tm.simState != TelephonyManager.SIM_STATE_READY) return false
    return !lineNumber(context).isNullOrBlank()
  }

  private fun sendTestSms(context: Context): Boolean {
    val self = lineNumber(context)
    if (self.isNullOrBlank()) return false
    return try {
      @Suppress("DEPRECATION")
      SmsManager.getDefault().sendTextMessage(self, null, "EEL factory SMS test", null, null)
      true
    } catch (e: Exception) {
      Log.w("EelDrivers", "sendTestSms failed: ${e.message}")
      false
    }
  }

  private fun hasPhonePermission(context: Context): Boolean {
    val perms = listOf(
      Manifest.permission.READ_SMS,
      Manifest.permission.READ_PHONE_NUMBERS,
      Manifest.permission.READ_PHONE_STATE,
    )
    return perms.any {
      // Framework API (minSdk 24) — no androidx dependency needed.
      context.checkSelfPermission(it) == PackageManager.PERMISSION_GRANTED
    }
  }
}
