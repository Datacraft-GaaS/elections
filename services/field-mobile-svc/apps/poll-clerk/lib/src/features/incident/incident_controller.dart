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

/// Incident reporting controller for poll-clerk.
/// Per FR-M31-016/017/018: structured incident form, free text, evidence, escalation chain.
/// Implements §5.3 state matrix (loading/loaded/empty/offline/error/permission/tamper).
library;

import 'package:flutter/foundation.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import '../../core/services/api_client.dart';

/// Incident severity levels (M31)
enum IncidentSeverity { low, medium, high, critical }

/// Incident category (M31)
enum IncidentCategory {
  security,         // Security issue
  procedural,       // Procedure violation
  equipment,        // Equipment failure
  connectivity,     // Network issue
  other,            // Other
}

/// State matrix per plan.md §5.3
enum IncidentScreenState { loading, loaded, empty, offline, error, permissionDenied, unauthenticated, tamperBlocked }

/// Immutable incident draft (UI state)
@immutable
class IncidentDraft {
  final String id;
  final IncidentCategory category;
  final IncidentSeverity severity;
  final String description;
  final List<String> evidenceRefs;
  final DateTime updatedAt;
  final bool offlineQueued;

  const IncidentDraft({
    required this.id,
    this.category = IncidentCategory.other,
    this.severity = IncidentSeverity.medium,
    this.description = '',
    this.evidenceRefs = const [],
    required this.updatedAt,
    this.offlineQueued = false,
  });

  /// The draft before anything has been entered.
  ///
  /// updatedAt is epoch rather than nullable, because "when was this draft
  /// last touched" has no meaningful answer before it exists; the UI shows the
  /// empty state, not a timestamp of 1970.
  static final IncidentDraft empty = IncidentDraft(
    id: '',
    updatedAt: DateTime.fromMillisecondsSinceEpoch(0, isUtc: true),
  );

  IncidentDraft copyWith({
    IncidentCategory? category,
    IncidentSeverity? severity,
    String? description,
    List<String>? evidenceRefs,
    DateTime? updatedAt,
    bool? offlineQueued,
  }) => IncidentDraft(
    id: id,
    category: category ?? this.category,
    severity: severity ?? this.severity,
    description: description ?? this.description,
    evidenceRefs: evidenceRefs ?? this.evidenceRefs,
    updatedAt: updatedAt ?? this.updatedAt,
    offlineQueued: offlineQueued ?? this.offlineQueued,
  );

  bool get isValid => description.length >= 10 && description.length <= 2048;
}

/// Submitted incident result
@immutable
class SubmittedIncident {
  final String incidentId;
  final String status;
  final DateTime createdAt;
  const SubmittedIncident({required this.incidentId, required this.status, required this.createdAt});
}

/// Incident controller state
@immutable
class IncidentState {
  final IncidentScreenState screenState;
  final IncidentDraft draft;
  final SubmittedIncident? submitted;
  final String? errorMessage;
  final String? supportCode;
  final int? httpStatus;

  IncidentState({
    this.screenState = IncidentScreenState.empty,
    IncidentDraft? draft,
    this.submitted,
    this.errorMessage,
    this.supportCode,
    this.httpStatus,
  }) : draft = draft ?? IncidentDraft.empty;

  factory IncidentState.initial() => IncidentState(
    screenState: IncidentScreenState.empty,
    draft: IncidentDraft.empty,
  );

  IncidentState copyWith({
    IncidentScreenState? screenState,
    IncidentDraft? draft,
    SubmittedIncident? submitted,
    String? errorMessage,
    String? supportCode,
    int? httpStatus,
  }) => IncidentState(
    screenState: screenState ?? this.screenState,
    draft: draft ?? this.draft,
    submitted: submitted ?? this.submitted,
    errorMessage: errorMessage ?? this.errorMessage,
    supportCode: supportCode ?? this.supportCode,
    httpStatus: httpStatus ?? this.httpStatus,
  );
}

/// Incident controller (Riverpod Notifier)
class IncidentController extends StateNotifier<IncidentState> {
  final EelHttpClient? _client;
  final bool _isOffline;

  IncidentController({EelHttpClient? client, bool isOffline = false})
      : _client = client,
        _isOffline = isOffline,
        super(IncidentState.initial());

  /// Update draft fields
  void updateCategory(IncidentCategory category) {
    state = state.copyWith(draft: state.draft.copyWith(category: category, updatedAt: DateTime.now()));
  }

  void updateSeverity(IncidentSeverity severity) {
    state = state.copyWith(draft: state.draft.copyWith(severity: severity, updatedAt: DateTime.now()));
  }

  void updateDescription(String description) {
    state = state.copyWith(draft: state.draft.copyWith(description: description, updatedAt: DateTime.now()));
  }

  void addEvidence(String evidenceRef) {
    final newRefs = [...state.draft.evidenceRefs, evidenceRef];
    state = state.copyWith(draft: state.draft.copyWith(evidenceRefs: newRefs, updatedAt: DateTime.now()));
  }

  /// Submit incident (idempotent)
  Future<void> submit(String stationCode) async {
    if (!state.draft.isValid) {
      state = state.copyWith(
        screenState: IncidentScreenState.error,
        errorMessage: 'Description must be 10-2048 characters',
        httpStatus: 422,
      );
      return;
    }

    state = state.copyWith(screenState: IncidentScreenState.loading);

    if (_isOffline || _client == null) {
      // Queue offline (journal-before-act per §2.5)
      state = state.copyWith(
        screenState: IncidentScreenState.loaded,
        draft: state.draft.copyWith(offlineQueued: true, updatedAt: DateTime.now()),
        submitted: SubmittedIncident(
          incidentId: state.draft.id,
          status: 'QUEUED_OFFLINE',
          createdAt: DateTime.now(),
        ),
      );
      return;
    }

    try {
      final response = await _client.post(
        '/api/v1/mobile/bff/incidents',
        data: {
          'incident_id': state.draft.id,
          'category': state.draft.category.name.toUpperCase(),
          'severity': state.draft.severity.name.toUpperCase(),
          'description': state.draft.description,
          'station_code': stationCode,
          'evidence_refs': state.draft.evidenceRefs,
        },
      );

      if (response.statusCode == 201 || response.statusCode == 200) {
        final data = response.data as Map<String, dynamic>;
        state = state.copyWith(
          screenState: IncidentScreenState.loaded,
          submitted: SubmittedIncident(
            incidentId: data['incident_id'] ?? state.draft.id,
            status: data['status'] ?? 'REPORTED',
            createdAt: DateTime.now(),
          ),
        );
      } else if (response.statusCode == 403) {
        state = state.copyWith(
          screenState: IncidentScreenState.permissionDenied,
          errorMessage: 'You do not have permission to report incidents at this station',
        );
      } else if (response.statusCode == 401) {
        state = state.copyWith(
          screenState: IncidentScreenState.unauthenticated,
          errorMessage: 'Session expired. Please sign in again.',
        );
      } else if (response.statusCode == 422) {
        state = state.copyWith(
          screenState: IncidentScreenState.error,
          errorMessage: 'Invalid incident data',
          httpStatus: 422,
        );
      } else {
        state = state.copyWith(
          screenState: IncidentScreenState.error,
          errorMessage: 'Server returned ${response.statusCode}',
          httpStatus: response.statusCode,
        );
      }
    } catch (e) {
      state = state.copyWith(
        screenState: IncidentScreenState.offline,
        draft: state.draft.copyWith(offlineQueued: true),
        errorMessage: 'Network error. Incident queued for retry.',
      );
    }
  }

  /// Clear draft for new incident
  void clear() {
    state = IncidentState.initial();
  }
}

/// Provider for incident controller
final incidentProvider = StateNotifierProvider<IncidentController, IncidentState>((ref) {
  return IncidentController();
});
