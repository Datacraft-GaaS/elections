import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../theme.dart';
import 'duty_controller.dart';
import 'workforce_api.dart';

class DutyScreen extends ConsumerWidget {
  const DutyScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final controller = ref.watch(dutyControllerProvider);
    final state = controller.state;

    return Scaffold(
      appBar: AppBar(
        title: const Text('My Duty'),
        actions: [
          IconButton(
            icon: const Icon(Icons.sync),
            tooltip: 'Sync',
            onPressed: () async {
              await controller.sync();
              await controller.refresh();
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('Sync complete')),
                );
              }
            },
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Refresh',
            onPressed: () => controller.refresh(),
          ),
        ],
      ),
      body: state.loading && !state.hasCachedData
          ? const Center(child: CircularProgressIndicator())
          : RefreshIndicator(
              onRefresh: () => controller.refresh(),
              child: ListView(
                physics: const AlwaysScrollableScrollPhysics(),
                children: [
                  if (state.errorMessage != null)
                    _ErrorBanner(message: state.errorMessage!),
                  if (controller.pendingEntries.isNotEmpty)
                    _PendingSyncBanner(count: controller.pendingEntries.length),
                  if (state.postings.isEmpty && !state.loading)
                    const Padding(
                      padding: EdgeInsets.all(24),
                      child: Center(child: Text('No active posting')),
                    )
                  else
                    for (final posting in state.postings) ...[
                      _PostingHeader(posting: posting),
                      for (final status in TaskStatus.values)
                        _TaskGroup(
                          postingId: posting.postingId,
                          status: status,
                          tasks:
                              (state.tasksByPosting[posting.postingId] ?? [])
                                  .where((t) => t.status == status)
                                  .toList(),
                          onAck: (t) => _ack(context, controller, t),
                          onDone: (t) => _doneFlow(context, controller, t),
                        ),
                    ],
                ],
              ),
            ),
    );
  }

  Future<void> _ack(
    BuildContext context,
    DutyController controller,
    WorkforceTask task,
  ) async {
    try {
      await controller.ackTask(task);
      final err = controller.state.errorMessage;
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
          content: Text(err ?? 'Task acknowledged'),
        ));
      }
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
          content: Text(describeDutyError(e)),
        ));
      }
    }
  }

  Future<void> _doneFlow(
    BuildContext context,
    DutyController controller,
    WorkforceTask task,
  ) async {
    String? evidenceHash;
    if (task.critical) {
      evidenceHash = await showDialog<String>(
        context: context,
        builder: (_) => _EvidenceHashDialog(taskTitle: task.title),
      );
      // User cancelled or submitted empty hash.
      if (evidenceHash == null) return;
      if (evidenceHash.isEmpty) {
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
            content: Text('Evidence hash is required for critical tasks'),
          ));
        }
        return;
      }
    }
    try {
      await controller.doneTask(task, evidenceHash);
      final err = controller.state.errorMessage;
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
          content: Text(err ?? 'Task done'),
        ));
      }
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
          content: Text(describeDutyError(e)),
        ));
      }
    }
  }
}

class _ErrorBanner extends StatelessWidget {
  const _ErrorBanner({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Theme.of(context).colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Icon(Icons.cloud_off,
                color: Theme.of(context).colorScheme.onErrorContainer),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                message,
                style: TextStyle(
                    color: Theme.of(context).colorScheme.onErrorContainer),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _PendingSyncBanner extends StatelessWidget {
  const _PendingSyncBanner({required this.count});

  final int count;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.fromLTRB(12, 12, 12, 0),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: EelColors.goldTint,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: const Color(0xFFE7D5AB)),
      ),
      child: Row(
        children: [
          const Icon(Icons.cloud_upload_outlined, color: EelColors.gold),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              '$count ${count == 1 ? 'change' : 'changes'} saved offline — will sync when you reconnect.',
              style: const TextStyle(color: EelColors.ink, fontWeight: FontWeight.w600),
            ),
          ),
        ],
      ),
    );
  }
}

class _PostingHeader extends StatelessWidget {
  const _PostingHeader({required this.posting});

  final WorkforcePosting posting;

  @override
  Widget build(BuildContext context) {
    final fmt = MaterialLocalizations.of(context);
    return Card(
      margin: const EdgeInsets.fromLTRB(12, 12, 12, 4),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '${posting.stationCode} — ${posting.roleCode}',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 4),
            Text(
              'Effective ${fmt.formatCompactDate(posting.effectiveFrom)}'
              '${posting.effectiveTo != null ? ' – ${fmt.formatCompactDate(posting.effectiveTo!)}' : ''}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }
}

class _TaskGroup extends StatelessWidget {
  const _TaskGroup({
    required this.postingId,
    required this.status,
    required this.tasks,
    required this.onAck,
    required this.onDone,
  });

  final String postingId;
  final TaskStatus status;
  final List<WorkforceTask> tasks;
  final void Function(WorkforceTask) onAck;
  final void Function(WorkforceTask) onDone;

  static const _labels = {
    TaskStatus.pending: 'Pending',
    TaskStatus.acked: 'Acknowledged',
    TaskStatus.done: 'Done',
    TaskStatus.blocked: 'Blocked',
  };

  @override
  Widget build(BuildContext context) {
    if (tasks.isEmpty) return const SizedBox.shrink();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
          child: Text(_labels[status]!,
              style: Theme.of(context).textTheme.titleSmall),
        ),
        for (final t in tasks)
          ListTile(
            leading: Icon(
              switch (status) {
                TaskStatus.pending => Icons.radio_button_unchecked,
                TaskStatus.acked => Icons.check_circle_outline,
                TaskStatus.done => Icons.check_circle,
                TaskStatus.blocked => Icons.block,
              },
              color: status == TaskStatus.blocked
                  ? Theme.of(context).colorScheme.error
                  : null,
            ),
            title: Text(t.title),
            subtitle: t.critical ? const Text('Critical') : null,
            trailing: switch (status) {
              TaskStatus.pending => FilledButton(
                  onPressed: () => onAck(t),
                  child: const Text('Ack')),
              TaskStatus.acked => OutlinedButton(
                  onPressed: () => onDone(t),
                  child: const Text('Done')),
              _ => null,
            },
          ),
      ],
    );
  }
}

class _EvidenceHashDialog extends StatefulWidget {
  const _EvidenceHashDialog({required this.taskTitle});

  final String taskTitle;

  @override
  State<_EvidenceHashDialog> createState() => _EvidenceHashDialogState();
}

class _EvidenceHashDialogState extends State<_EvidenceHashDialog> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Evidence hash required'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(widget.taskTitle),
          const SizedBox(height: 12),
          TextField(
            controller: _controller,
            autofocus: true,
            decoration: const InputDecoration(
              labelText: 'SHA-256 evidence hash',
              hintText: '64 hex characters',
            ),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: () =>
              Navigator.of(context).pop(_controller.text.trim()),
          child: const Text('Mark done'),
        ),
      ],
    );
  }
}
