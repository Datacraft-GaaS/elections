class CrlModel {
  final DateTime generatedAt;
  final List<String> revokedSerials;

  const CrlModel({
    required this.generatedAt,
    required this.revokedSerials,
  });

  factory CrlModel.fromJson(Map<String, dynamic> json) {
    return CrlModel(
      generatedAt: DateTime.parse(json['generatedAt'] as String),
      revokedSerials: (json['revokedSerials'] as List<dynamic>)
          .map((e) => e as String)
          .toList(),
    );
  }

  Map<String, dynamic> toJson() => {
        'generatedAt': generatedAt.toIso8601String(),
        'revokedSerials': revokedSerials,
      };

  bool isStale({Duration threshold = const Duration(minutes: 30)}) {
    return DateTime.now().difference(generatedAt) > threshold;
  }
}
