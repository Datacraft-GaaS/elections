// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// TZ display widget showing local station time and Nairobi time side-by-side.
///
/// Uses the `timezone` package for DST-safe conversions.

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:timezone/timezone.dart' as tz;
import 'package:timezone/data/latest.dart' as tz_data;

/// Call once at app startup to initialise timezone data.
void initTimezones() {
  tz_data.initializeTimeZones();
}

/// Formats a [DateTime] in the given IANA [tzName].
String formatInTz(DateTime utc, String tzName) {
  final location = tz.getLocation(tzName);
  final local = tz.TZDateTime.from(utc, location);
  final h = local.hour.toString().padLeft(2, '0');
  final m = local.minute.toString().padLeft(2, '0');
  final s = local.second.toString().padLeft(2, '0');
  final offset = local.timeZoneOffset;
  final sign = offset.isNegative ? '-' : '+';
  final oh = offset.inHours.abs().toString().padLeft(2, '0');
  final om = (offset.inMinutes.abs() % 60).toString().padLeft(2, '0');
  return '$h:$m:$s (UTC$sign$oh:$om)';
}

/// Side-by-side clock widget: station local time vs. Nairobi time.
class TzDisplayWidget extends StatefulWidget {
  const TzDisplayWidget({
    super.key,
    required this.stationTimezone,
    this.nairobiTimezone = 'Africa/Nairobi',
  });

  final String stationTimezone;
  final String nairobiTimezone;

  @override
  State<TzDisplayWidget> createState() => _TzDisplayWidgetState();
}

class _TzDisplayWidgetState extends State<TzDisplayWidget> {
  late Timer _timer;
  DateTime _now = DateTime.now().toUtc();

  @override
  void initState() {
    super.initState();
    _timer = Timer.periodic(const Duration(seconds: 1), (_) {
      if (mounted) setState(() => _now = DateTime.now().toUtc());
    });
  }

  @override
  void dispose() {
    _timer.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final stationTime = formatInTz(_now, widget.stationTimezone);
    final nairobiTime = formatInTz(_now, widget.nairobiTimezone);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          children: [
            _TimeColumn(
              label: 'Station time',
              value: stationTime,
              sublabel: widget.stationTimezone,
            ),
            const VerticalDivider(),
            _TimeColumn(
              label: 'Nairobi time',
              value: nairobiTime,
              sublabel: widget.nairobiTimezone,
            ),
          ],
        ),
      ),
    );
  }
}

class _TimeColumn extends StatelessWidget {
  const _TimeColumn({
    required this.label,
    required this.value,
    required this.sublabel,
  });

  final String label;
  final String value;
  final String sublabel;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(label, style: Theme.of(context).textTheme.labelSmall),
        const SizedBox(height: 4),
        Text(value, style: Theme.of(context).textTheme.bodyMedium),
        Text(sublabel, style: Theme.of(context).textTheme.labelSmall),
      ],
    );
  }
}
