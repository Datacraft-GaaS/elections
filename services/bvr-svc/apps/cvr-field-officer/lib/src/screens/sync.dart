/// SyncScreen: offline queue depth, retry controls, last successful sync.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../config.dart';
import '../models/sync_status.dart';
import '../services/api_client.dart';
import '../services/app_identity.dart';
import '../services/offline_queue.dart';

class SyncScreen extends ConsumerStatefulWidget {
  const SyncScreen({super.key});

  @override
  ConsumerState<SyncScreen> createState() => _SyncScreenState();
}

class _SyncScreenState extends ConsumerState<SyncScreen> {
  SyncStatus _status = const SyncStatus.idle();
  bool _isSyncing = false;

  @override
  void initState() {
    super.initState();
    _refreshStatus();
  }

  void _refreshStatus() {
    setState(() {
      _status = SyncStatus(
        state: SyncState.idle,
        pendingCount: OfflineQueueManager().pendingCount,
      );
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Sync')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Semantics(
                header: true,
                child: Text(
                  'Offline Queue',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ),
              const SizedBox(height: 12),

              Card(
                child: ListTile(
                  leading: const Icon(Icons.pending_actions),
                  title: Text('${_status.pendingCount} captures pending'),
                  subtitle: _status.lastSuccessfulSync != null
                      ? Text(
                          'Last sync: ${_status.lastSuccessfulSync!.toLocal()}',
                        )
                      : const Text('Never synced'),
                ),
              ),

              const SizedBox(height: 24),

              if (_isSyncing)
                const Center(child: CircularProgressIndicator())
              else
                SizedBox(
                  width: double.infinity,
                  height: 56,
                  child: FilledButton.icon(
                    icon: const Icon(Icons.sync),
                    label: const Text('Retry Sync Now'),
                    onPressed: _status.pendingCount == 0 ? null : _sync,
                  ),
                ),

              if (_status.state == SyncState.error && _status.errorMessage != null) ...[
                const SizedBox(height: 12),
                Semantics(
                  liveRegion: true,
                  child: Card(
                    color: Theme.of(context).colorScheme.errorContainer,
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Text(
                        _status.errorMessage!,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.onErrorContainer,
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _sync() async {
    setState(() => _isSyncing = true);

    await AppIdentity.instance.ensureAttested();
    final client = ApiClient(
      baseUri: AppConfig.apiBaseUri,
      svidProvider: () => AppIdentity.instance.spiffeId,
    );

    final result = await OfflineQueueManager().drain(
      sender: (record) async {
        try {
          await client.submitCapture(record);
          return true;
        } on Exception {
          return false;
        }
      },
    );

    setState(() {
      _status = result;
      _isSyncing = false;
    });
  }
}
