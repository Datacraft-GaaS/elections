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

import '../theme.dart';

/// The mobile counterpart of the web portal's ink stamp: a sealed, witnessed
/// confirmation. Stamp-red, double-ruled, mono, ever so slightly askew.
class SealStamp extends StatelessWidget {
  const SealStamp({super.key, required this.label, this.detail});

  final String label;
  final String? detail;

  @override
  Widget build(BuildContext context) {
    return Transform.rotate(
      angle: -0.04,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 22, vertical: 14),
        decoration: BoxDecoration(
          color: EelColors.raised,
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: EelColors.seal, width: 3),
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              label.toUpperCase(),
              textAlign: TextAlign.center,
              style: EelTypography.mono.copyWith(
                color: EelColors.seal,
                fontWeight: FontWeight.w700,
                letterSpacing: 0.1,
                fontSize: 15,
              ),
            ),
            if (detail != null) ...[
              const SizedBox(height: 4),
              Text(
                detail!,
                textAlign: TextAlign.center,
                style: EelTypography.mono.copyWith(
                  color: EelColors.sealDeep,
                  fontSize: 12,
                  letterSpacing: 0.06,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
