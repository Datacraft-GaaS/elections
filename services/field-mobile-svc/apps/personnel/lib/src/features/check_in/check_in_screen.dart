import 'package:flutter/material.dart';

import '../../services/attendance_signing.dart';
import '../../services/dispatch_api.dart';
import '../../services/duress_guard.dart';
import '../../services/location_service.dart';
import '../../session.dart';
import '../../theme.dart';
import '../../widgets/seal_stamp.dart';
import '../../widgets/voice_guide_button.dart';

class CheckInScreen extends StatefulWidget {
  const CheckInScreen({super.key, this.api, this.location, this.duress});

  final DispatchApi? api;
  final LocationProvider? location;
  final DuressGuard? duress;

  @override
  State<CheckInScreen> createState() => _CheckInScreenState();
}

class _CheckInScreenState extends State<CheckInScreen> {
  bool _busy = false;
  GpsFix? _fix;
  String? _error;
  final _pinController = TextEditingController();

  DispatchApi get _api => widget.api ?? DispatchApi();
  LocationProvider get _location => widget.location ?? const DeviceLocationProvider();
  DuressGuard get _duress => widget.duress ?? DuressGuard();

  @override
  void dispose() {
    _pinController.dispose();
    super.dispose();
  }

  Future<void> _checkIn() async {
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

    // Covert duress check: a duress PIN silently raises an alert while the
    // check-in continues normally. The result is intentionally not surfaced.
    final pin = _pinController.text.trim();
    if (pin.isNotEmpty) {
      await _duress.check(pin);
    }

    try {
      // The biometric hash is captured by the BVI kit at the station and
      // injected by the dispatch flow once the personnel app is paired with a
      // kit (see docs/plans/deferral-plan.md section D). Until pairing lands,
      // check-in submits a placeholder hash.
      final payload = <String, Object?>{
        'workerId': kWorkerId,
        'stationCode': kStationCode,
        'electionId': kElectionId,
        'biometricHash': 'simulated',
        'gpsLat': fix.lat,
        'gpsLon': fix.lon,
        'gpsAccuracyM': fix.accuracyM,
      };
      final signature = await signAttendancePayload(payload);
      await _api.checkIn(
        workerId: kWorkerId,
        stationCode: kStationCode,
        electionId: kElectionId,
        biometricHash: 'simulated',
        gpsLat: fix.lat,
        gpsLon: fix.lon,
        gpsAccuracyM: fix.accuracyM,
        signature: signature,
      );
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Checked in successfully')),
        );
      }
    } catch (_) {
      if (mounted) {
        setState(() {
          _busy = false;
          _fix = null;
          _error = 'Could not reach the dispatch server. Try again.';
        });
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Check-in failed — try again.')),
        );
      }
      return;
    }

    if (mounted) setState(() => _busy = false);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Check in')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(20, 12, 20, 32),
        children: [
          _StationCard(onStation: _fix != null),
          const SizedBox(height: 20),
          if (_fix != null) _FixCard(fix: _fix!),
          if (_error != null) _ErrorCard(message: _error!),
          if (_fix == null)
            Padding(
              padding: const EdgeInsets.fromLTRB(0, 8, 0, 0),
              child: TextField(
                controller: _pinController,
                keyboardType: TextInputType.number,
                obscureText: true,
                maxLength: 6,
                decoration: const InputDecoration(
                  labelText: 'Attendance PIN',
                  hintText: '6 digits',
                  counterText: '',
                ),
              ),
            ),
          const SizedBox(height: 28),
          Center(
            child: SizedBox(
              width: 180,
              height: 180,
              child: _ActionButton(
                busy: _busy,
                done: _fix != null,
                icon: _fix != null ? Icons.check : Icons.pin_drop,
                label: _fix != null ? 'Checked in' : 'Check in',
                onPressed: _fix == null && !_busy ? _checkIn : null,
              ),
            ),
          ),
          const SizedBox(height: 24),
          if (_fix != null)
            Center(
              child: SealStamp(label: 'On station', detail: _fix!.clockLabel),
            ),
          const SizedBox(height: 20),
          Center(
            child: VoiceGuideButton(
              label: 'Check in',
              voiceText: 'Tap to check in and start your shift.',
              onPressed: _fix == null && !_busy ? _checkIn : () {},
            ),
          ),
        ],
      ),
    );
  }
}

class _ActionButton extends StatelessWidget {
  const _ActionButton({
    required this.busy,
    required this.done,
    required this.icon,
    required this.label,
    required this.onPressed,
  });

  final bool busy;
  final bool done;
  final IconData icon;
  final String label;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: done ? EelColors.green : EelColors.greenTint,
        border: Border.all(color: EelColors.green, width: 2),
        boxShadow: const [
          BoxShadow(color: Color(0x2212231A), blurRadius: 24, offset: Offset(0, 10)),
        ],
      ),
      child: Material(
        color: Colors.transparent,
        shape: const CircleBorder(),
        child: InkWell(
          customBorder: const CircleBorder(),
          onTap: onPressed,
          child: Center(
            child: busy
                ? const SizedBox(
                    width: 32,
                    height: 32,
                    child: CircularProgressIndicator(strokeWidth: 3),
                  )
                : Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(icon, size: 48, color: done ? Colors.white : EelColors.green),
                      const SizedBox(height: 6),
                      Text(
                        label,
                        style: TextStyle(
                          fontWeight: FontWeight.w600,
                          fontSize: 16,
                          color: done ? Colors.white : EelColors.greenDeep,
                        ),
                      ),
                    ],
                  ),
          ),
        ),
      ),
    );
  }
}

class _StationCard extends StatelessWidget {
  const _StationCard({required this.onStation});

  final bool onStation;

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
              color: onStation ? EelColors.greenTint : EelColors.paper2,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Icon(
              Icons.location_on_outlined,
              color: onStation ? EelColors.green : EelColors.inkFaint,
            ),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Polling station', style: Theme.of(context).textTheme.bodySmall),
                Text(
                  kStationCode,
                  style: EelTypography.mono.copyWith(
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                    color: EelColors.ink,
                  ),
                ),
              ],
            ),
          ),
          if (onStation)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
              decoration: BoxDecoration(
                color: EelColors.greenTint,
                borderRadius: BorderRadius.circular(999),
              ),
              child: const Text(
                'On station',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                  color: EelColors.greenDeep,
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _FixCard extends StatelessWidget {
  const _FixCard({required this.fix});

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
              'S ${fix.lat.toStringAsFixed(4)} · E ${fix.lon.toStringAsFixed(4)} · ±${fix.accuracyM.toStringAsFixed(0)} m',
              style: EelTypography.mono.copyWith(fontSize: 13, color: EelColors.inkSoft),
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
