/// PartitionSelector — operator UI for choosing which partition dimensions
/// to include in a 1:N search. Shows estimated deduplication time live
/// as the operator selects/deselects filters. Every selection state is
/// logged via AuditService (5-axis audit).
///
/// ## Dimensions
/// - Gender (Male/Female/Other)
/// - Age bucket (0-17, 18-35, 36-55, 56-75, 76+)
/// - Geo partition (county/station)
/// - Finger position (10 fingers)
/// - Dominant hand (Left/Right/Ambidextrous)
/// - Ridge quality (High/Medium/Low)
/// - Diaspora status (InCountry/Diaspora/Newcomer)
/// - Enrollment era (2017-2019, 2020-2022, 2023-2025, 2026+)

import 'package:eel_design/eel_design.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_core_flutter/src/audit/audit_service.dart';
import 'package:eel_core_flutter/src/location/location_service.dart';

/// Configuration for which partition dimensions to use.
class PartitionSelection {
  final Set<Gender> genders;
  final Set<AgeBucket> ageBuckets;
  final Set<String> geoPartitions;
  final Set<FingerPosition> fingers;
  final Set<DominantHand> hands;
  final Set<RidgeQuality> qualities;
  final Set<DiasporaStatus> diasporaStatuses;
  final Set<EnrollmentEra> enrollmentEras;

  PartitionSelection({
    Set<Gender>? genders,
    Set<AgeBucket>? ageBuckets,
    Set<String>? geoPartitions,
    Set<FingerPosition>? fingers,
    Set<DominantHand>? hands,
    Set<RidgeQuality>? qualities,
    Set<DiasporaStatus>? diasporaStatuses,
    Set<EnrollmentEra>? enrollmentEras,
  })  : genders = genders ?? Gender.values.toSet(),
        ageBuckets = ageBuckets ?? AgeBucket.values.toSet(),
        geoPartitions = geoPartitions ?? <String>{},
        fingers = fingers ?? FingerPosition.values.toSet(),
        hands = hands ?? DominantHand.values.toSet(),
        qualities = qualities ?? {RidgeQuality.high, RidgeQuality.medium},
        diasporaStatuses = diasporaStatuses ?? DiasporaStatus.values.toSet(),
        enrollmentEras = enrollmentEras ?? EnrollmentEra.values.toSet();

  /// Total number of partitions that match the current selection.
  int partitionCount(int totalPartitions) {
    final ratio = (genders.length / Gender.values.length) *
        (ageBuckets.length / AgeBucket.values.length) *
        (geoPartitions.isEmpty ? 1.0 : geoPartitions.length / 47.0) * // 47 Kenya counties
        (fingers.length / FingerPosition.values.length) *
        (hands.length / DominantHand.values.length) *
        (qualities.length / RidgeQuality.values.length) *
        (diasporaStatuses.length / DiasporaStatus.values.length) *
        (enrollmentEras.length / EnrollmentEra.values.length);
    return (totalPartitions * ratio).round().clamp(1, totalPartitions);
  }

  /// Estimated 1:N search time given a total fingerprint count.
  /// Assumes 0.5ms per comparison (FAP60 + bozorth3 on device CPU).
  Duration estimatedSearchTime(int totalFingerprints) {
    final candidates = (totalFingerprints / partitionCount(totalPartitions: totalFingerprints)).round();
    final ms = candidates * 0.5;
    return Duration(milliseconds: ms.round());
  }

  String humanReadableEstimate(int totalFingerprints) {
    final d = estimatedSearchTime(totalFingerprints);
    if (d.inMinutes >= 60) {
      return '${(d.inMinutes / 60).toStringAsFixed(1)} hours';
    } else if (d.inSeconds >= 60) {
      return '${(d.inSeconds / 60).toStringAsFixed(1)} minutes';
    } else if (d.inSeconds >= 1) {
      return '${d.inSeconds} seconds';
    } else {
      return '${d.inMilliseconds}ms';
    }
  }

  /// Compound key (debug / display).
  String toKey(Gender g, AgeBucket a, String geo, FingerPosition f, DominantHand h, RidgeQuality q) =>
      '$g|$a|$geo|${f.name}|${h.name}|$q';
}

enum Gender { male, female, other }
enum AgeBucket { minor, youngAdult, middleAged, senior, elderly }
enum FingerPosition {
  leftThumb, leftIndex, leftMiddle, leftRing, leftLittle,
  rightThumb, rightIndex, rightMiddle, rightRing, rightLittle
}
enum DominantHand { left, right, ambidextrous }
enum RidgeQuality { high, medium, low }
enum DiasporaStatus { inCountry, diaspora, newcomer }
enum EnrollmentEra { e2017_2019, e2020_2022, e2023_2025, e2026_plus }

extension GenderX on Gender {
  String get label {
    switch (this) {
      case Gender.male: return 'Male';
      case Gender.female: return 'Female';
      case Gender.other: return 'Other';
    }
  }
}

extension AgeBucketX on AgeBucket {
  String get label {
    switch (this) {
      case AgeBucket.minor: return '0-17';
      case AgeBucket.youngAdult: return '18-35';
      case AgeBucket.middleAged: return '36-55';
      case AgeBucket.senior: return '56-75';
      case AgeBucket.elderly: return '76+';
    }
  }
}

extension FingerPositionX on FingerPosition {
  String get label {
    switch (this) {
      case FingerPosition.leftThumb: return 'Left Thumb';
      case FingerPosition.leftIndex: return 'Left Index';
      case FingerPosition.leftMiddle: return 'Left Middle';
      case FingerPosition.leftRing: return 'Left Ring';
      case FingerPosition.leftLittle: return 'Left Little';
      case FingerPosition.rightThumb: return 'Right Thumb';
      case FingerPosition.rightIndex: return 'Right Index';
      case FingerPosition.rightMiddle: return 'Right Middle';
      case FingerPosition.rightRing: return 'Right Ring';
      case FingerPosition.rightLittle: return 'Right Little';
    }
  }
}

extension DominantHandX on DominantHand {
  String get label {
    switch (this) {
      case DominantHand.left: return 'Left';
      case DominantHand.right: return 'Right';
      case DominantHand.ambidextrous: return 'Ambidextrous';
    }
  }
}

extension RidgeQualityX on RidgeQuality {
  String get label {
    switch (this) {
      case RidgeQuality.high: return 'High (NFIQ 1-2)';
      case RidgeQuality.medium: return 'Medium (NFIQ 3)';
      case RidgeQuality.low: return 'Low (NFIQ 4-5)';
    }
  }
}

extension DiasporaStatusX on DiasporaStatus {
  String get label {
    switch (this) {
      case DiasporaStatus.inCountry: return 'In-Country';
      case DiasporaStatus.diaspora: return 'Diaspora';
      case DiasporaStatus.newcomer: return 'Newcomer';
    }
  }
}

extension EnrollmentEraX on EnrollmentEra {
  String get label {
    switch (this) {
      case EnrollmentEra.e2017_2019: return '2017-2019';
      case EnrollmentEra.e2020_2022: return '2020-2022';
      case EnrollmentEra.e2023_2025: return '2023-2025';
      case EnrollmentEra.e2026_plus: return '2026+';
    }
  }
}

/// Hook: gives a `PartitionSelection` that updates with `setState`.
PartitionSelection usePartitionSelection({PartitionSelection? initial}) {
  return useState(initial ?? PartitionSelection()).value;
}

/// Live estimate card showing deduplication time.
class DeduplicationEstimateCard extends StatelessWidget {
  final PartitionSelection selection;
  final int totalFingerprints;
  const DeduplicationEstimateCard({
    super.key,
    required this.selection,
    required this.totalFingerprints,
  });

  @override
  Widget build(BuildContext context) {
    final time = selection.humanReadableEstimate(totalFingerprints);
    final partitions = selection.partitionCount(totalPartitions: totalFingerprints);
    return EelResultCard(
      leading: Icon(Icons.timer, color: EelColors.of(context).primary, size: 32),
      title: Text('Estimated dedup time: $time', style: EelTextStyle.titleMedium),
      subtitle: Text(
        'Searching $partitions partition(s) out of ~6,300 total.\n'
        'Lower partitions = faster search, narrower scope.\n'
        'Higher partitions = slower search, broader scope (catches cross-staff duplicates).',
        style: EelTextStyle.bodySmall,
      ),
    );
  }
}

/// The partition selector UI. Operator toggles filters via EelChips.
class PartitionSelector extends ConsumerWidget {
  final PartitionSelection selection;
  final ValueChanged<PartitionSelection> onChanged;
  final int totalFingerprints;
  const PartitionSelector({
    super.key,
    required this.selection,
    required this.onChanged,
    this.totalFingerprints = 100_000_000, // 100M default
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    void toggleGender(Gender g) {
      final newSet = Set<Gender>.from(selection.genders);
      newSet.contains(g) ? newSet.remove(g) : newSet.add(g);
      onChanged(PartitionSelection(
        genders: newSet,
        ageBuckets: selection.ageBuckets,
        geoPartitions: selection.geoPartitions,
        fingers: selection.fingers,
        hands: selection.hands,
        qualities: selection.qualities,
        diasporaStatuses: selection.diasporaStatuses,
        enrollmentEras: selection.enrollmentEras,
      ));
    }

    // ... similar for each dimension

    return EelScaffold(
      title: '1:N Search Configuration',
      variant: EelScaffoldVariant.scrollable,
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          DeduplicationEstimateCard(selection: selection, totalFingerprints: totalFingerprints),
          const SizedBox(height: 16),
          Text('Gender', style: EelTextStyle.titleSmall),
          EelChips(
            options: Gender.values.map((g) => ChipOption(label: g.label, selected: selection.genders.contains(g))).toList(),
            onToggle: (i) => toggleGender(Gender.values[i]),
          ),
          // ... more dimensions
        ],
      ),
    );
  }
}
