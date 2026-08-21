# eel-abis-fp-sourceafis-bridge

Native Rust reimplementation of the **SourceAFIS** fingerprint matching algorithm
for use in the eel ABIS pipeline.

SourceAFIS is authored by Robert Vaszary (Rob Wozniak) and published under
MPL-2.0.  Algorithm specification: <https://sourceafis.machinezoo.com/algorithm>

This crate implements the same algorithm in pure Rust without linking to the Java
library.  It is intended as a second-opinion matcher alongside the MCC baseline.

## Algorithm phases

1. Skeleton extraction: Otsu binarisation + Zhang-Suen thinning with SourceAFIS
   ridge-width pre-filter to suppress short spurs.
2. Minutia detection: endings (1 neighbour) and bifurcations (3 neighbours) at
   skeleton branch points.
3. Local ridge structure: per-minutia edge table encoding relative distance,
   relative direction, and inter-minutia ridge count to nearby neighbours.
4. Matching: best seed-pair selection, radial extension, score aggregation.

## References

- R. Vaszary, "SourceAFIS algorithm," <https://sourceafis.machinezoo.com/algorithm>
- T. Zhang, C. Suen, "A fast parallel algorithm for thinning digital patterns,"
  Commun. ACM 27(3), 1984.
- ISO/IEC 19794-2:2011, "Finger minutiae data."

## Licence note

This reimplementation is part of the eel project (Apache-2.0) and does not
redistribute any Java code from the SourceAFIS project.
