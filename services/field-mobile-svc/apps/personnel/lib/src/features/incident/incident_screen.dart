import 'package:flutter/material.dart';

import '../../services/location_service.dart';
import '../../theme.dart';
import '../../widgets/seal_stamp.dart';

class IncidentScreen extends StatefulWidget {
  const IncidentScreen({super.key, this.location});

  final LocationProvider? location;

  @override
  State<IncidentScreen> createState() => _IncidentScreenState();
}

class _IncidentScreenState extends State<IncidentScreen> {
  final _controller = TextEditingController();
  String _type = 'Queue dispute';
  bool _logged = false;
  GpsFix? _fix;

  static const _types = [
    'Queue dispute',
    'Equipment fault',
    'Security',
    'Medical',
    'Other',
  ];

  LocationProvider get _location => widget.location ?? const SimulatedLocationProvider();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    final fix = await _location.currentFix();
    setState(() {
      _fix = fix;
      _logged = true;
    });
  }

  @override
  Widget build(BuildContext context) {
    if (_logged) {
      return Scaffold(
        appBar: AppBar(title: const Text('Incident Report')),
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.verified_outlined, size: 56, color: EelColors.green),
                const SizedBox(height: 20),
                Text('Incident logged', style: Theme.of(context).textTheme.headlineMedium),
                const SizedBox(height: 8),
                Text(
                  'The command centre has been notified and will follow up.',
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
                const SizedBox(height: 24),
                SealStamp(label: 'Logged', detail: _fix?.clockLabel),
                const SizedBox(height: 24),
                OutlinedButton(
                  onPressed: () => setState(() {
                    _logged = false;
                    _controller.clear();
                  }),
                  child: const Text('Report another'),
                ),
              ],
            ),
          ),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(title: const Text('Incident Report')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(20, 12, 20, 32),
        children: [
          Text(
            'What happened?',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 4),
          Text(
            'Details are sent to the command centre. Include what you saw.',
            style: Theme.of(context).textTheme.bodyMedium,
          ),
          const SizedBox(height: 20),
          _TypeSelector(selected: _type, onChanged: (t) => setState(() => _type = t)),
          const SizedBox(height: 20),
          TextField(
            controller: _controller,
            maxLines: 4,
            textCapitalization: TextCapitalization.sentences,
            decoration: const InputDecoration(
              labelText: 'Description',
              hintText: 'e.g. Ballot box seal was already broken on arrival…',
              alignLabelWithHint: true,
            ),
          ),
          const SizedBox(height: 16),
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: EelColors.paper2,
              borderRadius: BorderRadius.circular(14),
            ),
            child: Row(
              children: [
                const Icon(Icons.gps_fixed, color: EelColors.green, size: 20),
                const SizedBox(width: 12),
                Expanded(
                  child: Text(
                    'Your location and a photo are attached automatically.',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 24),
          FilledButton.icon(
            onPressed: _controller.text.trim().isEmpty ? null : _submit,
            icon: const Icon(Icons.send_outlined),
            label: const Text('Submit report'),
          ),
        ],
      ),
    );
  }
}

class _TypeSelector extends StatelessWidget {
  const _TypeSelector({required this.selected, required this.onChanged});

  final String selected;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        for (final type in _IncidentScreenState._types)
          ChoiceChip(
            label: Text(type),
            selected: type == selected,
            onSelected: (_) => onChanged(type),
            selectedColor: EelColors.greenTint,
            backgroundColor: EelColors.raised,
            side: BorderSide(color: type == selected ? EelColors.green : EelColors.line),
            labelStyle: TextStyle(
              fontWeight: FontWeight.w600,
              color: type == selected ? EelColors.greenDeep : EelColors.ink,
            ),
            checkmarkColor: EelColors.green,
          ),
      ],
    );
  }
}
