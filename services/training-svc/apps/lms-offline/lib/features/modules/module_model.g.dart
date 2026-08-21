// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'module_model.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

CourseModule _$CourseModuleFromJson(Map<String, dynamic> json) => CourseModule(
      id: json['id'] as String,
      title: json['title'] as String,
      targetAudience: json['targetAudience'] as String,
      language: json['language'] as String,
      contentMarkdown: json['contentMarkdown'] as String,
      assetIds: (json['assetIds'] as List<dynamic>).map((e) => e as String).toList(),
      passMark: (json['passMark'] as num?)?.toInt() ?? 80,
      moduleType: _$enumDecode(_$ModuleTypeEnumMap, json['moduleType']),
    );

Map<String, dynamic> _$CourseModuleToJson(CourseModule instance) =>
    <String, dynamic>{
      'id': instance.id,
      'title': instance.title,
      'targetAudience': instance.targetAudience,
      'language': instance.language,
      'contentMarkdown': instance.contentMarkdown,
      'assetIds': instance.assetIds,
      'passMark': instance.passMark,
      'moduleType': _$ModuleTypeEnumMap[instance.moduleType]!,
    };

const _$ModuleTypeEnumMap = {
  ModuleType.core: 'core',
  ModuleType.refresher: 'refresher',
  ModuleType.briefing: 'briefing',
};

T _$enumDecode<T extends Enum>(
  Map<T, dynamic> enumValues,
  dynamic source, {
  T? unknownValue,
}) {
  if (source == null) {
    throw ArgumentError(
      'A value for field is required but was null.',
    );
  }
  for (final entry in enumValues.entries) {
    if (entry.value == source) {
      return entry.key;
    }
  }
  if (unknownValue != null) return unknownValue;
  throw ArgumentError(
    '`$source` is not one of the supported values: ${enumValues.values.join(', ')}',
  );
}
