import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'crl_model.dart';
import 'crl_repository.dart';

const _kRefreshInterval = Duration(minutes: 5);
const _kStaleThreshold = Duration(minutes: 30);

final crlStaleProvider = StateProvider<bool>((ref) => false);

final crlModelProvider = StateProvider<CrlModel?>((ref) => null);

class CrlSyncService {
  final CrlRepository _repo;
  final Ref _ref;
  Timer? _timer;

  CrlSyncService({required CrlRepository repo, required Ref ref})
      : _repo = repo,
        _ref = ref;

  void start() {
    _refresh();
    _timer = Timer.periodic(_kRefreshInterval, (_) => _refresh());
  }

  void stop() {
    _timer?.cancel();
    _timer = null;
  }

  Future<void> _refresh() async {
    try {
      final crl = await _repo.fetchCrl();
      _ref.read(crlModelProvider.notifier).state = crl;
      final stale = DateTime.now().difference(crl.generatedAt) > _kStaleThreshold;
      _ref.read(crlStaleProvider.notifier).state = stale;
    } catch (_) {
      // Network unavailable: check if cached CRL is stale
      final stale = await _repo.isCrlStale();
      _ref.read(crlStaleProvider.notifier).state = stale;
    }
  }
}
