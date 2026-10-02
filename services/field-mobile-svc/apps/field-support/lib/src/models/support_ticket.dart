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

enum TicketSeverity { sev1, sev2, sev3 }
enum TicketStatus { open, inProgress, escalated, resolved, closed }

class SupportTicket {
  final String id;
  final String stationCode;
  final String kitSerial;
  final String reporterRole;
  final TicketSeverity severity;
  final String title;
  final String description;
  final TicketStatus status;
  final String? assignedEngineerId;
  final DateTime slaTargetAt;
  final DateTime? resolvedAt;
  final int escalationLevel;

  const SupportTicket({
    required this.id,
    required this.stationCode,
    required this.kitSerial,
    required this.reporterRole,
    required this.severity,
    required this.title,
    required this.description,
    required this.status,
    this.assignedEngineerId,
    required this.slaTargetAt,
    this.resolvedAt,
    this.escalationLevel = 0,
  });
}
