import 'package:flutter/material.dart';

import '../../models/worker_status_ping.dart';
import '../../services/dispatch_api.dart';
import '../../session.dart';
import '../../theme.dart';

class StatusPingScreen extends StatefulWidget {
  const StatusPingScreen({super.key, this.api});

  final DispatchApi? api;

  @override
  State<StatusPingScreen> createState() => _StatusPingScreenState();
}

class _StatusPingScreenState extends State<StatusPingScreen> {
  WorkerStatus _status = WorkerStatus.onStation;
  DateTime? _lastPing;
  bool _busy = false;

  DispatchApi get _api => widget.api ?? DispatchApi();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _loadStatus());
  }

  Future<void> _loadStatus() async {
    try {
      final ping = await _api.getStatus(kWorkerId);
      if (mounted && ping != null) {
        setState(() {
          _status = ping.status;
          _lastPing = ping.pingedAt;
        });
      }
    } catch (_) {
      // Offline: keep the local selection; the ping will queue on retry.
    }
  }

  Future<void> _ping() async {
    setState(() => _busy = true);
    try {
      final ping = await _api.ping(
        workerId: kWorkerId,
        stationCode: kStationCode,
        status: _status,
      );
      if (mounted) {
        setState(() => _lastPing = ping.pingedAt);
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Status sent')),
        );
      }
    } catch (_) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Could not send status — try again.')),
        );
      }
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Status')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(20, 12, 20, 32),
        children: [
          Text('How are you right now?', style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 4),
          Text(
            'A quick status every 15 minutes keeps the command centre accurate.',
            style: Theme.of(context).textTheme.bodyMedium,
          ),
          const SizedBox(height: 20),
          _StatusSelector(status: _status, onChanged: (s) => setState(() => _status = s)),
          const SizedBox(height: 24),
          FilledButton.icon(
            onPressed: _busy ? null : _ping,
            icon: _busy
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                  )
                : const Icon(Icons.sensors),
            label: const Text('Ping now'),
          ),
          const SizedBox(height: 20),
          _LastPingCard(status: _status, lastPing: _lastPing),
        ],
      ),
    );
  }
}

class _StatusSelector extends StatelessWidget {
  const _StatusSelector({required this.status, required this.onChanged});

  final WorkerStatus status;
  final ValueChanged<WorkerStatus> onChanged;

  static const _items = [
    (WorkerStatus.onStation, Icons.location_on, 'On station'),
    (WorkerStatus.onBreak, Icons.free_breakfast, 'On break'),
    (WorkerStatus.absent, Icons.no_accounts, 'Absent'),
  ];

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        for (final (s, icon, label) in _items)
          Padding(
            padding: const EdgeInsets.only(bottom: 10),
            child: _SelectorTile(
              selected: status == s,
              icon: icon,
              label: label,
              onTap: () => onChanged(s),
            ),
          ),
      ],
    );
  }
}

class _SelectorTile extends StatelessWidget {
  const _SelectorTile({
    required this.selected,
    required this.icon,
    required this.label,
    required this.onTap,
  });

  final bool selected;
  final IconData icon;
  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: selected ? EelColors.greenTint : EelColors.raised,
      borderRadius: BorderRadius.circular(14),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(14),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(14),
            border: Border.all(
              color: selected ? EelColors.green : EelColors.line,
              width: selected ? 2 : 1,
            ),
          ),
          child: Row(
            children: [
              Icon(icon, color: selected ? EelColors.greenDeep : EelColors.inkFaint),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  label,
                  style: TextStyle(
                    fontWeight: FontWeight.w600,
                    color: selected ? EelColors.greenDeep : EelColors.ink,
                  ),
                ),
              ),
              Icon(
                selected ? Icons.radio_button_checked : Icons.radio_button_off,
                color: selected ? EelColors.green : EelColors.lineStrong,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _LastPingCard extends StatelessWidget {
  const _LastPingCard({required this.status, required this.lastPing});

  final WorkerStatus status;
  final DateTime? lastPing;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(18),
      decoration: BoxDecoration(
        color: EelColors.paper2,
        borderRadius: BorderRadius.circular(14),
      ),
      child: Row(
        children: [
          const Icon(Icons.history, color: EelColors.inkFaint),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Last ping', style: Theme.of(context).textTheme.bodySmall),
                Text(
                  lastPing == null ? 'Not yet today' : _time(lastPing!),
                  style: EelTypography.mono.copyWith(
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                    color: EelColors.ink,
                  ),
                ),
              ],
            ),
          ),
          if (lastPing != null)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
              decoration: BoxDecoration(
                color: _tint(status),
                borderRadius: BorderRadius.circular(999),
              ),
              child: Text(
                _label(status),
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                  color: _color(status),
                ),
              ),
            ),
        ],
      ),
    );
  }

  String _time(DateTime t) {
    final local = t.toLocal();
    final hh = local.hour.toString().padLeft(2, '0');
    final mm = local.minute.toString().padLeft(2, '0');
    return '$hh:$mm';
  }

  String _label(WorkerStatus s) => switch (s) {
        WorkerStatus.onStation => 'On station',
        WorkerStatus.onBreak => 'On break',
        WorkerStatus.absent => 'Absent',
      };

  Color _color(WorkerStatus s) => switch (s) {
        WorkerStatus.onStation => EelColors.greenDeep,
        WorkerStatus.onBreak => const Color(0xFF7A5718),
        WorkerStatus.absent => EelColors.sealDeep,
      };

  Color _tint(WorkerStatus s) => switch (s) {
        WorkerStatus.onStation => EelColors.greenTint,
        WorkerStatus.onBreak => EelColors.goldTint,
        WorkerStatus.absent => EelColors.sealTint,
      };
}
