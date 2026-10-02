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

/// Base URL for workforce-svc, overridable via --dart-define.
const kWorkforceBaseUrl = String.fromEnvironment(
  'EEL_WORKFORCE_URL',
  defaultValue: 'http://localhost:8100',
);

enum TaskStatus { pending, acked, done, blocked }

/// Worker notification kinds as emitted by workforce-svc (§4/§8).
enum NotificationKind { posting, payment, appointment, training, message }

NotificationKind notificationKindFromString(String value) {
  return NotificationKind.values.firstWhere(
    (k) => k.name == value,
    orElse: () => NotificationKind.message,
  );
}

/// A worker notification recorded by workforce-svc. `channel`/`deliveryRef`
/// carry hashes only (DPA 2019).
class WorkforceNotification {
  final String notificationId;
  final String workerId;
  final NotificationKind kind;
  final String? channel;
  final String subject;
  final String body;
  final String? deliveryRef;
  final DateTime createdAt;

  const WorkforceNotification({
    required this.notificationId,
    required this.workerId,
    required this.kind,
    this.channel,
    required this.subject,
    required this.body,
    this.deliveryRef,
    required this.createdAt,
  });

  factory WorkforceNotification.fromJson(Map<String, dynamic> j) =>
      WorkforceNotification(
        notificationId: j['notification_id'] as String,
        workerId: j['worker_id'] as String,
        kind: notificationKindFromString(j['kind'] as String? ?? 'message'),
        channel: j['channel'] as String?,
        subject: j['subject'] as String,
        body: j['body'] as String,
        deliveryRef: j['delivery_ref'] as String?,
        createdAt: DateTime.parse(j['created_at'] as String),
      );
}

class WorkforcePosting {
  final String postingId;
  final String workerId;
  final String stationCode;
  final String roleCode;
  final DateTime effectiveFrom;
  final DateTime? effectiveTo;

  const WorkforcePosting({
    required this.postingId,
    required this.workerId,
    required this.stationCode,
    required this.roleCode,
    required this.effectiveFrom,
    this.effectiveTo,
  });

  factory WorkforcePosting.fromJson(Map<String, dynamic> j) =>
      WorkforcePosting(
        postingId: j['posting_id'] as String,
        workerId: j['worker_id'] as String,
        stationCode: j['station_code'] as String,
        roleCode: j['role_code'] as String,
        effectiveFrom: DateTime.parse(j['effective_from'] as String),
        effectiveTo: j['effective_to'] == null
            ? null
            : DateTime.parse(j['effective_to'] as String),
      );
}

class WorkforceTask {
  final String assignmentId;
  final String postingId;
  final String title;
  final bool critical;
  final DateTime dueFrom;
  final DateTime dueTo;
  final TaskStatus status;
  final String? evidenceHash;

  const WorkforceTask({
    required this.assignmentId,
    required this.postingId,
    required this.title,
    required this.critical,
    required this.dueFrom,
    required this.dueTo,
    required this.status,
    this.evidenceHash,
  });

  factory WorkforceTask.fromJson(Map<String, dynamic> j) => WorkforceTask(
        assignmentId: j['assignment_id'] as String,
        postingId: j['posting_id'] as String,
        title: (j['title'] as String?) ?? j['template_id'] as String,
        critical: (j['critical'] as bool?) ?? false,
        dueFrom: DateTime.parse(j['due_from'] as String),
        dueTo: DateTime.parse(j['due_to'] as String),
        status: TaskStatus.values
            .firstWhere((s) => s.name == (j['status'] as String)),
        evidenceHash: j['evidence_hash'] as String?,
      );

  WorkforceTask copyWith({
    TaskStatus? status,
    String? evidenceHash,
  }) =>
      WorkforceTask(
        assignmentId: assignmentId,
        postingId: postingId,
        title: title,
        critical: critical,
        dueFrom: dueFrom,
        dueTo: dueTo,
        status: status ?? this.status,
        evidenceHash: evidenceHash ?? this.evidenceHash,
      );
}

/// Typed client over workforce-svc (:8100), design §5-§6.
class WorkforceApi {
  WorkforceApi({Dio? dio, String? baseUrl})
      : _dio = dio ??
            Dio(BaseOptions(
              baseUrl: baseUrl ?? kWorkforceBaseUrl,
              connectTimeout: const Duration(seconds: 10),
              receiveTimeout: const Duration(seconds: 30),
            ));

  final Dio _dio;

  Future<List<WorkforcePosting>> listActivePostings(String workerId) async {
    final res = await _dio.get<Map<String, dynamic>>(
      '/v1/workforce/postings',
      queryParameters: {'worker_id': workerId, 'active': 'true'},
    );
    final items = (res.data?['postings'] as List?) ?? const [];
    return items
        .map((e) => WorkforcePosting.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<List<WorkforceTask>> listTasks(String postingId) async {
    final res = await _dio.get<Map<String, dynamic>>(
      '/v1/workforce/tasks',
      queryParameters: {'posting_id': postingId},
    );
    final items = (res.data?['assignments'] as List?) ?? const [];
    return items
        .map((e) => WorkforceTask.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<void> ackTask(String id) async {
    await _dio.post<void>('/v1/workforce/tasks/$id/ack');
  }

  Future<void> doneTask(String id, String evidenceHash) async {
    await _dio.post<void>(
      '/v1/workforce/tasks/$id/done',
      data: {'evidence_hash': evidenceHash},
    );
  }

  Future<List<WorkforceNotification>> listNotifications(String workerId) async {
    final res = await _dio.get<Map<String, dynamic>>(
      '/v1/workforce/notifications',
      queryParameters: {'worker_id': workerId},
    );
    final items = (res.data?['notifications'] as List?) ?? const [];
    return items
        .map((e) => WorkforceNotification.fromJson(e as Map<String, dynamic>))
        .toList();
  }
}
