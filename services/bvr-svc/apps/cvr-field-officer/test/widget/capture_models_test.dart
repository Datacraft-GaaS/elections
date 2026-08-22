import 'package:flutter_test/flutter_test.dart';

import 'package:eel_cvr_field_officer/src/models/capture.dart';
import 'package:eel_cvr_field_officer/src/models/sync_status.dart';

final _epoch = DateTime.utc(1970);

void main() {
  group('CaptureRecord', () {
    test('copyWith preserves unchanged fields', () {
      final original = CaptureRecord(
        captureId: 'id-1',
        officerId: 'off-1',
        modality: CaptureModality.deathCertificate,
        capturedAt: _epoch,
        geoLat: -1.28,
        geoLon: 36.81,
        geoAccuracyM: 5.0,
        imagePath: '/path.jpg',
        certSha256: 'aabbcc',
        officerHsmSignature: 'sig',
      );

      final updated = original.copyWith(verdict: ReviewVerdict.accepted);

      expect(updated.captureId, 'id-1');
      expect(updated.verdict, ReviewVerdict.accepted);
      expect(updated.modality, CaptureModality.deathCertificate);
    });

    test('default verdict is pending', () {
      final r = CaptureRecord(
        captureId: 'id-2',
        officerId: 'off-1',
        modality: CaptureModality.idCorrection,
        capturedAt: _epoch,
        geoLat: 0,
        geoLon: 0,
        geoAccuracyM: 0,
        imagePath: '/x.jpg',
        certSha256: 'cc',
        officerHsmSignature: 's',
      );
      expect(r.verdict, ReviewVerdict.pending);
    });
  });

  group('ExtractedFields.fromJson', () {
    test('parses death cert fields', () {
      final f = ExtractedFields.fromJson({
        'deceased_name': 'John Doe',
        'national_id': '12345678',
        'date_of_death': '2026-08-01',
        'place_of_death': 'Nairobi',
        'cert_serial': 'CERT-ABCD',
      });
      expect(f.deceasedName, 'John Doe');
      expect(f.nationalId, '12345678');
      expect(f.certSerial, 'CERT-ABCD');
    });

    test('tolerates null values', () {
      final f = ExtractedFields.fromJson({});
      expect(f.deceasedName, isNull);
      expect(f.nationalId, isNull);
    });
  });

  group('SyncStatus', () {
    test('SyncStatus.idle has zero pending count', () {
      const s = SyncStatus.idle();
      expect(s.pendingCount, 0);
      expect(s.state, SyncState.idle);
    });

    test('copyWith updates fields', () {
      const s = SyncStatus.idle();
      final updated = s.copyWith(pendingCount: 3, state: SyncState.syncing);
      expect(updated.pendingCount, 3);
      expect(updated.state, SyncState.syncing);
    });
  });
}
