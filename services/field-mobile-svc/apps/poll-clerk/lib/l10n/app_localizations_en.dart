// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'Poll Clerk';

  @override
  String get loginButton => 'Login';

  @override
  String get submitButton => 'Submit';

  @override
  String get errorGeneric => 'An error occurred. Please try again.';

  @override
  String get loadingMessage => 'Loading...';

  @override
  String get stationDashboardTitle => 'Station Dashboard';

  @override
  String get reportIncidentButton => 'Report Incident';

  @override
  String get eodReportButton => 'End-of-Day Report';

  @override
  String get kioskModeButton => 'Kiosk Mode';

  @override
  String get paperBackupNote => 'Paper backup available';
}
