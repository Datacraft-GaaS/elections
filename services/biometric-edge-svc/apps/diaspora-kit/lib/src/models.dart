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

/// Domain models for the Diaspora Kit app (WP-5.8, M24).

enum ImportProfile {
  nairobi('nairobi', 'Nairobi', 'Africa/Nairobi'),
  restOfWorld('rest-of-world', 'Rest of World', 'UTC'),
  dubai('dubai', 'Dubai', 'Asia/Dubai'),
  london('london', 'London', 'Europe/London'),
  washington('washington', 'Washington D.C.', 'America/New_York');

  const ImportProfile(this.code, this.label, this.timezone);

  final String code;
  final String label;
  final String timezone;
}

class EmbassyStation {
  const EmbassyStation({
    required this.id,
    required this.countryIso,
    required this.city,
    required this.timezone,
    required this.stationCode,
    required this.isOpen,
  });

  final String id;
  final String countryIso;
  final String city;
  final String timezone;
  final String stationCode;
  final bool isOpen;

  @override
  String toString() => '$stationCode - $city ($countryIso)';
}

class Form34AEntry {
  Form34AEntry({
    required this.stationId,
    required this.formId,
    required this.results,
    required this.rejected,
    required this.disputed,
    this.hsmSignature,
    this.queuedAt,
    this.deferredTransmissionAt,
  });

  final String stationId;
  final String formId;
  final Map<String, int> results;
  final int rejected;
  final int disputed;
  String? hsmSignature;
  DateTime? queuedAt;
  DateTime? deferredTransmissionAt;
}

class PouchRecord {
  PouchRecord({
    required this.id,
    required this.formIds,
    this.sealId,
    this.sealedAt,
    this.transfers = const [],
  });

  final String id;
  final List<String> formIds;
  String? sealId;
  DateTime? sealedAt;
  final List<CustodyTransfer> transfers;
}

class CustodyTransfer {
  const CustodyTransfer({
    required this.fromPerson,
    required this.toPerson,
    required this.location,
    required this.transferredAt,
  });

  final String fromPerson;
  final String toPerson;
  final String location;
  final DateTime transferredAt;
}
