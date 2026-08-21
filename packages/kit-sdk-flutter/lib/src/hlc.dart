/// Hybrid Logical Clock compatible with eel-hlc-rs.
///
/// Wire format: `<ms>.<counter>.<node>` where ms is Unix epoch milliseconds,
/// counter is a 16-bit monotonic counter within the same ms, and node is a
/// 16-bit node identifier.
class Hlc implements Comparable<Hlc> {
  const Hlc({required this.ms, required this.counter, required this.node});

  final int ms;
  final int counter;
  final int node;

  factory Hlc.parse(String s) {
    final parts = s.split('.');
    if (parts.length != 3) throw FormatException('Invalid HLC: $s');
    return Hlc(
      ms: int.parse(parts[0]),
      counter: int.parse(parts[1]),
      node: int.parse(parts[2]),
    );
  }

  @override
  String toString() => '$ms.$counter.$node';

  /// Advance the clock, ensuring monotonic output.
  Hlc tick(int wallMs) {
    if (wallMs > ms) {
      return Hlc(ms: wallMs, counter: 0, node: node);
    }
    return Hlc(ms: ms, counter: counter + 1, node: node);
  }

  /// Merge two HLC values, taking the maximum and advancing counter if tied.
  Hlc merge(Hlc other, int wallMs) {
    final maxMs =
        [ms, other.ms, wallMs].reduce((a, b) => a > b ? a : b);
    if (maxMs == ms && maxMs == other.ms) {
      final maxC =
          [counter, other.counter].reduce((a, b) => a > b ? a : b);
      return Hlc(ms: maxMs, counter: maxC + 1, node: node);
    } else if (maxMs == ms) {
      return Hlc(ms: maxMs, counter: counter + 1, node: node);
    } else if (maxMs == other.ms) {
      return Hlc(ms: maxMs, counter: other.counter + 1, node: node);
    } else {
      return Hlc(ms: maxMs, counter: 0, node: node);
    }
  }

  @override
  int compareTo(Hlc other) {
    final msCmp = ms.compareTo(other.ms);
    if (msCmp != 0) return msCmp;
    final cCmp = counter.compareTo(other.counter);
    if (cCmp != 0) return cCmp;
    return node.compareTo(other.node);
  }

  @override
  bool operator ==(Object other) =>
      other is Hlc &&
      ms == other.ms &&
      counter == other.counter &&
      node == other.node;

  @override
  int get hashCode => Object.hash(ms, counter, node);
}
