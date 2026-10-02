/// LMS Offline Modules — Beautiful, ergonomic, accessible.
/// Uses eel_design: EelScaffold, EelResultCard, EelButton, EelStatusChip, EelChips, EelBanner.
library;
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

import 'module_model.dart';
import 'module_repository.dart';
import '../sync/offline_queue.dart';
import 'package:dio/dio.dart';

final _offlineQueueProvider = Provider((ref) => OfflineQueue());
final _moduleRepoProvider = Provider((ref) {
  final queue = ref.watch(_offlineQueueProvider);
  return ModuleRepository(
    dio: Dio(BaseOptions(baseUrl: 'https://training.eel.example')),
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
    final colors = EelColors.of(context);

    return EelScaffold(
      title: 'Training Modules',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(16),
      body: cachedAsync.when(
        loading: () => const Center(
          child: EelLoadingState(),
        ),
        error: (e, st) => Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.cloud_off, size: 48, color: colors.onSurfaceMuted),
            const SizedBox(height: 16),
            const Text('Could not load modules', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
            const SizedBox(height: 8),
            const Text('Why: offline or server error. Fix: check connection and retry. If it persists, open Sync.',
                style: EelTextStyle.bodyMedium, textAlign: TextAlign.center),
            const SizedBox(height: 8),
            Text('Prevention: sync modules before going offline.',
                style: EelTextStyle.bodySmall.copyWith(color: colors.onSurfaceMuted), textAlign: TextAlign.center),
            const SizedBox(height: 20),
            EelButton(
              label: 'Retry',
              icon: Icons.refresh,
              onPressed: () => ref.invalidate(_cachedModulesProvider),
            ),
          ],
        ),
        data: (modules) => _ModuleList(
          modules: modules,
          onDownload: (id) async {
            final repo = ref.read(_moduleRepoProvider);
            if (modules.length >= 5) {
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('5 module limit reached. Remove a module before downloading another.'),
                  ),
                );
              }
              return;
            }
            try {
              await repo.downloadModule(id);
              ref.invalidate(_cachedModulesProvider);
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Module downloaded — ${modules.length + 1}/5 used'),
                      action: SnackBarAction(label: 'View', onPressed: () => context.go('/modules/$id'))),
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
              const Icon(Icons.menu_book_outlined, size: 48),
              const SizedBox(height: 16),
              const Text('No modules yet', style: EelTextStyle.titleLarge, textAlign: TextAlign.center),
              const SizedBox(height: 8),
              const Text('Connect and download modules for offline study. Up to 5 modules can be stored.',
                  style: EelTextStyle.bodyMedium, textAlign: TextAlign.center),
              const SizedBox(height: 20),
              EelButton(
                label: 'Browse modules',
                icon: Icons.download,
                onPressed: () => context.go('/sync'),
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
        return EelResultCard(
          leading: Icon(Icons.book, color: EelColors.of(context).primary, size: 32),
          title: Text(m.title, style: EelTextStyle.titleMedium),
          subtitle: Text('${m.moduleType.name} | ${m.language}', style: EelTextStyle.bodyMedium),
        );
      },
    );
  }
}
