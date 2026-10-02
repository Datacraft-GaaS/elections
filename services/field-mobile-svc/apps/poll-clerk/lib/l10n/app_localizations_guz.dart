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

// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Gusii Ekegusii (`guz`).
class AppLocalizationsGuz extends AppLocalizations {
  AppLocalizationsGuz([String locale = 'guz']) : super(locale);

  @override
  String get appTitle => 'Omochie';

  @override
  String get loginButton => 'Ingia';

  @override
  String get submitButton => 'Wasilisha';

  @override
  String get errorGeneric => 'Hitilafu imetokea. Jaribu tena.';

  @override
  String get loadingMessage => 'Inapakia...';

  @override
  String get stationDashboardTitle => 'Dashibodi ya Kituo';

  @override
  String get reportIncidentButton => 'Ripoti Tukio';

  @override
  String get eodReportButton => 'Ripoti ya Mwisho wa Siku';

  @override
  String get kioskModeButton => 'Hali ya Kiosk';

  @override
  String get paperBackupNote => 'Nakala ya karatasi inapatikana';
}
