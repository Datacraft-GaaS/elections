//! bozorth3 — NIST NBIS minutiae matcher.
//!
//! Algorithm (Bozorth3, NIST-Developed, 2003-present):
//! 1. Build minutiae compatibility tables (galton features)
//! 2. Pair compatible minutiae into edge-tables
//! 3. Propagate pair chains forward and backward
//! 4. Score chains by sum of inter-minutiae distances
//!
//! Output: integer score 0–1000+ (no upper bound; threshold-tuned).
//! At threshold 45: FAR ~0.1%, FRR ~5% on FVC2002.
//! At threshold 50: FAR ~0.01% (production grade).

use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use super::NbError;
use crate::MatchResult;
use super::gallery::CandidateMatch;
use super::template::{MinutiaType, Template};
use super::ffi::{bozorth_match, NbisMinutia};

/// In-process Bozorth3 matcher over the linked NBIS static library
/// (no external binaries needed — contrast the `match_1_to_1` shell-out
/// below, which drives an installed NBIS `bin/bozorth3`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Bozorth3;

impl Bozorth3 {
    /// 1:1 match between two templates via real `bz_match`.
    pub fn match_templates(
        &self,
        probe: &Template,
        enrolled: &Template,
        threshold: u32,
    ) -> MatchResult {
        let to_nbis = |m: &super::template::Minutia| NbisMinutia {
            x: m.x as std::os::raw::c_int,
            y: m.y as std::os::raw::c_int,
            theta: ((m.theta_deg * 10.0) as i32).rem_euclid(1800) as std::os::raw::c_int,
            quality: m.quality as std::os::raw::c_int,
            mclass: match m.mclass {
                MinutiaType::RidgeEnd => 0,
                MinutiaType::Bifurcation => 1,
                MinutiaType::DontCare => 2,
            },
        };
        let p: Vec<NbisMinutia> = probe.minutiae.iter().map(&to_nbis).collect();
        let e: Vec<NbisMinutia> = enrolled.minutiae.iter().map(&to_nbis).collect();
        let score = bozorth_match(&p, &e).max(0) as u32;
        MatchResult {
            score,
            threshold,
            matched: score >= threshold,
        }
    }
}

pub fn match_1_to_1(
    probe: &Path,
    enrolled: &Path,
    threshold: u32,
    nbis_home: &Path,
) -> Result<MatchResult, NbError> {
    let bin = nbis_home.join("bin/bozorth3");
    if !bin.exists() {
        return Err(NbError::BinaryNotFound(bin.display().to_string()));
    }
    let out = Command::new(&bin)
        .arg("-T").arg(format!("{threshold}"))
        .arg(probe)
        .arg(enrolled)
        .output()?;

    if !out.status.success() {
        return Err(NbError::ToolFailed {
            tool: "bozorth3".into(),
            exit_code: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }

    // bozorth3 output: "score" (integer) on stdout
    let score_str = String::from_utf8_lossy(&out.stdout);
    let score: i32 = score_str.trim().parse().unwrap_or(0);
    Ok(MatchResult {
        score: score.max(0) as u32,
        threshold,
        matched: score as u32 >= threshold,
    })
}

/// 1:N search: run probe against every .xyt in gallery directory.
/// NBIS `bozorth3 -p` runs the probe against all enrolled templates
/// and outputs `<enrolled_id> <score>` per candidate.
pub fn match_1_to_n(
    probe: &Path,
    gallery_dir: &Path,
    threshold: u32,
    nbis_home: &Path,
) -> Result<Vec<CandidateMatch>, NbError> {
    let bin = nbis_home.join("bin/bozorth3");
    if !bin.exists() {
        return Err(NbError::BinaryNotFound(bin.display().to_string()));
    }
    if !gallery_dir.is_dir() {
        return Err(NbError::Io(format!("gallery not a dir: {}", gallery_dir.display())));
    }

    let out = Command::new(&bin)
        .arg("-T").arg(format!("{threshold}"))
        .arg("-p") // parallel gallery mode
        .arg(gallery_dir)
        .arg(probe)
        .output()?;

    if !out.status.success() {
        return Err(NbError::ToolFailed {
            tool: "bozorth3".into(),
            exit_code: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut candidates = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 { continue; }
        let id = parts[0].to_string();
        let score: i32 = parts[1].parse().unwrap_or(0);
        let score_u = score.max(0) as u32;
        if score_u >= threshold {
            candidates.push(CandidateMatch {
                person_id: id,
                score: score_u,
                matched: true,
                partition_key: String::new(),
            });
        }
    }
    // Sort descending by score (rank-N = first hit)
    candidates.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(candidates)
}
