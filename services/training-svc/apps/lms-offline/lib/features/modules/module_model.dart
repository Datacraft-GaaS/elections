import 'package:json_annotation/json_annotation.dart';

part 'module_model.g.dart';

enum ModuleType {
  @JsonValue('core')
  core,
  @JsonValue('refresher')
  refresher,
  @JsonValue('briefing')
  briefing,
}

@JsonSerializable()
class CourseModule {
  final String id;
  final String title;
  final String targetAudience;
  final String language;
  final String contentMarkdown;
  final List<String> assetIds;
  final int passMark;
  final ModuleType moduleType;

  const CourseModule({
    required this.id,
    required this.title,
    required this.targetAudience,
    required this.language,
    required this.contentMarkdown,
    required this.assetIds,
    this.passMark = 80,
    required this.moduleType,
  });

  factory CourseModule.fromJson(Map<String, dynamic> json) =>
      _$CourseModuleFromJson(json);

  Map<String, dynamic> toJson() => _$CourseModuleToJson(this);
}
