/// Offline queue extension for diaspora forms with deferred transmission.
///
/// Extends the kit-sdk queue concept with a [deferredTransmissionAt] timestamp
/// so that batches can be held until a scheduled release time.

import 'models.dart';

/// A queued diaspora form with a scheduled release time.
class DeferredForm {
  const DeferredForm({
    required this.form,
    required this.deferredTransmissionAt,
    this.attempts = 0,
  });

  final Form34AEntry form;

  /// UTC timestamp when this form's batch should be released for transmission.
  final DateTime deferredTransmissionAt;

  /// Number of delivery attempts already made.
  final int attempts;

  /// Returns true if the form is ready to transmit as of [now].
  bool isReadyAt(DateTime now) => now.isAfter(deferredTransmissionAt);

  DeferredForm withAttempt() => DeferredForm(
        form: form,
        deferredTransmissionAt: deferredTransmissionAt,
        attempts: attempts + 1,
      );

  @override
  String toString() =>
      'DeferredForm(formId=${form.formId}, releaseAt=$deferredTransmissionAt, attempts=$attempts)';
}

/// In-memory offline queue for diaspora forms.
/// In production this would persist to Isar via eel_kit_sdk.
class OfflineQueueDiaspora {
  final List<DeferredForm> _queue = [];

  void enqueue(Form34AEntry form, DateTime deferredTransmissionAt) {
    _queue.add(DeferredForm(form: form, deferredTransmissionAt: deferredTransmissionAt));
  }

  /// Returns all forms whose deferred time has passed as of [now].
  List<DeferredForm> readyForms(DateTime now) =>
      _queue.where((f) => f.isReadyAt(now)).toList();

  /// Remove a form from the queue after successful delivery.
  void remove(String formId) {
    _queue.removeWhere((f) => f.form.formId == formId);
  }

  /// Record a failed attempt (increments counter).
  void recordFailure(String formId) {
    final idx = _queue.indexWhere((f) => f.form.formId == formId);
    if (idx >= 0) {
      _queue[idx] = _queue[idx].withAttempt();
    }
  }

  int get length => _queue.length;
  List<DeferredForm> get all => List.unmodifiable(_queue);
}
