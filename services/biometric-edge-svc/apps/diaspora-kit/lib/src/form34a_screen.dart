/// Form 34A capture screen for diaspora embassy stations.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'models.dart';
import 'providers.dart';

// Candidate list used for demonstration; in production fetched from roll.
const _kCandidates = ['Candidate A', 'Candidate B', 'Candidate C'];

class Form34AScreen extends ConsumerStatefulWidget {
  const Form34AScreen({super.key, required this.station});

  final EmbassyStation station;

  @override
  ConsumerState<Form34AScreen> createState() => _Form34AScreenState();
}

class _Form34AScreenState extends ConsumerState<Form34AScreen> {
  final _formKey = GlobalKey<FormState>();
  final Map<String, TextEditingController> _tallies = {
    for (final c in _kCandidates) c: TextEditingController(text: '0'),
  };
  final _rejectedCtrl = TextEditingController(text: '0');
  final _disputedCtrl = TextEditingController(text: '0');

  bool _queued = false;

  @override
  void dispose() {
    for (final c in _tallies.values) {
      c.dispose();
    }
    _rejectedCtrl.dispose();
    _disputedCtrl.dispose();
    super.dispose();
  }

  void _submit() {
    if (!_formKey.currentState!.validate()) return;

    final results = <String, int>{
      for (final entry in _tallies.entries)
        entry.key: int.parse(entry.value.text),
    };
    final formId = 'FORM-${DateTime.now().millisecondsSinceEpoch}';

    final form = Form34AEntry(
      stationId: widget.station.id,
      formId: formId,
      results: results,
      rejected: int.parse(_rejectedCtrl.text),
      disputed: int.parse(_disputedCtrl.text),
      hsmSignature: 'stub-kit-signature-${widget.station.stationCode}',
      queuedAt: DateTime.now().toUtc(),
      deferredTransmissionAt:
          DateTime.now().toUtc().add(const Duration(hours: 2)),
    );

    ref.read(offlineFormQueueProvider.notifier).enqueue(form);

    setState(() => _queued = true);

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('Form $formId queued for transmission'),
        backgroundColor: Colors.green,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Form 34A - ${widget.station.stationCode}'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Form(
          key: _formKey,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Candidate Tallies',
                  style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 8),
              for (final candidate in _kCandidates)
                Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: TextFormField(
                    key: Key('tally-$candidate'),
                    controller: _tallies[candidate],
                    decoration: InputDecoration(labelText: candidate),
                    keyboardType: TextInputType.number,
                    validator: (v) {
                      if (v == null || int.tryParse(v) == null) {
                        return 'Enter a valid number';
                      }
                      return null;
                    },
                  ),
                ),
              const SizedBox(height: 8),
              TextFormField(
                key: const Key('rejected-field'),
                controller: _rejectedCtrl,
                decoration: const InputDecoration(labelText: 'Rejected ballots'),
                keyboardType: TextInputType.number,
                validator: (v) =>
                    v != null && int.tryParse(v) != null ? null : 'Required',
              ),
              const SizedBox(height: 8),
              TextFormField(
                key: const Key('disputed-field'),
                controller: _disputedCtrl,
                decoration: const InputDecoration(labelText: 'Disputed ballots'),
                keyboardType: TextInputType.number,
                validator: (v) =>
                    v != null && int.tryParse(v) != null ? null : 'Required',
              ),
              const SizedBox(height: 24),
              if (_queued)
                const Text(
                  'Form queued for deferred transmission.',
                  style: TextStyle(color: Colors.green),
                )
              else
                FilledButton.icon(
                  key: const Key('submit-form34a'),
                  icon: const Icon(Icons.send),
                  label: const Text('Sign and Queue'),
                  onPressed: _submit,
                ),
            ],
          ),
        ),
      ),
    );
  }
}
