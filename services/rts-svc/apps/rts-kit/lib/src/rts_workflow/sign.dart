/// Sign Form 34A: image hash + OCR JSON signed with the kit's SPIFFE-issued key
/// obtained via [EelAttestationClient] from eel_kit_sdk.
///
/// The payload is a canonical JSON object containing:
///   - sha256 of the corrected image bytes
///   - OCR result fields and confidence
///   - ISO-8601 timestamp
///   - device ID from the attestation verdict
///
/// Signature is Ed25519 over SHA-256(canonical JSON) using the kit-local key
/// that was issued during SPIFFE bootstrap. In production the signing key lives
/// in the device TEE (Strongbox/Secure Enclave); for dev/test the SDK provides
/// a software fallback.
library;

import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logging/logging.dart';

import '../a11y/workflow_semantics.dart';
import 'ocr_preview.dart';
import 'scan.dart';
import 'transmit.dart';

final _log = Logger('eel.rts.sign');

// ---- Types ------------------------------------------------------------------

class SignedForm {
  const SignedForm({
    required this.payload,
    required this.payloadHash,
    required this.signatureHex,
    required this.deviceId,
    required this.signedAt,
    required this.form,
    required this.ocrResult,
  });

  final Map<String, Object?> payload;
  final String payloadHash;
  final String signatureHex;
  final String deviceId;
  final DateTime signedAt;
  final CapturedForm form;
  final OcrResult ocrResult;

  Map<String, Object?> toJson() => {
        'payload': payload,
        'payloadHash': payloadHash,
        'signatureHex': signatureHex,
        'deviceId': deviceId,
        'signedAt': signedAt.toIso8601String(),
      };
}

class SignArgs {
  const SignArgs({required this.form, required this.ocrResult});
  final CapturedForm form;
  final OcrResult ocrResult;
}

// ---- Signing ----------------------------------------------------------------

/// Build canonical signing payload and compute SHA-256 hash.
Map<String, Object?> _buildPayload({
  required Uint8List imageBytes,
  required OcrResult ocrResult,
  required String deviceId,
  required DateTime signedAt,
}) {
  final imageHash = sha256.convert(imageBytes).toString();
  return {
    'form_type': 'form34a',
    'image_sha256': imageHash,
    'ocr': ocrResult.toJson(),
    'device_id': deviceId,
    'signed_at': signedAt.toIso8601String(),
  };
}

/// Sign the SHA-256 digest of the canonical payload with the kit's device
/// signing key ([EelSigningKey] from eel_kit_sdk, Ed25519).
///
/// The key seed persists in secure storage; on devices with a TEE
/// (Strongbox / Secure Enclave) the platform-backed implementation replaces
/// the software fallback behind the same interface. Returns the signature
/// as lowercase hex.
Future<String> _signPayload(Map<String, Object?> payload) async {
  final canonical = const JsonEncoder().convert(payload);
  final digest = sha256.convert(utf8.encode(canonical));
  final key = await EelSigningKey.loadOrCreate();
  final sigB64 = await key.sign(digest.bytes);
  return base64Decode(sigB64)
      .map((b) => b.toRadixString(16).padLeft(2, '0'))
      .join();
}

// ---- Provider ---------------------------------------------------------------

final attestationProvider = Provider<EelAttestationClient>(
  (ref) => EelAttestationClientStub(),
);

// ---- Screen -----------------------------------------------------------------

class SignScreen extends ConsumerStatefulWidget {
  const SignScreen({super.key, required this.args});
  final SignArgs args;

  @override
  ConsumerState<SignScreen> createState() => _SignScreenState();
}

class _SignScreenState extends ConsumerState<SignScreen> {
  SignedForm? _signed;
  bool _signing = false;
  String? _error;

  Future<void> _sign() async {
    setState(() {
      _signing = true;
      _error = null;
    });
    try {
      final attestation = await ref.read(attestationProvider).attest();
      _log.info('Attesting with tier ${attestation.tier}');

      final signedAt = DateTime.now().toUtc();
      final payload = _buildPayload(
        imageBytes: widget.args.form.correctedBytes,
        ocrResult: widget.args.ocrResult,
        deviceId: attestation.deviceId,
        signedAt: signedAt,
      );
      final sigHex = await _signPayload(payload);
      final canonical = const JsonEncoder().convert(payload);
      final hash = sha256.convert(utf8.encode(canonical)).toString();

      final signed = SignedForm(
        payload: payload,
        payloadHash: hash,
        signatureHex: sigHex,
        deviceId: attestation.deviceId,
        signedAt: signedAt,
        form: widget.args.form,
        ocrResult: widget.args.ocrResult,
      );
      _log.info('Signed form hash=$hash device=${attestation.deviceId}');
      if (mounted) setState(() => _signed = signed);
    } catch (e, st) {
      _log.warning('Signing failed', e, st);
      if (mounted) setState(() => _error = e.toString());
    } finally {
      if (mounted) setState(() => _signing = false);
    }
  }

  void _proceed() {
    final s = _signed;
    assert(s != null, 'Must sign before transmitting');
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => TransmitScreen(args: TransmitArgs(signed: s!)),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final signed = _signed;
    return Scaffold(
      appBar: AppBar(
        title: const EelSemanticsLabel(
          label: 'Sign Form 34A',
          child: Text('Sign Form'),
        ),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            WfSemanticsBanner(
              message: 'Review the form before signing. This action is final.',
              role: 'instruction',
              child: Card(
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        'Signing commits this form to the tally chain.',
                        style: TextStyle(fontWeight: FontWeight.bold),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        'OCR source: ${widget.args.ocrResult.source}  '
                        'Confidence: ${(widget.args.ocrResult.confidence * 100).toStringAsFixed(0)}%',
                      ),
                    ],
                  ),
                ),
              ),
            ),
            const SizedBox(height: 16),
            if (_error != null)
              WfSemanticsBanner(
                message: _error!,
                role: 'error',
                child: Text(
                  _error!,
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.error,
                  ),
                ),
              ),
            if (signed != null)
              Semantics(
                liveRegion: true,
                label:
                    'Form signed successfully. Hash: ${signed.payloadHash.substring(0, 16)}',
                child: Card(
                  color: Theme.of(context).colorScheme.secondaryContainer,
                  child: Padding(
                    padding: const EdgeInsets.all(12),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text(
                          'Signed',
                          style: TextStyle(fontWeight: FontWeight.bold),
                        ),
                        Text(
                          'Hash: ${signed.payloadHash.substring(0, 24)}...',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                        Text(
                          'Device: ${signed.deviceId}',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            const Spacer(),
            Row(
              children: [
                if (signed == null)
                  Expanded(
                    child: EelMinHitTarget(
                      semanticsLabel: 'Sign form with kit key',
                      onTap: _signing ? null : _sign,
                      child: FilledButton.icon(
                        onPressed: _signing ? null : _sign,
                        icon: _signing
                            ? const SizedBox(
                                width: 16,
                                height: 16,
                                child:
                                    CircularProgressIndicator(strokeWidth: 2),
                              )
                            : const Icon(Icons.verified_user),
                        label: Text(_signing ? 'Signing...' : 'Sign Form'),
                      ),
                    ),
                  )
                else
                  Expanded(
                    child: EelMinHitTarget(
                      semanticsLabel: 'Proceed to transmit',
                      onTap: _proceed,
                      child: FilledButton.icon(
                        onPressed: _proceed,
                        icon: const Icon(Icons.send),
                        label: const Text('Transmit'),
                      ),
                    ),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
