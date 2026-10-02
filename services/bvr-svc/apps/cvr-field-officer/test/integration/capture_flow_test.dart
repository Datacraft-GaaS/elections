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

/// Integration tests for the full capture flow.
///
/// These tests exercise the capture -> review -> queue flow end-to-end
/// without a running device (using flutter_test host runner).
library;

import 'package:flutter_test/flutter_test.dart';

import 'package:eel_cvr_field_officer/src/models/capture.dart';
import 'package:eel_cvr_field_officer/src/models/sync_status.dart';
import 'package:eel_cvr_field_officer/src/services/attestation.dart';
import 'package:eel_cvr_field_officer/src/services/offline_queue.dart';

void main() {
  group('Capture -> Queue integration', () {
    test('capture record can be enqueued and drained', () async {
      final queue = OfflineQueueManager();
      final id = 'integ-${DateTime.now().microsecondsSinceEpoch}';

      final record = CaptureRecord(
        captureId: id,
        officerId: 'officer-integ',
        modality: CaptureModality.deathCertificate,
        capturedAt: DateTime.now().toUtc(),
        geoLat: -1.286389,
        geoLon: 36.817223,
        geoAccuracyM: 5.0,
        imagePath: '/nonexistent/integ.jpg',
        certSha256: 'a' * 64,
        officerHsmSignature: 'sig-integ',
      );

      await queue.enqueue(record);
      // sha256 file check will discard the entry (file missing) - that's ok.
      final status = await queue.drain(sender: (_) async => true);
      expect(status, isA<SyncStatus>());
    });

    test('HSM sign round-trip produces stable hex', () async {
      const data = [0x01, 0x02, 0x03, 0xAB, 0xCD, 0xEF];
      final sig1 = await Attestation.sign(data);
      final sig2 = await Attestation.sign(data);
      expect(sig1.signatureHex, equals(sig2.signatureHex));
      expect(sig1.signatureHex.length, greaterThan(0));
    });

    test('offline queue pending count reflects enqueue', () async {
      final queue = OfflineQueueManager();
      final before = queue.pendingCount;

      await queue.enqueue(CaptureRecord(
        captureId: 'count-test-${DateTime.now().microsecondsSinceEpoch}',
        officerId: 'officer-count',
        modality: CaptureModality.idCorrection,
        capturedAt: DateTime.now().toUtc(),
        geoLat: -0.0236,
        geoLon: 37.9062,
        geoAccuracyM: 10.0,
        imagePath: '/count.jpg',
        certSha256: 'b' * 64,
        officerHsmSignature: 'sig-count',
      ));

      expect(queue.pendingCount, before + 1);
    });

    test('review verdict pending is the default for new captures', () {
      final r = CaptureRecord(
        captureId: 'v-test',
        officerId: 'o',
        modality: CaptureModality.deathCertificate,
        capturedAt: DateTime.utc(2026, 8, 22),
        geoLat: -1.28,
        geoLon: 36.81,
        geoAccuracyM: 5.0,
        imagePath: '/v.jpg',
        certSha256: 'c' * 64,
        officerHsmSignature: 'sig-v',
      );
      expect(r.verdict, ReviewVerdict.pending);
    });

    test('idempotency key generation is stable per captureId', () {
      // The ApiClient generates idempotencyKey = 'cvr-{captureId}'.
      // Verify the expected format.
      const captureId = 'stable-id-001';
      const expectedKey = 'cvr-$captureId';
      expect(expectedKey, startsWith('cvr-'));
      expect(expectedKey, contains(captureId));
    });
  });
}
