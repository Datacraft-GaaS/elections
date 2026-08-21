//! Serialise ExtendedFeatureSet back to ANSI/NIST-ITL Type-9 field 300+ bytes.
//!
//! Reference: NIST SP 500-290 e3 Annex A.
//!
//! Wire format: u16 BE tag + u16 BE payload-length + payload, repeated per field.
//! Round-trip through parse_efs(serialise_efs(efs)) must be byte-identical.

use crate::features::{ExtendedFeatureSet, SingularPointKind, SkinConditionKind};

fn push_field(buf: &mut Vec<u8>, tag: u16, payload: &[u8]) {
    buf.extend_from_slice(&tag.to_be_bytes());
    buf.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    buf.extend_from_slice(payload);
}

/// Serialise an ExtendedFeatureSet to Type-9 extended field bytes.
pub fn serialise_efs(efs: &ExtendedFeatureSet) -> Vec<u8> {
    let mut buf = Vec::new();

    // Field 300: pattern class
    push_field(&mut buf, 300, &[efs.pattern_class.to_code()]);

    // Field 301: singular points
    if !efs.singular_points.is_empty() {
        let mut p = Vec::with_capacity(efs.singular_points.len() * 7);
        for sp in &efs.singular_points {
            p.push(if sp.kind == SingularPointKind::Core {
                0u8
            } else {
                1u8
            });
            p.extend_from_slice(&sp.x.to_be_bytes());
            p.extend_from_slice(&sp.y.to_be_bytes());
            let orient_raw = (sp.orientation * 10.0) as u16;
            p.extend_from_slice(&orient_raw.to_be_bytes());
        }
        push_field(&mut buf, 301, &p);
    }

    // Field 302: quality map
    if let Some(qm) = &efs.quality_map {
        let mut p = Vec::with_capacity(4 + qm.data.len());
        p.extend_from_slice(&qm.width.to_be_bytes());
        p.extend_from_slice(&qm.height.to_be_bytes());
        p.extend_from_slice(&qm.data);
        push_field(&mut buf, 302, &p);
    }

    // Field 303: flow map
    if let Some(fm) = &efs.flow_map {
        let mut p = Vec::with_capacity(4 + fm.data.len());
        p.extend_from_slice(&fm.width.to_be_bytes());
        p.extend_from_slice(&fm.height.to_be_bytes());
        p.extend_from_slice(&fm.data);
        push_field(&mut buf, 303, &p);
    }

    // Field 304: wavelength map
    if let Some(wm) = &efs.wavelength_map {
        let mut p = Vec::with_capacity(4 + wm.data.len() * 2);
        p.extend_from_slice(&wm.width.to_be_bytes());
        p.extend_from_slice(&wm.height.to_be_bytes());
        for v in &wm.data {
            p.extend_from_slice(&v.to_be_bytes());
        }
        push_field(&mut buf, 304, &p);
    }

    // Field 305: segmentation mask
    if let Some(seg) = &efs.segmentation {
        let mut p = Vec::with_capacity(4 + seg.data.len());
        p.extend_from_slice(&seg.width.to_be_bytes());
        p.extend_from_slice(&seg.height.to_be_bytes());
        p.extend_from_slice(&seg.data);
        push_field(&mut buf, 305, &p);
    }

    // Field 320: minutia extended
    if !efs.minutia_extended.is_empty() {
        let mut p = Vec::new();
        for me in &efs.minutia_extended {
            p.push(me.minutia_idx as u8);
            p.push(me.feature_type);
            let mut flags = 0u8;
            if me.is_dot {
                flags |= 0x01;
            }
            if me.is_incipient {
                flags |= 0x02;
            }
            p.push(flags);
            p.push(me.ridge_counts_to_neighbours.len() as u8);
            for (nidx, count) in &me.ridge_counts_to_neighbours {
                p.push(*nidx as u8);
                p.push(*count);
            }
        }
        push_field(&mut buf, 320, &p);
    }

    // Field 330: dots
    if !efs.dots.is_empty() {
        let mut p = Vec::with_capacity(efs.dots.len() * 5);
        for d in &efs.dots {
            p.extend_from_slice(&d.x.to_be_bytes());
            p.extend_from_slice(&d.y.to_be_bytes());
            p.push(d.quality);
        }
        push_field(&mut buf, 330, &p);
    }

    // Field 331: incipient ridges
    if !efs.incipient_ridges.is_empty() {
        let mut p = Vec::with_capacity(efs.incipient_ridges.len() * 9);
        for ir in &efs.incipient_ridges {
            p.extend_from_slice(&ir.start_x.to_be_bytes());
            p.extend_from_slice(&ir.start_y.to_be_bytes());
            p.extend_from_slice(&ir.end_x.to_be_bytes());
            p.extend_from_slice(&ir.end_y.to_be_bytes());
            p.push(ir.quality);
        }
        push_field(&mut buf, 331, &p);
    }

    // Field 350: skin conditions
    if !efs.skin_conditions.is_empty() {
        let mut p = Vec::with_capacity(efs.skin_conditions.len() * 9);
        for sc in &efs.skin_conditions {
            let kind_byte = match sc.kind {
                SkinConditionKind::Crease => 0u8,
                SkinConditionKind::Scar => 1,
                SkinConditionKind::Wart => 2,
                SkinConditionKind::OtherSkinCondition => 3,
            };
            p.push(kind_byte);
            p.extend_from_slice(&sc.x.to_be_bytes());
            p.extend_from_slice(&sc.y.to_be_bytes());
            p.extend_from_slice(&sc.width.to_be_bytes());
            p.extend_from_slice(&sc.height.to_be_bytes());
        }
        push_field(&mut buf, 350, &p);
    }

    // Field 360: ridge edge shapes
    if !efs.ridge_edge_shapes.is_empty() {
        let mut p = Vec::with_capacity(efs.ridge_edge_shapes.len() * 9);
        for re in &efs.ridge_edge_shapes {
            p.push(re.minutia_idx as u8);
            p.extend_from_slice(&re.curvature.to_be_bytes());
            p.extend_from_slice(&re.ridge_width.to_be_bytes());
        }
        push_field(&mut buf, 360, &p);
    }

    buf
}
