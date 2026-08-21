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
