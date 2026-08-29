import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

import 'module_model.dart';
import 'module_repository.dart';
import '../sync/offline_queue.dart';
import 'package:dio/dio.dart';

final _offlineQueueProvider = Provider((ref) => OfflineQueue());
final _moduleRepoProvider = Provider((ref) {
  final queue = ref.watch(_offlineQueueProvider);
  return ModuleRepository(
    dio: Dio(BaseOptions(baseUrl: 'https://training.eel.iebc.go.ke')),
    queue: queue,
  );
});

final _cachedModulesProvider = FutureProvider<List<CourseModule>>((ref) async {
  final repo = ref.watch(_moduleRepoProvider);
  return repo.getCachedModules();
});

class ModulesScreen extends HookConsumerWidget {
  final String? selectedId;

  const ModulesScreen({super.key, this.selectedId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final cachedAsync = ref.watch(_cachedModulesProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Training Modules'),
        actions: [
          IconButton(
            icon: const Icon(Icons.sync),
            tooltip: 'Sync',
            onPressed: () => context.go('/sync'),
          ),
        ],
      ),
      body: cachedAsync.when(
        loading: () => Semantics(
          liveRegion: true,
          label: 'Loading training modules',
          child: const Center(child: CircularProgressIndicator()),
        ),
        error: (e, st) => Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.cloud_off, size: 48, color: Colors.black54),
                const SizedBox(height: 16),
                Semantics(header: true, child: Text('Could not load modules', style: Theme.of(context).textTheme.titleLarge, textAlign: TextAlign.center)),
                const SizedBox(height: 8),
                Text('Why: offline or server error. Fix: check connection and retry. If it persists, open Sync.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
                const SizedBox(height: 8),
                Text('Prevention: sync modules before going offline.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.black54)),
                const SizedBox(height: 20),
                Semantics(
                  button: true,
                  label: 'Retry loading modules',
                  child: ElevatedButton.icon(
                    onPressed: () => ref.invalidate(_cachedModulesProvider),
                    icon: const Icon(Icons.refresh),
                    label: const Text('Retry'),
                    style: ElevatedButton.styleFrom(minimumSize: const Size(44, 44)),
                  ),
                ),
              ],
            ),
          ),
        ),
        data: (modules) => _ModuleList(
          modules: modules,
          onDownload: (id) async {
            final repo = ref.read(_moduleRepoProvider);
            if (modules.length >= 5) {
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text(
                    '5 module limit reached. Remove a module before downloading another.',
                  ),
                ),
              );
              return;
            }
            try {
              await repo.downloadModule(id);
              ref.invalidate(_cachedModulesProvider);
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Module downloaded — ${modules.length + 1}/5 used'), action: SnackBarAction(label: 'View', onPressed: () => context.go('/modules/$id'))),
                );
              }
            } catch (e) {
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Download failed: $e. Why: network or storage. Fix: check connection and free space, then retry.')),
                );
              }
            }
          },
        ),
      ),
    );
  }
}

class _ModuleList extends StatelessWidget {
  final List<CourseModule> modules;
  final void Function(String id) onDownload;

  const _ModuleList({required this.modules, required this.onDownload});

  @override
  Widget build(BuildContext context) {
    if (modules.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.menu_book_outlined, size: 48, color: Colors.black54),
              const SizedBox(height: 16),
              Semantics(header: true, child: Text('No modules yet', style: Theme.of(context).textTheme.titleLarge, textAlign: TextAlign.center)),
              const SizedBox(height: 8),
              Text('Connect and download modules for offline study. Up to 5 modules can be stored.', textAlign: TextAlign.center, style: Theme.of(context).textTheme.bodyMedium),
              const SizedBox(height: 20),
              Semantics(
                button: true,
                label: 'Go to sync to download modules',
                child: ElevatedButton.icon(
                  onPressed: () => context.go('/sync'),
                  icon: const Icon(Icons.download),
                  label: const Text('Browse modules'),
                  style: ElevatedButton.styleFrom(minimumSize: const Size(44, 44)),
                ),
              ),
            ],
          ),
        ),
      );
    }
    return ListView.builder(
      itemCount: modules.length,
      itemBuilder: (context, index) {
        final m = modules[index];
        return ListTile(
          title: Text(m.title),
          subtitle: Text('${m.moduleType.name} | ${m.language}'),
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Semantics(
                label: 'Module type ${m.moduleType.name}',
                child: Chip(
                  label: Text(m.moduleType.name),
                  backgroundColor: _typeColor(m.moduleType),
                ),
              ),
              const SizedBox(width: 8),
              Semantics(
                button: true,
                label: 'Take assessment for ${m.title}',
                child: IconButton(
                  icon: const Icon(Icons.quiz),
                  tooltip: 'Take assessment',
                  onPressed: () => context.go('/assessment/${m.id}'),
                  style: IconButton.styleFrom(minimumSize: const Size(44, 44)),
                ),
              ),
            ],
          ),
          onTap: () => context.go('/modules/${m.id}'),
        );
      },
    );
  }

  Color _typeColor(ModuleType type) {
    switch (type) {
      case ModuleType.core:
        return Colors.blue.shade100;
      case ModuleType.refresher:
        return Colors.orange.shade100;
      case ModuleType.briefing:
        return Colors.green.shade100;
    }
  }
}
