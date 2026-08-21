// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'Party Agent';

  @override
  String get loginButton => 'Login';

  @override
  String get submitButton => 'Submit';

  @override
  String get errorGeneric => 'An error occurred. Please try again.';

  @override
  String get loadingMessage => 'Loading...';

  @override
  String get checkInButton => 'Check In';

  @override
  String get verifyFormButton => 'Verify Form 34A';

  @override
  String get countersignButton => 'Countersign';

  @override
  String get disputeButton => 'Raise Dispute';

  @override
  String get duressAlertButton => 'Silent Alert';
}
