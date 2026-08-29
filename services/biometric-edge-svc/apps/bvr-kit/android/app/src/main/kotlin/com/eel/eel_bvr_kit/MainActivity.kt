package com.eel.eel_bvr_kit

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine

class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        IrisPlugin(this, flutterEngine)
        FingerprintPlugin(this, flutterEngine)
    }
}
