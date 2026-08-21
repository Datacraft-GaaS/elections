// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'Observer';

  @override
  String get loginButton => 'Login';

  @override
  String get submitButton => 'Submit';

  @override
  String get errorGeneric => 'An error occurred. Please try again.';

  @override
  String get loadingMessage => 'Loading...';

  @override
  String get stationVisitButton => 'Log Station Visit';

  @override
  String get reportIncidentButton => 'Report Incident';

  @override
  String get verifyTallyButton => 'Verify Tally';

  @override
  String get exportMissionButton => 'Export Mission PDF';

  @override
  String get ussdFallbackNote => 'USSD fallback available';
}
