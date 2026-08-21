import 'package:flutter/material.dart';

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
                  'Legacy Android shim active. Attestation: software only.',
                  style: TextStyle(color: Colors.red),
                  textAlign: TextAlign.center,
                ),
              ),
            const SizedBox(height: 24),
            ElevatedButton(
              onPressed: () {},
              child: const Text('BVR Enrolment'),
            ),
            const SizedBox(height: 12),
            ElevatedButton(
              onPressed: () {},
              child: const Text('BVI Identification'),
            ),
          ],
        ),
      ),
    );
  }
}
