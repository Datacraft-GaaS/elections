import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('KitTemplate round-trips the snake_case contract', () {
    const t = KitTemplate(modality: 'iris', dataB64: 'aGVsbG8=', quality: 0.88);
    expect(t.toJson(), {
      'modality': 'iris',
      'data_b64': 'aGVsbG8=',
      'quality': 0.88,
    });
    final back = KitTemplate.fromJson(t.toJson());
    expect(back.modality, 'iris');
    expect(back.dataB64, 'aGVsbG8=');
    expect(back.quality, 0.88);
  });

  test('KitEnrolResult parses the ABIS response', () {
    final r = KitEnrolResult.fromJson({
      'job_id': 'j1',
      'status': 'queued',
      'subject_id': 's1',
    });
    expect(r.jobId, 'j1');
    expect(r.status, 'queued');
    expect(r.subjectId, 's1');
  });

  test('KitVerifyResult parses the ABIS response', () {
    final r = KitVerifyResult.fromJson({
      'subject_id': 's1',
      'verified': true,
      'score': 0.97,
      'decision': 'match',
    });
    expect(r.verified, isTrue);
    expect(r.score, 0.97);
    expect(r.decision, 'match');
  });

  test('GalleryConfig parses with and without last_sync_at', () {
    final withSync = GalleryConfig.fromJson({
      'sync_url': 'https://abis.internal/v1/kit/gallery',
      'last_sync_at': '2026-08-27T10:00:00Z',
      'entries': 42,
    });
    expect(withSync.syncUrl, 'https://abis.internal/v1/kit/gallery');
    expect(withSync.entries, 42);
    expect(withSync.lastSyncAt, DateTime.utc(2026, 8, 27, 10));

    final withoutSync = GalleryConfig.fromJson({
      'sync_url': 'https://abis.internal/v1/kit/gallery',
      'last_sync_at': null,
      'entries': 0,
    });
    expect(withoutSync.lastSyncAt, isNull);
    expect(withoutSync.entries, 0);
  });
}
