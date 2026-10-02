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
import 'package:eel_kit_sdk/eel_kit_sdk.dart';

void main() => runApp(const EelExampleApp());

class EelExampleApp extends StatelessWidget {
  const EelExampleApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'eel Kit SDK Example',
      home: Scaffold(
        appBar: AppBar(title: const Text('eel Kit SDK')),
        body: Center(
          child: EelMinHitTarget(
            semanticsLabel: 'Start session',
            onTap: () async {
              // spiffe://eel.local is the issuer for attestation in production.
              final client = EelAttestationClientStub();
              final verdict = await client.attest();
              debugPrint(
                  'Attested: ${verdict.tier} device=${verdict.deviceId}');
            },
            child: const Text('Attest Device'),
          ),
        ),
      ),
    );
  }
}
