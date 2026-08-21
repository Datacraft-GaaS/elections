// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'assessment_model.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

AssessmentAttempt _$AssessmentAttemptFromJson(Map<String, dynamic> json) =>
    AssessmentAttempt(
      id: json['id'] as String,
      workerId: json['workerId'] as String,
      moduleId: json['moduleId'] as String,
      answers: (json['answers'] as Map<String, dynamic>).map(
        (k, e) => MapEntry(k, e as String),
      ),
      score: (json['score'] as num?)?.toDouble(),
      passed: json['passed'] as bool?,
      idempotencyKey: json['idempotencyKey'] as String,
      submittedAt: json['submittedAt'] == null
          ? null
          : DateTime.parse(json['submittedAt'] as String),
    );

Map<String, dynamic> _$AssessmentAttemptToJson(AssessmentAttempt instance) =>
    <String, dynamic>{
      'id': instance.id,
      'workerId': instance.workerId,
      'moduleId': instance.moduleId,
      'answers': instance.answers,
      'score': instance.score,
      'passed': instance.passed,
      'idempotencyKey': instance.idempotencyKey,
      'submittedAt': instance.submittedAt?.toIso8601String(),
    };
