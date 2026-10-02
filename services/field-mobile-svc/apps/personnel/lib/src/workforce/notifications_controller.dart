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

import 'package:dio/dio.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'workforce_api.dart';

class NotificationsState {
  final List<WorkforceNotification> notifications;
  final bool loading;
  final String? errorMessage;

  const NotificationsState({
    this.notifications = const [],
    this.loading = false,
    this.errorMessage,
  });

  bool get hasCachedData => notifications.isNotEmpty;

  NotificationsState copyWith({
    List<WorkforceNotification>? notifications,
    bool? loading,
    String? errorMessage,
  }) =>
      NotificationsState(
        notifications: notifications ?? this.notifications,
        loading: loading ?? this.loading,
        errorMessage: errorMessage == _clear ? null : (errorMessage ?? this.errorMessage),
      );

  static const _clear = '__clear__';
  static const clearError = _clear;
}

/// Read-only controller for the notifications inbox (design §4/§8).
///
/// Cached data survives a failed refresh so a worker with poor connectivity
/// still sees their latest postings and payments.
class NotificationsController extends ChangeNotifier {
  NotificationsController({
    WorkforceApi? api,
    String workerId = 'local-worker',
  })  : _api = api ?? WorkforceApi(),
        _workerId = workerId;

  final WorkforceApi _api;
  final String _workerId;

  NotificationsState state = const NotificationsState();

  Future<void> refresh() async {
    state = state.copyWith(loading: true);
    notifyListeners();
    try {
      final notifications = await _api.listNotifications(_workerId);
      state = state.copyWith(
        // Newest first for the inbox.
        notifications: notifications.reversed.toList(),
        loading: false,
        errorMessage: NotificationsState.clearError,
      );
    } catch (e) {
      state = state.copyWith(
        loading: false,
        errorMessage: describeNotificationError(e),
      );
    }
    notifyListeners();
  }
}

String describeNotificationError(Object error) {
  if (error is DioException) {
    final data = error.response?.data;
    if (data is Map && data['error'] != null) return data['error'].toString();
    if (data is String && data.isNotEmpty) return data;
    return error.message ?? 'network error';
  }
  return error.toString();
}

final notificationsApiProvider = Provider<WorkforceApi>((ref) => WorkforceApi());

final notificationsControllerProvider =
    ChangeNotifierProvider<NotificationsController>((ref) {
  return NotificationsController(api: ref.watch(notificationsApiProvider));
});
