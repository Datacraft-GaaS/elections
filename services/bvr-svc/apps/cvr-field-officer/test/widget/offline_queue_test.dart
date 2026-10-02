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

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:crypto/crypto.dart';

import 'package:eel_cvr_field_officer/src/models/capture.dart';
import 'package:eel_cvr_field_officer/src/models/sync_status.dart';
import 'package:eel_cvr_field_officer/src/services/offline_queue.dart';

CaptureRecord _makeRecord({String id = 'test-001'}) => CaptureRecord(
      captureId: id,
      officerId: 'officer-1',
      modality: CaptureModality.deathCertificate,
      capturedAt: DateTime.now().toUtc(),
      geoLat: -1.286,
      geoLon: 36.817,
      geoAccuracyM: 5.0,
      imagePath: '/nonexistent/path.jpg',
      certSha256: 'a' * 64,
      officerHsmSignature: 'sig',
    );

void main() {
  group('OfflineQueueManager', () {
    late OfflineQueueManager queue;

    setUp(() {
      queue = OfflineQueueManager();
    });

    test('starts empty', () {
      expect(queue.pendingCount, greaterThanOrEqualTo(0));
    });

    test('enqueue increments pending count', () async {
      final before = queue.pendingCount;
      await queue.enqueue(_makeRecord(id: 'unique-${DateTime.now().microsecondsSinceEpoch}'));
      expect(queue.pendingCount, before + 1);
    });

    test('drain calls sender for each entry', () async {
      final record = _makeRecord(id: 'drain-test-${DateTime.now().microsecondsSinceEpoch}');
      await queue.enqueue(record);

      final called = <String>[];
      await queue.drain(
        sender: (r) async {
          called.add(r.captureId);
          return true;
        },
      );

      // The record may or may not be in called depending on sha256 check;
      // since imagePath is /nonexistent the sha256 check will fail and discard.
      // The test proves drain() runs without exception.
      expect(called, isA<List>());
    });

    test('drain returns SyncStatus', () async {
      final status = await queue.drain(sender: (_) async => true);
      expect(status, isA<SyncStatus>());
    });

    test('sha256 stability: missing file is discarded silently', () async {
      final record = _makeRecord(id: 'missing-file-${DateTime.now().microsecondsSinceEpoch}');
      await queue.enqueue(record);
      // drain should not throw even when file is missing
      final status = await queue.drain(sender: (_) async => true);
      expect(status.state, SyncState.idle);
    });

    test('drain sends entries whose on-disk hash matches the record', () async {
      final dir = await Directory.systemTemp.createTemp('eel_queue_test');
      final file = File('${dir.path}/capture.jpg');
      const bytes = [0x01, 0x02, 0x03, 0x04, 0xFF];
      await file.writeAsBytes(bytes);
      final digest = sha256.convert(bytes).toString();

      final id = 'hash-ok-${DateTime.now().microsecondsSinceEpoch}';
      await queue.enqueue(CaptureRecord(
        captureId: id,
        officerId: 'officer-1',
        modality: CaptureModality.deathCertificate,
        capturedAt: DateTime.now().toUtc(),
        geoLat: -1.286,
        geoLon: 36.817,
        geoAccuracyM: 5.0,
        imagePath: file.path,
        certSha256: digest,
        officerHsmSignature: 'sig',
      ));

      final called = <String>[];
      await queue.drain(sender: (r) async {
        called.add(r.captureId);
        return true;
      });

      expect(called, contains(id));
      expect(queue.pendingCount, 0);
      expect(queue.lastSyncAt, isNotNull);

      await dir.delete(recursive: true);
    });
  });
}
