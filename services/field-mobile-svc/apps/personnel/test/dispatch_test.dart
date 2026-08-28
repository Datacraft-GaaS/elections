import 'package:eel_personnel/src/models/worker_assignment.dart';
import 'package:eel_personnel/src/models/worker_check_in.dart';
import 'package:eel_personnel/src/models/worker_check_out.dart';
import 'package:eel_personnel/src/models/worker_status_ping.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('parses check-in response (camelCase)', () {
    final c = WorkerCheckIn.fromJson({
      'id': 'c1',
      'workerId': 'w1',
      'stationCode': 'STN-001',
      'electionId': 'e1',
      'biometricHash': 'b1',
      'gpsLat': -1.2921,
      'gpsLon': 36.8219,
      'gpsAccuracyM': 8.0,
      'checkInAt': '2026-08-08T05:00:00Z',
      'signature': 'sig',
    });
    expect(c.stationCode, 'STN-001');
    expect(c.gpsLat, -1.2921);
    expect(c.checkInAt, DateTime.utc(2026, 8, 8, 5));
  });

  test('parses check-out response with hours and optional payroll trigger', () {
    final co = WorkerCheckOut.fromJson({
      'id': 'co1',
      'workerId': 'w1',
      'stationCode': 'STN-001',
      'electionId': 'e1',
      'gpsLat': -1.29,
      'gpsLon': 36.82,
      'checkOutAt': '2026-08-08T18:00:00Z',
      'hoursWorked': 12.5,
      'payrollTriggerId': null,
      'signature': 'sig',
    });
    expect(co.hoursWorked, 12.5);
    expect(co.payrollTriggerId, isNull);
  });

  test('status enum round-trips through dispatch SCREAMING_SNAKE_CASE', () {
    expect(WorkerStatus.onStation.apiValue, 'ON_STATION');
    expect(WorkerStatus.onBreak.apiValue, 'BREAK');
    expect(WorkerStatus.absent.apiValue, 'ABSENT');
    expect(WorkerStatus.fromApi('BREAK'), WorkerStatus.onBreak);
    expect(WorkerStatus.fromApi('unknown'), WorkerStatus.onStation);
  });

  test('parses status ping response', () {
    final p = WorkerStatusPing.fromJson({
      'id': 'p1',
      'workerId': 'w1',
      'stationCode': 'STN-001',
      'status': 'ABSENT',
      'reason': null,
      'pingedAt': '2026-08-08T12:00:00Z',
    });
    expect(p.status, WorkerStatus.absent);
    expect(p.pingedAt, DateTime.utc(2026, 8, 8, 12));
  });

  test('assignment status enum parses SCREAMING_SNAKE_CASE', () {
    expect(AssignmentStatus.fromApi('ACCEPTED'), AssignmentStatus.accepted);
    expect(AssignmentStatus.fromApi('REASSIGNED'), AssignmentStatus.reassigned);
    expect(AssignmentStatus.fromApi('bogus'), AssignmentStatus.pending);
  });

  test('parses assignment response', () {
    final a = WorkerAssignment.fromJson({
      'id': 'a1',
      'workerId': 'w1',
      'stationCode': 'STN-022-004',
      'electionId': 'e1',
      'role': 'clerk',
      'reportTime': '2026-08-08T05:00:00Z',
      'supervisorId': 'ro-1',
      'status': 'ACCEPTED',
      'acceptedAt': '2026-08-01T09:00:00Z',
    });
    expect(a.status, AssignmentStatus.accepted);
    expect(a.role, 'clerk');
    expect(a.acceptedAt, DateTime.utc(2026, 8, 1, 9));
  });
}
