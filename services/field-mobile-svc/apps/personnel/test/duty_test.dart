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

import 'package:dio/dio.dart';
import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:eel_personnel/src/workforce/duty_controller.dart';
import 'package:eel_personnel/src/workforce/workforce_api.dart';
import 'package:flutter_test/flutter_test.dart';

class FakeWorkforceApi implements WorkforceApi {
  List<WorkforcePosting> postings = const [];
  Map<String, List<WorkforceTask>> tasksByPosting = const {};
  List<WorkforceNotification> notifications = const [];
  Object? throwOnList;
  Object? throwOnMutation;

  final acked = <String>[];
  final dones = <(String, String?)>[];
  int mutationCalls = 0;

  @override
  Future<List<WorkforcePosting>> listActivePostings(String workerId) async {
    if (throwOnList != null) throw throwOnList!;
    return postings;
  }

  @override
  Future<List<WorkforceTask>> listTasks(String postingId) async {
    if (throwOnList != null) throw throwOnList!;
    return tasksByPosting[postingId] ?? const [];
  }

  @override
  Future<List<WorkforceNotification>> listNotifications(String workerId) async {
    if (throwOnList != null) throw throwOnList!;
    return notifications;
  }

  @override
  Future<void> ackTask(String id) async {
    mutationCalls++;
    if (throwOnMutation != null) throw throwOnMutation!;
    acked.add(id);
  }

  @override
  Future<void> doneTask(String id, String evidenceHash) async {
    mutationCalls++;
    if (throwOnMutation != null) throw throwOnMutation!;
    dones.add((id, evidenceHash));
  }
}

WorkforceTask task({
  String id = 't1',
  String postingId = 'p1',
  bool critical = false,
  TaskStatus status = TaskStatus.pending,
}) =>
    WorkforceTask(
      assignmentId: id,
      postingId: postingId,
      title: 'Verify seals',
      critical: critical,
      dueFrom: DateTime(2026, 8, 1),
      dueTo: DateTime(2026, 8, 2),
      status: status,
    );

void main() {
  late FakeWorkforceApi api;
  late EelOfflineQueue queue;
  late DutyController controller;

  setUp(() async {
    api = FakeWorkforceApi();
    queue = EelOfflineQueue(storage: InMemoryOutboxStorage());
    await queue.init();
    controller = DutyController(api: api, queue: queue);
  });

  test('optimistic ack applies locally and enqueues outbox entry', () async {
    api.postings = [
      WorkforcePosting(
        postingId: 'p1',
        workerId: 'w1',
        stationCode: 'STN-001',
        roleCode: 'po',
        effectiveFrom: DateTime(2026, 8, 9),
      ),
    ];
    api.tasksByPosting = {'p1': [task()]};
    await controller.refresh();

    api.throwOnMutation = Exception('offline');
    await controller.ackTask(task());

    expect(controller.state.tasksByPosting['p1']!.single.status,
        TaskStatus.acked);
    expect(queue.pending.length, 1);
    final entry = queue.pending.single;
    expect(entry.payload['action'], 'task.ack');
    expect(entry.payload['taskId'], 't1');
  });

  test('done without evidence hash fails validation before any network call',
      () async {
    final critical = task(id: 'c1', critical: true);
    await expectLater(
      controller.doneTask(critical, null),
      throwsA(isA<DutyValidationException>()),
    );
    expect(api.mutationCalls, 0);
    expect(critical.status, TaskStatus.pending);

    await expectLater(
      controller.doneTask(critical, ''),
      throwsA(isA<DutyValidationException>()),
    );
    expect(api.mutationCalls, 0);
  });

  test('refresh keeps cached data when api throws (offline resilience)',
      () async {
    api.postings = [
      WorkforcePosting(
        postingId: 'p1',
        workerId: 'w1',
        stationCode: 'STN-002',
        roleCode: 'dpo',
        effectiveFrom: DateTime(2026, 8, 9),
      ),
    ];
    api.tasksByPosting = {'p1': [task()]};
    await controller.refresh();
    expect(controller.state.postings.length, 1);

    api.throwOnList = DioException(requestOptions: RequestOptions(path: '/'));
    await controller.refresh();

    expect(controller.state.postings.length, 1);
    expect(controller.state.tasksByPosting['p1']!.single.assignmentId, 't1');
    expect(controller.state.errorMessage, isNotNull);
  });

  test('sync drains queue calling upload per entry', () async {
    api.throwOnMutation = Exception('offline');
    api.tasksByPosting = {'p1': [task(), task(id: 't2', critical: true)]};
    await controller.refresh();

    await controller.ackTask(task());
    await controller.doneTask(task(id: 't2', critical: true), 'a' * 64);
    expect(queue.pending.length, 2);
    expect(api.mutationCalls, 2); // both write-through attempts failed

    api.throwOnMutation = null;
    await controller.sync();

    expect(api.acked, contains('t1'));
    expect(api.dones.map((d) => d.$1), contains('t2'));
    expect(api.dones.map((d) => d.$2), contains('a' * 64));
    expect(queue.pending, isEmpty);
  });
}
