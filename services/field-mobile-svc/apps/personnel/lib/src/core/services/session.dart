/// Personnel session (M33) - staff/admin tier
library;
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:uuid/uuid.dart';

class PersonnelSessionState {
  final bool authenticated;
  final String? userId; final String? role; final String? staffCode;
  final DateTime expiresAt; final String sessionId;
  const PersonnelSessionState({
    this.authenticated = false, this.userId, this.role, this.staffCode,
    required this.expiresAt, required this.sessionId,
  });
  factory PersonnelSessionState.initial() => PersonnelSessionState(expiresAt: DateTime.now(), sessionId: const Uuid().v7());
  factory PersonnelSessionState.authenticated({required String userId, String? staffCode}) => PersonnelSessionState(
    authenticated: true, userId: userId, role: 'personnel', staffCode: staffCode,
    expiresAt: DateTime.now().add(const Duration(hours: 8)), sessionId: const Uuid().v7(),
  );
  PersonnelSessionState copyWith({bool? authenticated, String? userId, String? role, String? staffCode, DateTime? expiresAt, String? sessionId}) => PersonnelSessionState(
    authenticated: authenticated ?? this.authenticated,
    userId: userId ?? this.userId, role: role ?? this.role,
    staffCode: staffCode ?? this.staffCode,
    expiresAt: expiresAt ?? this.expiresAt,
    sessionId: sessionId ?? this.sessionId,
  );
  bool get isExpired => DateTime.now().isAfter(expiresAt);
}

class PersonnelSessionService {
  final FlutterSecureStorage _store;
  PersonnelSessionService({FlutterSecureStorage? s}) : _store = s ?? const FlutterSecureStorage();
  Future<PersonnelSessionState> load() async {
    try {
      final d = await _store.read(key: 'personnel_session');
      if (d == null) return PersonnelSessionState.initial();
      final data = jsonDecode(d) as Map<String, dynamic>;
      return PersonnelSessionState(
        authenticated: data['authenticated'] == true,
        userId: data['user_id'] as String?, role: data['role'] as String?,
        staffCode: data['staff_code'] as String?,
        expiresAt: DateTime.parse(data['expires_at']),
        sessionId: data['session_id'] as String?,
      );
    } catch (_) { return PersonnelSessionState.initial(); }
  }
  Future<void> save(PersonnelSessionState s) async => _store.write(key: 'personnel_session', value: jsonEncode({
    'authenticated': s.authenticated, 'user_id': s.userId, 'role': s.role,
    'staff_code': s.staffCode, 'expires_at': s.expiresAt.toIso8601String(),
    'session_id': s.sessionId,
  }));
  Future<void> clear() async => _store.delete(key: 'personnel_session');
}
