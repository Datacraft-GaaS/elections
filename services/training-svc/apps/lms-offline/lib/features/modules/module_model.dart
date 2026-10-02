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
