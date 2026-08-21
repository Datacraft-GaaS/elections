import 'package:json_annotation/json_annotation.dart';

part 'assessment_model.g.dart';

@JsonSerializable()
class AssessmentAttempt {
  final String id;
  final String workerId;
  final String moduleId;
  final Map<String, String> answers;
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
