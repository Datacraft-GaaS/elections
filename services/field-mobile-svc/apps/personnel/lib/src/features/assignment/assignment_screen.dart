import 'package:flutter/material.dart';

import '../../models/worker_assignment.dart';
import '../../services/dispatch_api.dart';
import '../../session.dart';
import '../../theme.dart';

/// Role code → human-readable title.
String roleTitle(String code) => switch (code.toLowerCase()) {
      'po' => 'Presiding Officer',
      'dpo' => 'Deputy Presiding Officer',
      'clerk' => 'Polling Clerk',
      final other => other.toUpperCase(),
    };

class AssignmentScreen extends StatefulWidget {
  const AssignmentScreen({super.key, this.api});

  final DispatchApi? api;

  @override
  State<AssignmentScreen> createState() => _AssignmentScreenState();
}

class _AssignmentScreenState extends State<AssignmentScreen> {
  List<WorkerAssignment>? _assignments;
  bool _loading = true;
  bool _error = false;

  DispatchApi get _api => widget.api ?? DispatchApi();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _load());
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = false;
    });
    try {
      final assignments = await _api.getAssignments(kWorkerId);
      if (mounted) setState(() { _assignments = assignments; _loading = false; });
    } catch (_) {
      if (mounted) setState(() { _loading = false; _error = true; });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('My Assignment')),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_error) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.cloud_off, size: 48, color: EelColors.inkFaint),
              const SizedBox(height: 16),
              Text('Could not load your assignment', style: Theme.of(context).textTheme.titleLarge),
              const SizedBox(height: 8),
              Text('Check your connection and try again.', style: Theme.of(context).textTheme.bodyMedium),
              const SizedBox(height: 20),
              OutlinedButton.icon(
                onPressed: _load,
                icon: const Icon(Icons.refresh),
                label: const Text('Retry'),
              ),
            ],
          ),
        ),
      );
    }
    final assignments = _assignments ?? const <WorkerAssignment>[];
    if (assignments.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.assignment_outlined, size: 48, color: EelColors.green),
              const SizedBox(height: 16),
              Text('No assignment yet', style: Theme.of(context).textTheme.titleLarge),
              const SizedBox(height: 8),
              Text(
                'Your posting will appear here once it is issued.',
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
            ],
          ),
        ),
      );
    }

    final first = assignments.first;
    final rest = assignments.skip(1).toList();

    return ListView(
      padding: const EdgeInsets.fromLTRB(20, 12, 20, 32),
      children: [
        _AssignmentHero(assignment: first),
        if (rest.isNotEmpty) ...[
          const SizedBox(height: 20),
          Text('Previous assignments', style: Theme.of(context).textTheme.titleMedium),
        ],
        for (final a in rest) _AssignmentRow(assignment: a),
        const SizedBox(height: 20),
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: EelColors.goldTint,
            borderRadius: BorderRadius.circular(14),
            border: Border.all(color: const Color(0xFFE7D5AB)),
          ),
          child: Row(
            children: [
              const Icon(Icons.info_outline, color: EelColors.gold),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  'Your posting and tasks appear under Duty. Check in on arrival to start your shift.',
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _AssignmentHero extends StatelessWidget {
  const _AssignmentHero({required this.assignment});

  final WorkerAssignment assignment;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: EelColors.greenDeep,
        borderRadius: BorderRadius.circular(16),
        boxShadow: const [
          BoxShadow(color: Color(0x3312231A), blurRadius: 24, offset: Offset(0, 10)),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  roleTitle(assignment.role),
                  style: const TextStyle(
                    fontFamily: EelFonts.display,
                    fontFamilyFallback: EelFonts.displayFallback,
                    fontSize: 24,
                    fontWeight: FontWeight.w600,
                    color: EelColors.paper,
                  ),
                ),
              ),
              _StatusBadge(status: assignment.status),
            ],
          ),
          const SizedBox(height: 4),
          Text(
            assignment.stationCode,
            style: EelTypography.mono.copyWith(color: const Color(0xFFBFD4C6)),
          ),
          const SizedBox(height: 18),
          _HeroRow(icon: Icons.event, label: 'Report time', value: _reportTime(assignment.reportTime)),
          _HeroRow(icon: Icons.badge_outlined, label: 'Election', value: assignment.electionId),
          _HeroRow(icon: Icons.supervisor_account_outlined, label: 'Returning Officer', value: assignment.supervisorId),
        ],
      ),
    );
  }

  String _reportTime(DateTime t) {
    final local = t.toLocal();
    final hh = local.hour.toString().padLeft(2, '0');
    final mm = local.minute.toString().padLeft(2, '0');
    return '${local.day}/${local.month} at $hh:$mm';
  }
}

class _HeroRow extends StatelessWidget {
  const _HeroRow({required this.icon, required this.label, required this.value});

  final IconData icon;
  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(top: 10),
      child: Row(
        children: [
          Icon(icon, color: const Color(0xFFBFD4C6), size: 18),
          const SizedBox(width: 10),
          Text(
            label,
            style: const TextStyle(color: Color(0xFFBFD4C6), fontSize: 13),
          ),
          const Spacer(),
          Flexible(
            child: Text(
              value,
              textAlign: TextAlign.end,
              overflow: TextOverflow.ellipsis,
              style: EelTypography.mono.copyWith(color: EelColors.paper, fontSize: 13),
            ),
          ),
        ],
      ),
    );
  }
}

class _AssignmentRow extends StatelessWidget {
  const _AssignmentRow({required this.assignment});

  final WorkerAssignment assignment;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      decoration: BoxDecoration(
        color: EelColors.raised,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: EelColors.line),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(roleTitle(assignment.role), style: const TextStyle(fontWeight: FontWeight.w600, color: EelColors.ink)),
                Text(assignment.stationCode, style: EelTypography.mono.copyWith(fontSize: 12, color: EelColors.inkFaint)),
              ],
            ),
          ),
          _StatusBadge(status: assignment.status),
        ],
      ),
    );
  }
}

class _StatusBadge extends StatelessWidget {
  const _StatusBadge({required this.status});

  final AssignmentStatus status;

  @override
  Widget build(BuildContext context) {
    final (color, tint, label) = switch (status) {
      AssignmentStatus.pending => (EelColors.inkFaint, EelColors.paper2, 'Pending'),
      AssignmentStatus.accepted => (EelColors.greenDeep, EelColors.greenTint, 'Accepted'),
      AssignmentStatus.declined => (EelColors.sealDeep, EelColors.sealTint, 'Declined'),
      AssignmentStatus.reassigned => (const Color(0xFF7A5718), EelColors.goldTint, 'Reassigned'),
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: tint,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(
        label,
        style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: color),
      ),
    );
  }
}
