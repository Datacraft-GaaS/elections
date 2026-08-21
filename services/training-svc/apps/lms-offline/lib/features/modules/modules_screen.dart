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
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, st) => Center(child: Text('Error: $e')),
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
            } catch (e) {
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Download failed: $e')),
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
      return const Center(
        child: Text('No modules downloaded. Connect to download modules.'),
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
              Chip(
                label: Text(m.moduleType.name),
                backgroundColor: _typeColor(m.moduleType),
              ),
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.quiz),
                tooltip: 'Take assessment',
                onPressed: () =>
                    context.go('/assessment/${m.id}'),
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
