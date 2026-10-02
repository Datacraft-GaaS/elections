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

import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:flutter/foundation.dart';

/// RFC 9457 problem+json (plan §2.3). Parsed into localized copy by apps.
@immutable
class ApiProblem {
  const ApiProblem({
    required this.type,
    required this.title,
    required this.status,
    this.detail,
    this.instance,
    this.extra = const {},
  });

  final String type;
  final String title;
  final int status;
  final String? detail;
  final String? instance;
  final Map<String, Object?> extra;

  factory ApiProblem.fromJson(Map<String, dynamic> json) => ApiProblem(
        type: json['type'] as String? ?? 'about:blank',
        title: json['title'] as String? ?? 'Error',
        status: json['status'] as int? ?? 500,
        detail: json['detail'] as String?,
        instance: json['instance'] as String?,
        extra: Map<String, Object?>.from(json)..removeWhere(
          (k, _) => ['type', 'title', 'status', 'detail', 'instance'].contains(k),
        ),
      );

  Map<String, dynamic> toJson() => {
        'type': type,
        'title': title,
        'status': status,
        if (detail != null) 'detail': detail,
        if (instance != null) 'instance': instance,
        ...extra,
      };

  /// Stable error id for support screens (plan §5.1).
  String get supportId => '$status:${Uri.parse(type).pathSegments.lastOrNull ?? type}';

  @override
  bool operator ==(Object other) =>
      other is ApiProblem &&
      other.type == type &&
      other.title == title &&
      other.status == status;

  @override
  int get hashCode => Object.hash(type, title, status);
}

/// Thrown by the API client when a problem+json body is received.
class ApiProblemException implements Exception {
  ApiProblemException(this.problem);
  final ApiProblem problem;

  @override
  String toString() => 'ApiProblem(${problem.status}, ${problem.type})';
}

/// Parses a problem+json payload from an HTTP response body.
ApiProblem parseProblem(Object? body, {int fallbackStatus = 500}) {
  if (body is Map<String, dynamic>) {
    return ApiProblem.fromJson(body);
  }
  return ApiProblem(
    type: 'about:blank',
    title: 'Unexpected response',
    status: fallbackStatus,
  );
}