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

/// M3 feature-screen gate — wraps any feature widget with hardware-availability banner.
/// Per plan.md M3: feature screens must show "hardware required" message when devices missing.
library;

import 'package:flutter/material.dart';
import '../session/m3_session.dart';

class M3FeatureGate extends StatelessWidget {
  final M3SessionState session;
  final Widget child;
  const M3FeatureGate({super.key, required this.session, required this.child});

  @override
  Widget build(BuildContext context) {
    if (session.isReady) return child;
    return Column(
      children: [
        Container(
          width: double.infinity,
          color: Theme.of(context).colorScheme.errorContainer,
          padding: const EdgeInsets.all(12),
          child: Text(
            session.errorMessage ?? 'Hardware required (M3)',
            style: TextStyle(color: Theme.of(context).colorScheme.onErrorContainer),
          ),
        ),
        const Expanded(child: Center(child: Text('Feature disabled — hardware not procured.'))),
      ],
    );
  }
}
