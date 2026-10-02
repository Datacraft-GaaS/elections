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

class ShiftReport {
  final String id;
  final String engineerId;
  final String clusterId;
  final DateTime shiftStart;
  final DateTime shiftEnd;
  final int ticketsHandled;
  final double slaCompliancePct;
  final int escalations;
  final int sparesConsumed;
  final String engineerSignature;

  const ShiftReport({
    required this.id,
    required this.engineerId,
    required this.clusterId,
    required this.shiftStart,
    required this.shiftEnd,
    required this.ticketsHandled,
    required this.slaCompliancePct,
    required this.escalations,
    required this.sparesConsumed,
    required this.engineerSignature,
  });
}
