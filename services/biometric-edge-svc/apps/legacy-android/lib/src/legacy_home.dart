// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

/// Legacy Android shim (WP-3.8).
///
/// Deep-links into real kit apps (bvr-kit, bvi-kit) when installed.
/// Falls back to MDM managed-app store install guidance when not installed.
/// Per docs/mobile/legacy-android.md: shim is intentional — no parallel features.
class LegacyHomeScreen extends StatelessWidget {
  final bool shimEnabled;

  const LegacyHomeScreen({super.key, required this.shimEnabled});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('eel Legacy Kit'),
        backgroundColor: shimEnabled ? Colors.red.shade100 : null,
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            if (shimEnabled)
              Container(
                padding: const EdgeInsets.all(8),
                color: Colors.red.shade50,
                child: const Text(
                  'Legacy Android shim active. Attestation: software only. '
                  'Install full kit apps (BVR/BVI) for hardware features.',
                  style: TextStyle(color: Colors.red),
                  textAlign: TextAlign.center,
                ),
              ),
            const SizedBox(height: 24),
            ElevatedButton(
              onPressed: () => _openKit(context, 'eel://kit/bvr/enrol', 'bvr-kit'),
              child: const Text('BVR Enrolment'),
            ),
            const SizedBox(height: 12),
            ElevatedButton(
              onPressed: () => _openKit(context, 'eel://kit/bvi/identify', 'bvi-kit'),
              child: const Text('BVI Identification'),
            ),
          ],
        ),
      ),
    );
  }

  void _openKit(BuildContext context, String deepLink, String packageName) async {
    final uri = Uri.parse(deepLink);
    try {
      final launched = await launchUrl(uri, mode: LaunchMode.externalApplication);
      if (!launched) _showInstallGuidance(context, packageName);
    } catch (_) {
      _showInstallGuidance(context, packageName);
    }
  }

  void _showInstallGuidance(BuildContext context, String packageName) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Install Kit App'),
        content: Text(
          'Install $packageName from MDM managed-app store to use hardware features.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              Navigator.pop(ctx);
              launchUrl(
                Uri.parse('https://eel.local/mdm/apps/$packageName'),
                mode: LaunchMode.externalApplication,
              );
            },
            child: const Text('Open Store'),
          ),
        ],
      ),
    );
  }
}
