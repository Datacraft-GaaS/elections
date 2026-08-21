// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Masai (`mas`).
class AppLocalizationsMas extends AppLocalizations {
  AppLocalizationsMas([String locale = 'mas']) : super(locale);

  @override
  String get appTitle => 'Enkiama';

  @override
  String get loginButton => 'Kiri';

  @override
  String get submitButton => 'Wasilisha';

  @override
  String get errorGeneric => 'Hitilafu imetokea. Jaribu tena.';

  @override
  String get loadingMessage => 'Inapakia...';

  @override
  String get stationVisitButton => 'Kumbuka Ziara ya Kituo';

  @override
  String get reportIncidentButton => 'Ripoti Tukio';

  @override
  String get verifyTallyButton => 'Thibitisha Hesabu';

  @override
  String get exportMissionButton => 'Hamisha PDF ya Ujumbe';

  @override
  String get ussdFallbackNote => 'USSD inapatikana kama hifadhi';
}
