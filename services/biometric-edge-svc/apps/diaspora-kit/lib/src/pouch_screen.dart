/// Pouch management screen: seal, transfer log, receive at Nairobi.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'models.dart';
import 'providers.dart';

class PouchScreen extends ConsumerWidget {
  const PouchScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final pouches = ref.watch(pouchesProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Pouch Custody')),
      body: pouches.isEmpty
          ? const Center(child: Text('No pouches created.'))
          : ListView.builder(
              itemCount: pouches.length,
              itemBuilder: (context, i) {
                final p = pouches[i];
                return Card(
                  margin: const EdgeInsets.all(8),
                  child: ExpansionTile(
                    title: Text('Pouch ${p.id.substring(0, 8)}'),
                    subtitle: Text(
                      p.sealId != null
                          ? 'Sealed: ${p.sealId}'
                          : 'Not sealed',
                    ),
                    children: [
                      Padding(
                        padding: const EdgeInsets.all(8),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text('Forms: ${p.formIds.length}'),
                            const SizedBox(height: 4),
                            if (p.transfers.isEmpty)
                              const Text('No transfers recorded.')
                            else
                              for (final t in p.transfers)
                                ListTile(
                                  dense: true,
                                  leading: const Icon(Icons.swap_horiz),
                                  title: Text('${t.fromPerson} to ${t.toPerson}'),
                                  subtitle: Text(t.location),
                                ),
                            const SizedBox(height: 8),
                            FilledButton(
                              child: const Text('Add Transfer'),
                              onPressed: () =>
                                  _showTransferDialog(context, ref, p.id),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                );
              },
            ),
      floatingActionButton: FloatingActionButton.extended(
        key: const Key('new-pouch-btn'),
        icon: const Icon(Icons.add),
        label: const Text('New Pouch'),
        onPressed: () => _createPouch(ref),
      ),
    );
  }

  void _createPouch(WidgetRef ref) {
    final pouch = PouchRecord(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      formIds: const [],
    );
    ref.read(pouchesProvider.notifier).add(pouch);
  }

  Future<void> _showTransferDialog(
    BuildContext context,
    WidgetRef ref,
    String pouchId,
  ) async {
    final fromCtrl = TextEditingController();
    final toCtrl = TextEditingController();
    final locCtrl = TextEditingController();
    await showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Record Transfer'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: fromCtrl,
              decoration: const InputDecoration(labelText: 'From'),
            ),
            TextField(
              controller: toCtrl,
              decoration: const InputDecoration(labelText: 'To'),
            ),
            TextField(
              controller: locCtrl,
              decoration: const InputDecoration(labelText: 'Location'),
            ),
          ],
        ),
        actions: [
          TextButton(
            child: const Text('Cancel'),
            onPressed: () => Navigator.pop(ctx),
          ),
          FilledButton(
            child: const Text('Save'),
            onPressed: () {
              ref.read(pouchesProvider.notifier).addTransfer(
                    pouchId,
                    CustodyTransfer(
                      fromPerson: fromCtrl.text,
                      toPerson: toCtrl.text,
                      location: locCtrl.text,
                      transferredAt: DateTime.now().toUtc(),
                    ),
                  );
              Navigator.pop(ctx);
            },
          ),
        ],
      ),
    );
  }
}
