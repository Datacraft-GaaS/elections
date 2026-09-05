/// Party-Agent QR Verify (M30 FR-006) — Beautiful, ergonomic, accessible.
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class QRVerifyScreen extends ConsumerWidget {
  const QRVerifyScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return EelScaffold(
      title: 'QR Verify',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('QR Code Verification', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
          const SizedBox(height: 24),
          EelResultCard(
            leading: Icon(Icons.qr_code, color: EelColors.of(context).primary, size: 32),
            title: Text('QR Scanner', style: EelTextStyle.titleMedium),
            subtitle: Text('Ready to scan QR codes', style: EelTextStyle.bodyMedium),
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Start Scanner',
            icon: Icons.camera_alt,
            fullWidth: true,
            onPressed: () {},
          ),
        ],
      ),
    );
  }
}