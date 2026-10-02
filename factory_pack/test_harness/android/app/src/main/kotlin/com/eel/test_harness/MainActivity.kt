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

package com.eel.test_harness

import android.Manifest
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothDevice
import android.content.Context
import android.content.pm.PackageManager
import android.hardware.usb.UsbDevice
import android.hardware.usb.UsbManager
import android.os.Build
import android.os.Bundle
import android.telephony.TelephonyManager
import android.util.Log
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.net.InetAddress
import java.net.UnknownHostException

class MainActivity : FlutterActivity() {
    private val CHANNEL = "eel/device/info"
    private val REQUEST_CODE = 1001

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL).setMethodCallHandler { call, result ->
            when (call.method) {
                "getInfo" -> {
                    // Request permissions if needed
                    requestDevicePermissions()
                    val info = collectDeviceInfo()
                    result.success(info)
                }

                else -> {
                    result.notImplemented()
                }
            }
        }
    }

    private fun requestDevicePermissions() {
        val permissions = mutableListOf<String>()
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.BLUETOOTH_SCAN) != PackageManager.PERMISSION_GRANTED) {
            permissions.add(Manifest.permission.BLUETOOTH_SCAN)
        }
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) != PackageManager.PERMISSION_GRANTED) {
            permissions.add(Manifest.permission.ACCESS_FINE_LOCATION)
        }
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.READ_PHONE_STATE) != PackageManager.PERMISSION_GRANTED) {
            permissions.add(Manifest.permission.READ_PHONE_STATE)
        }
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.READ_PHONE_NUMBERS) != PackageManager.PERMISSION_GRANTED) {
            permissions.add(Manifest.permission.READ_PHONE_NUMBERS)
        }
        if (permissions.isNotEmpty()) {
            ActivityCompat.requestPermissions(this, permissions.toTypedArray(), REQUEST_CODE)
        }
    }

    @Suppress("MissingPermission")
    private fun collectDeviceInfo(): Map<String, Any?> {
        val info = mutableMapOf<String, Any?>()

        // Basic platform info
        info["os"] = "android"
        info["osVersion"] = android.os.Build.VERSION.RELEASE
        info["hostname"] = getLocalHostName()
        info["model"] = android.os.Build.MODEL
        info["manufacturer"] = android.os.Build.MANUFACTURER

        // USB devices
        val usbList = mutableListOf<Map<String, Any?>>()
        try {
            val usbManager = getSystemService(Context.USB_SERVICE) as UsbManager
            val deviceList = usbManager.deviceList
            for (device in deviceList.values) {
                val devMap = mutableMapOf<String, Any?>()
                devMap["deviceId"] = device.deviceId
                devMap["vendorId"] = device.vendorId
                devMap["productId"] = device.productId
                devMap["deviceName"] = device.deviceName
                devMap["deviceClass"] = device.deviceClass
                devMap["deviceSubclass"] = device.deviceSubclass
                devMap["deviceProtocol"] = device.deviceProtocol
                usbList.add(devMap)
            }
        } catch (e: Exception) {
            Log.w("MainActivity", "USB enumeration failed: ${e.message}")
        }
        info["usbDevices"] = usbList

        // Bluetooth
        val bluetoothMap = mutableMapOf<String, Any?>()
        try {
            val bluetoothAdapter: BluetoothAdapter? = BluetoothAdapter.getDefaultAdapter()
            if (bluetoothAdapter != null) {
                bluetoothMap["name"] = bluetoothAdapter.name
                bluetoothMap["address"] = bluetoothAdapter.address
                val pairedDevices = bluetoothAdapter.bondedDevices
                val pairedList = mutableListOf<Map<String, String?>>()
                for (device in pairedDevices) {
                    val pairMap = mutableMapOf<String, String?>()
                    pairMap["name"] = device.name
                    pairMap["address"] = device.address
                    pairedList.add(pairMap)
                }
                bluetoothMap["pairedDevices"] = pairedList
            } else {
                bluetoothMap["error"] = "Bluetooth adapter not available"
            }
        } catch (e: Exception) {
            Log.w("MainActivity", "Bluetooth enumeration failed: ${e.message}")
            bluetoothMap["error"] = e.message.toString()
        }
        info["bluetooth"] = bluetoothMap

        // Telephony (IMEI, SIM info)
        val telephonyMap = mutableMapOf<String, Any?>()
        try {
            val teleMgr = getSystemService(Context.TELEPHONY_SERVICE) as TelephonyManager
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                // Android 10+; note: IMEI access may be restricted
                telephonyMap["imei"] = teleMgr.imei
                telephonyMap["deviceId"] = teleMgr.deviceId
                telephonyMap["deviceSoftwareVersion"] = teleMgr.deviceSoftwareVersion
                telephonyMap["simCountryIso"] = teleMgr.simCountryIso
                telephonyMap["simOperator"] = teleMgr.simOperator
                telephonyMap["simOperatorName"] = teleMgr.simOperatorName
                telephonyMap["simSerialNumber"] = teleMgr.simSerialNumber
                telephonyMap["simState"] = teleMgr.simState
                telephonyMap["networkOperator"] = teleMgr.networkOperator
                telephonyMap["networkOperatorName"] = teleMgr.networkOperatorName
            } else {
                // Pre-Android 10
                telephonyMap["imei"] = teleMgr.deviceId // For GSM, deviceId is IMEI
                telephonyMap["simSerialNumber"] = teleMgr.simSerialNumber
                telephonyMap["networkOperator"] = teleMgr.networkOperator
                telephonyMap["networkOperatorName"] = teleMgr.networkOperatorName
            }
        } catch (e: SecurityException) {
            Log.w("MainActivity", "Telephony info requires READ_PHONE_STATE permission: ${e.message}")
            telephonyMap["error"] = "Missing READ_PHONE_STATE permission"
        } catch (e: Exception) {
            Log.w("MainActivity", "Telephony enumeration failed: ${e.message}")
            telephonyMap["error"] = e.message.toString()
        }
        info["telephony"] = telephonyMap

        return info
    }

    private fun getLocalHostName(): String? =
        try {
            InetAddress.getLocalHost().hostName
        } catch (e: UnknownHostException) {
            null
        }
}
