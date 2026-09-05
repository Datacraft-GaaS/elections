import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart' as crypto show sha256;
import 'package:flutter/foundation.dart';

/// HLC string + record → SHA-256 chain hash (EC-MOB-01 WAL chain).
String chainHash({
  required String previousHash,
  required Map<String, Object?> payload,
  required String hlc,
}) {
  final bytes = utf8.encode('$previousHash|${jsonEncode(payload)}|$hlc');
  return crypto.sha256.convert(bytes).toString();
}

/// HLC generator interface; kit-sdk `eel_hlc` provides the real one.
abstract class HlcSource {
  String next();
}

/// Wall-clock HLC (app-tier fallback; kit-sdk HLC preferred).
class WallClockHlc implements HlcSource {
  @override
  String next() {
    final ms = DateTime.now().millisecondsSinceEpoch;
    return '$ms-0000-node';
  }
}

/// Builds the CloudEvents 1.0 envelope with the `eel_*` shape from
/// `packages/event-schemas/**/v1.schema.json` (plan §2.3).
Map<String, Object?> buildCloudEvent({
  required String type,
  required String source,
  required String subject,
  required String eventId,
  required String hlc,
  required Map<String, Object?> data,
  Map<String, String>? traceContext,
}) =>
    {
      'specversion': '1.0',
      'id': eventId,
      'type': type,
      'source': source,
      'subject': subject,
      'time': DateTime.now().toUtc().toIso8601String(),
      'eel_hlc': hlc,
      'data': data,
      if (traceContext != null) 'traceparent': traceContext['traceparent'],
    };

/// SHA-256 digest of a byte payload — evidence upload integrity footer.
String sha256Hex(Uint8List bytes) =>
    crypto.sha256.convert(bytes).toString();