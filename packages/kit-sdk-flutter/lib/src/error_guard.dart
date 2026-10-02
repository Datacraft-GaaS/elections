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

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// App-wide error capture for eel apps (plan §5.1 error copy, §9 DoD).
///
/// Two tiers:
/// - Framework errors ([FlutterError.onError], i.e. build/layout failures
///   where the app cannot render) → [EelFatalErrorScreen] replaces the
///   route stack. The app is broken; say so with a support ID.
/// - Zone (async) errors → recorded + a dismissible banner over the current
///   screen. The app keeps running (e.g. a background telemetry poll fails);
///   the banner carries the support ID and opens the fatal screen on tap.
///
/// Apps wire it in `main()` BEFORE any other async work and install
/// [eelNavigatorKey] on their [MaterialApp]:
///
/// ```dart
/// void main() {
///   runEelGuardedApp(app: const MyApp());
/// }
/// // ...
/// MaterialApp(navigatorKey: eelNavigatorKey, home: ...)
/// ```
///
/// Error reports go to the injectable [EelErrorReporter] (default: in-memory
/// [EelErrorLog] + debugPrint). Apps forward to telemetry-ingest in M2.
/// Messages shown on screen are truncated and never include stack traces
/// (stacks may carry PII); stacks stay in the in-memory record only.

// ---------------------------------------------------------------------------
// Support IDs
// ---------------------------------------------------------------------------

/// Stable, support-desk-friendly ID: `EEL-<base36 millis>-<4 hex hash>`.
/// Deterministic per (error, millisecond); no device/user data inside.
String eelSupportId(Object error, {DateTime? at}) {
  final ms = (at ?? DateTime.now()).millisecondsSinceEpoch;
  final hash = (error.runtimeType.toString().hashCode ^ ms) & 0xFFFF;
  return 'EEL-${ms.toRadixString(36).toUpperCase()}-${hash.toRadixString(16).padLeft(4, '0').toUpperCase()}';
}

// ---------------------------------------------------------------------------
// Records + log + reporter hook
// ---------------------------------------------------------------------------

/// One captured error.
@immutable
class EelErrorRecord {
  const EelErrorRecord({
    required this.supportId,
    required this.errorType,
    required this.messageSnippet,
    required this.at,
    required this.fatal,
  });

  final String supportId;
  final String errorType;

  /// First 200 chars of `toString()` — may contain app data, never leaves
  /// the device except through the wired reporter.
  final String messageSnippet;
  final DateTime at;
  final bool fatal;
}

/// In-memory ring buffer (last 50) backing the support screen and tests.
abstract final class EelErrorLog {
  static const int capacity = 50;
  static final List<EelErrorRecord> _records = [];

  static List<EelErrorRecord> get all => List.unmodifiable(_records);

  static void add(EelErrorRecord record) {
    _records.add(record);
    if (_records.length > capacity) {
      _records.removeAt(0);
    }
  }

  @visibleForTesting
  static void clearForTest() => _records.clear();
}

/// Receives every captured error. Apps wire telemetry-ingest here.
typedef EelErrorReporter = void Function(EelErrorRecord record);

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Navigator key the guarded app must install on its [MaterialApp].
final GlobalKey<NavigatorState> eelNavigatorKey = GlobalKey<NavigatorState>();

/// Installs capture and runs [app]. Safe to call once per process.
void runEelGuardedApp({
  required Widget app,
  EelErrorReporter? onError,
  FlutterExceptionHandler? previousFlutterHandler,
}) {
  final prev = previousFlutterHandler ?? FlutterError.onError;
  FlutterError.onError = (details) {
    handleFlutterError(details, onError: onError);
    try {
      prev?.call(details);
    } catch (_) {}
  };
  runZonedGuarded(
    () => runApp(app),
    (error, stack) => handleZoneError(error, stack, onError: onError),
  );
}

/// Framework-error entry (also the test seam for the wiring above).
EelErrorRecord handleFlutterError(
  FlutterErrorDetails details, {
  EelErrorReporter? onError,
}) =>
    _capture(details.exception, details.stack,
        fatal: true, onError: onError);

/// Zone-error entry (also the test seam for the wiring above).
EelErrorRecord handleZoneError(
  Object error,
  StackTrace stack, {
  EelErrorReporter? onError,
}) =>
    _capture(error, stack, fatal: false, onError: onError);

EelErrorRecord _capture(
  Object error,
  StackTrace? stack, {
  required bool fatal,
  EelErrorReporter? onError,
}) {
  final record = EelErrorRecord(
    supportId: eelSupportId(error),
    errorType: error.runtimeType.toString(),
    messageSnippet: _snippet(error.toString()),
    at: DateTime.now(),
    fatal: fatal,
  );
  EelErrorLog.add(record);
  // Always visible in logcat (type + ID only — never message/stack, which
  // may carry PII). The wired reporter gets the full record.
  debugPrint('EEL ${fatal ? "FATAL" : "ERROR"} ${record.supportId} ${record.errorType}');
  try {
    onError?.call(record);
  } catch (_) {}
  if (fatal) {
    EelErrorOverlay.showFatal(record);
  } else {
    EelErrorOverlay.showBanner(record);
  }
  return record;
}

String _snippet(String s) {
  final oneLine = s.replaceAll(RegExp(r'\s+'), ' ').trim();
  return oneLine.length <= 200 ? oneLine : '${oneLine.substring(0, 200)}…';
}

// ---------------------------------------------------------------------------
// Presentation (pure Material — no eel_design dependency at device tier)
// ---------------------------------------------------------------------------

/// Overlay + navigation for captured errors. Test seam: [navigatorOverride].
abstract final class EelErrorOverlay {
  @visibleForTesting
  static GlobalKey<NavigatorState>? navigatorOverride;

  static bool _fatalShowing = false;

  @visibleForTesting
  static void resetForTest() => _fatalShowing = false;

  static GlobalKey<NavigatorState> get _key =>
      navigatorOverride ?? eelNavigatorKey;

  /// Replace the route stack with the fatal screen. Re-entrant calls while
  /// the fatal screen is up only record (prevents error loops).
  static void showFatal(EelErrorRecord record) {
    if (_fatalShowing) return;
    final nav = _key.currentState;
    if (nav == null) {
      debugPrint('EEL FATAL ${record.supportId} ${record.errorType}');
      return;
    }
    _fatalShowing = true;
    unawaited(
      nav.pushAndRemoveUntil(
        MaterialPageRoute<void>(
          builder: (_) => EelFatalErrorScreen(record: record),
        ),
        (_) => false,
      ),
    );
  }

  /// Dismissible banner over the current screen; tap opens details.
  static void showBanner(EelErrorRecord record) {
    final context = _key.currentContext;
    if (context == null) {
      debugPrint('EEL ERROR ${record.supportId} ${record.errorType}');
      return;
    }
    final overlay = Navigator.of(context).overlay;
    if (overlay == null) return;
    late final OverlayEntry entry;
    entry = OverlayEntry(
      builder: (ctx) => Positioned(
        top: MediaQuery.of(ctx).padding.top + 8,
        left: 16,
        right: 16,
        child: Material(
          elevation: 6,
          borderRadius: BorderRadius.circular(8),
          color: Theme.of(ctx).colorScheme.errorContainer,
          child: InkWell(
            borderRadius: BorderRadius.circular(8),
            onTap: () {
              entry.remove();
              showFatal(record);
            },
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
                  const Icon(Icons.error_outline),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const Text(
                          'Something went wrong',
                          style: TextStyle(fontWeight: FontWeight.w600),
                        ),
                        Text(
                          'Error ${record.supportId} — tap for details',
                          style: Theme.of(ctx).textTheme.bodySmall,
                        ),
                      ],
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.close),
                    tooltip: 'Dismiss',
                    onPressed: entry.remove,
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
    overlay.insert(entry);
    Future.delayed(const Duration(seconds: 8), () {
      if (entry.mounted) entry.remove();
    });
  }
}

/// Full-screen fatal error: what happened, what to do, support ID.
class EelFatalErrorScreen extends StatefulWidget {
  const EelFatalErrorScreen({super.key, required this.record, this.onRestart});

  final EelErrorRecord record;

  /// Defaults to leaving to the launcher so the user relaunches fresh.
  final VoidCallback? onRestart;

  @override
  State<EelFatalErrorScreen> createState() => _EelFatalErrorScreenState();
}

class _EelFatalErrorScreenState extends State<EelFatalErrorScreen> {
  bool _showDetails = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Scaffold(
      body: SafeArea(
        child: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.error_outline,
                    size: 48, color: theme.colorScheme.error),
                const SizedBox(height: 16),
                Text(
                  'Something went wrong',
                  style: theme.textTheme.headlineSmall,
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),
                const Text(
                  'The app hit a problem it cannot recover from. '
                  'Please restart the app. If it keeps happening, '
                  'contact support with the error ID below.',
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 16),
                SelectableText(
                  'Error ID: ${widget.record.supportId}',
                  style: theme.textTheme.bodyMedium?.copyWith(
                    fontFamily: 'monospace',
                    fontWeight: FontWeight.w600,
                  ),
                  textAlign: TextAlign.center,
                ),
                if (_showDetails) ...[
                  const SizedBox(height: 8),
                  Text(
                    '${widget.record.errorType}: ${widget.record.messageSnippet}',
                    style: theme.textTheme.bodySmall,
                    textAlign: TextAlign.center,
                  ),
                ],
                const SizedBox(height: 24),
                FilledButton(
                  onPressed: widget.onRestart ??
                      () => SystemNavigator.pop(),
                  style: FilledButton.styleFrom(
                    minimumSize: const Size(0, 48),
                  ),
                  child: const Text('Restart app'),
                ),
                TextButton(
                  onPressed: () =>
                      setState(() => _showDetails = !_showDetails),
                  child: Text(_showDetails ? 'Hide details' : 'Details'),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}