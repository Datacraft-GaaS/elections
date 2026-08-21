enum ScanStatus {
  active,
  revoked,
  suspended,
}

class ScanResult {
  final String serial;
  final ScanStatus status;
  final List<String> zones;
  final DateTime checkedAt;
  final bool isFromCacheOnly;

  const ScanResult({
    required this.serial,
    required this.status,
    required this.zones,
    required this.checkedAt,
    required this.isFromCacheOnly,
  });
}
