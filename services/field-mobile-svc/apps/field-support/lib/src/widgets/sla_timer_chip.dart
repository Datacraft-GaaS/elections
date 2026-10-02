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

import 'package:flutter/material.dart';

class SlaTimerChip extends StatelessWidget {
  final DateTime slaTargetAt;

  const SlaTimerChip({super.key, required this.slaTargetAt});

  @override
  Widget build(BuildContext context) {
    final remaining = slaTargetAt.difference(DateTime.now());
    final isBreached = remaining.isNegative;
    return Chip(
      backgroundColor: isBreached ? Colors.red : Colors.green,
      label: Text(
        isBreached
            ? 'SLA BREACHED'
            : '${remaining.inMinutes}m remaining',
        style: const TextStyle(color: Colors.white, fontSize: 12),
      ),
    );
  }
}
