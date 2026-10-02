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

import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import '../assessment/assessment_model.dart';
import '../assessment/assessment_repository.dart';
import 'offline_queue.dart';

class SyncService {
  final AssessmentRepository _assessmentRepo;
  bool _syncing = false;

  SyncService({required AssessmentRepository assessmentRepo})
      : _assessmentRepo = assessmentRepo {
    _listenConnectivity();
  }

  void _listenConnectivity() {
    Connectivity().onConnectivityChanged.listen((results) {
      final hasNetwork = results.any((r) => r != ConnectivityResult.none);
      if (hasNetwork && !_syncing) {
        syncPending();
      }
    });
  }

  Future<SyncResult> syncPending() async {
    _syncing = true;
    int synced = 0;
    int failed = 0;

    try {
      final pending = await _assessmentRepo.getPendingAttempts();
      for (final entry in pending) {
        final attempt = _entryToAttempt(entry);
        try {
          await _assessmentRepo.submitAttempt(attempt);
          synced++;
        } on DioException {
          failed++;
        }
      }
    } finally {
      _syncing = false;
    }

    return SyncResult(synced: synced, failed: failed);
  }

  AssessmentAttempt _entryToAttempt(OfflineQueueEntry entry) {
    return AssessmentAttempt(
      id: entry.id,
      workerId: entry.workerId,
      moduleId: entry.moduleId,
      answers: entry.answers,
      idempotencyKey: entry.idempotencyKey,
    );
  }
}

class SyncResult {
  final int synced;
  final int failed;

  const SyncResult({required this.synced, required this.failed});
}

class SyncStatusScreen extends HookConsumerWidget {
  const SyncStatusScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: const Text('Sync Status')),
      body: const Center(
        child: Text('Sync will run automatically on reconnect.'),
      ),
    );
  }
}
