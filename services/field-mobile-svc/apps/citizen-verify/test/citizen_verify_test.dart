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

import 'package:citizen_verify/main.dart';
import 'package:citizen_verify/src/core/services/results_service.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

void main() {
  testWidgets('app renders without crash', (tester) async {
    await tester.pumpWidget(const ProviderScope(child: CitizenVerifyApp()));
    await tester.pump();
    expect(find.byType(MaterialApp), findsOneWidget);
  });

  group('ResultsData.fromJson', () {
    test('reads a signed result set', () {
      final r = ResultsData.fromJson(<String, dynamic>{
        'station_code': 'KE-NBI-001',
        'total_voters': 12450,
        'total_turnout': 87.0,
        'form_34a_url': '/form34a/KE-NBI-001',
        'snapshot_id': 'snap-1',
        'proof': <String, dynamic>{
          'is_valid': true,
          'root_hash': 'abc123',
          'leaf_hash': 'def456',
          'merkle_path': 'a/b/c',
          'tree_height': 12,
        },
      });

      expect(r.ok, isTrue);
      expect(r.stationCode, 'KE-NBI-001');
      expect(r.totalVoters, 12450);
      // A JSON number arrives as double; the model must not hand back 87.0.
      expect(r.totalTurnout, 87);
      expect(r.proof!.isValid, isTrue);
      expect(r.proof!.rootHash, 'abc123');
      expect(r.proof!.height, 12);
    });

    test('a result with no proof still parses', () {
      final r = ResultsData.fromJson(<String, dynamic>{
        'station_code': 'KE-NBI-002',
      });
      expect(r.ok, isTrue);
      expect(r.proof, isNull);
      expect(r.totalVoters, isNull);
    });

    test('a result with an unsigned proof is not treated as valid', () {
      final r = ResultsData.fromJson(<String, dynamic>{
        'proof': <String, dynamic>{'is_valid': false, 'error': 'bad signature'},
      });
      expect(r.proof!.isValid, isFalse);
      expect(r.proof!.error, 'bad signature');
    });

    test('ResultsData.error carries the reason and is not ok', () {
      final r = ResultsData.error('HTTP 503');
      expect(r.ok, isFalse);
      expect(r.error, 'HTTP 503');
    });
  });

  group('verifyProofClient', () {
    final service = ResultsService();

    test('accepts a valid proof with a root hash', () async {
      final ok = await service.verifyProofClient(
        proof: const MerkleProof(isValid: true, rootHash: 'abc123'),
      );
      expect(ok, isTrue);
    });

    test('rejects a proof with no root hash', () async {
      // isValid true but nothing to verify against is not a proof.
      final ok = await service.verifyProofClient(
        proof: const MerkleProof(isValid: true),
      );
      expect(ok, isFalse);
    });

    test('rejects an empty root hash', () async {
      final ok = await service.verifyProofClient(
        proof: const MerkleProof(isValid: true, rootHash: ''),
      );
      expect(ok, isFalse);
    });

    test('rejects an invalid proof', () async {
      final ok = await service.verifyProofClient(
        proof: const MerkleProof(isValid: false, rootHash: 'abc123'),
      );
      expect(ok, isFalse);
    });
  });
}
