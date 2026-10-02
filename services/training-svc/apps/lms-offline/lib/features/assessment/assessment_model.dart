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

import 'package:json_annotation/json_annotation.dart';

part 'assessment_model.g.dart';

@JsonSerializable()
class AssessmentAttempt {
  final String id;
  final String workerId;
  final String moduleId;
  final Map<String, dynamic> answers;
  final double? score;
  final bool? passed;
  final String idempotencyKey;
  final DateTime? submittedAt;

  const AssessmentAttempt({
    required this.id,
    required this.workerId,
    required this.moduleId,
    required this.answers,
    this.score,
    this.passed,
    required this.idempotencyKey,
    this.submittedAt,
  });

  factory AssessmentAttempt.fromJson(Map<String, dynamic> json) =>
      _$AssessmentAttemptFromJson(json);

  Map<String, dynamic> toJson() => _$AssessmentAttemptToJson(this);

  AssessmentAttempt copyWith({
    double? score,
    bool? passed,
    DateTime? submittedAt,
  }) {
    return AssessmentAttempt(
      id: id,
      workerId: workerId,
      moduleId: moduleId,
      answers: answers,
      score: score ?? this.score,
      passed: passed ?? this.passed,
      idempotencyKey: idempotencyKey,
      submittedAt: submittedAt ?? this.submittedAt,
    );
  }
}
