import 'package:flutter/material.dart';

import '../../models/worker_check_out.dart';
import '../../services/attendance_signing.dart';
import '../../services/dispatch_api.dart';
import '../../services/location_service.dart';
import '../../session.dart';
import '../../theme.dart';
import '../../widgets/seal_stamp.dart';
import '../../widgets/voice_guide_button.dart';

class CheckOutScreen extends StatefulWidget {
  const CheckOutScreen({super.key, this.api, this.location});

  final DispatchApi? api;
  final LocationProvider? location;

  @override
  State<CheckOutScreen> createState() => _CheckOutScreenState();
}

class _CheckOutScreenState extends State<CheckOutScreen> {
  bool _busy = false;
  GpsFix? _fix;
  WorkerCheckOut? _result;
  String? _error;

  DispatchApi get _api => widget.api ?? DispatchApi();
  LocationProvider get _location => widget.location ?? const DeviceLocationProvider();

  Future<void> _checkOut() async {
    setState(() {
      _busy = true;
      _error = null;
    });

    final GpsFix fix;
    try {
      fix = await _location.currentFix();
    } on LocationException catch (e) {
      if (mounted) setState(() { _busy = false; _error = e.message; });
      return;
    }

    if (fix.isSpoofed) {
      if (mounted) {
        setState(() {
          _busy = false;
          _error = 'Location could not be verified. Move closer to the station and try again.';
        });
      }
      return;
    }

    setState(() => _fix = fix);

    try {
      final payload = <String, Object?>{
        'workerId': kWorkerId,
        'stationCode': kStationCode,
        'electionId': kElectionId,
        'gpsLat': fix.lat,
        'gpsLon': fix.lon,
      };
      final signature = await signAttendancePayload(payload);
      final result = await _api.checkOut(
        workerId: kWorkerId,
        stationCode: kStationCode,
        electionId: kElectionId,
        gpsLat: fix.lat,
        gpsLon: fix.lon,
        signature: signature,
      );
      if (mounted) setState(() { _busy = false; _result = result; });
    } catch (_) {
      if (mounted) {
        setState(() {
          _busy = false;
          _fix = null;
          _error = 'Could not reach the dispatch server. Try again.';
        });
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Check-out failed — try again.')),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final done = _result != null;
    return Scaffold(
      appBar: AppBar(title: const Text('Check out')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(20, 12, 20, 32),
        children: [
          _HoursCard(hours: _result?.hoursWorked, done: done),
          const SizedBox(height: 20),
          if (_fix != null) _FixLine(fix: _fix!),
          if (_error != null) _ErrorCard(message: _error!),
          const SizedBox(height: 28),
          Center(
            child: FilledButton.icon(
              onPressed: !done && !_busy ? _checkOut : null,
              icon: _busy
                  ? const SizedBox(
                      width: 18,
                      height: 18,
                      child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                    )
                  : Icon(done ? Icons.check : Icons.logout),
              label: Text(done ? 'Checked out' : 'Check out'),
              style: FilledButton.styleFrom(
                backgroundColor: done ? EelColors.gold : EelColors.green,
                padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 18),
              ),
            ),
          ),
          const SizedBox(height: 24),
          if (done)
            Center(
              child: SealStamp(label: 'Checked out', detail: _result!.checkOutAtLabel),
            ),
          if (done) ...[
            const SizedBox(height: 20),
            _PayrollCard(result: _result!),
          ],
          const SizedBox(height: 20),
          Center(
            child: VoiceGuideButton(
              label: 'Check out',
              voiceText: 'Tap to check out and end your shift.',
              onPressed: !done && !_busy ? _checkOut : () {},
            ),
          ),
        ],
      ),
    );
  }
}

String formatHours(double hours) {
  final h = hours.floor();
  final m = ((hours - h) * 60).round();
  return '${h}h ${m.toString().padLeft(2, '0')}m';
}

class _HoursCard extends StatelessWidget {
  const _HoursCard({required this.hours, required this.done});

  final double? hours;
  final bool done;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(18),
      decoration: BoxDecoration(
        color: EelColors.raised,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: EelColors.line),
      ),
      child: Row(
        children: [
          Container(
            width: 44,
            height: 44,
            decoration: BoxDecoration(
              color: done ? EelColors.goldTint : EelColors.paper2,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Icon(Icons.schedule, color: done ? EelColors.gold : EelColors.inkFaint),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Shift hours', style: Theme.of(context).textTheme.bodySmall),
                Text(
                  hours != null ? formatHours(hours!) : '—',
                  style: EelTypography.mono.copyWith(
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                    color: EelColors.ink,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _FixLine extends StatelessWidget {
  const _FixLine({required this.fix});

  final GpsFix fix;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(top: 12),
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
              'S ${fix.lat.toStringAsFixed(4)} · E ${fix.lon.toStringAsFixed(4)}',
              style: EelTypography.mono.copyWith(fontSize: 13, color: EelColors.inkSoft),
            ),
          ),
        ],
      ),
    );
  }
}

class _PayrollCard extends StatelessWidget {
  const _PayrollCard({required this.result});

  final WorkerCheckOut result;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: EelColors.goldTint,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: const Color(0xFFE7D5AB)),
      ),
      child: Row(
        children: [
          const Icon(Icons.payments_outlined, color: EelColors.gold, size: 22),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Payroll logged',
                  style: TextStyle(fontWeight: FontWeight.w600, color: EelColors.ink),
                ),
                const SizedBox(height: 2),
                Text(
                  '${formatHours(result.hoursWorked)} submitted for payment',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _ErrorCard extends StatelessWidget {
  const _ErrorCard({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(top: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: EelColors.sealTint,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: const Color(0xFFF0C9C4)),
      ),
      child: Row(
        children: [
          const Icon(Icons.gpp_bad_outlined, color: EelColors.sealDeep),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              message,
              style: const TextStyle(color: EelColors.sealDeep, fontWeight: FontWeight.w600),
            ),
          ),
        ],
      ),
    );
  }
}

extension on WorkerCheckOut {
  String get checkOutAtLabel {
    final t = checkOutAt.toLocal();
    final hh = t.hour.toString().padLeft(2, '0');
    final mm = t.minute.toString().padLeft(2, '0');
    return '$hh:$mm';
  }
}
