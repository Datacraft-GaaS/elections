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

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use chrono::Utc;
use std::io::Cursor;
use uuid::Uuid;

use crate::{
    domain::{GaitCycle, GaitFrame, GaitTemplate, JointAngle, NUM_JOINTS},
    GaitError,
};

const MAGIC: [u8; 4] = [0x47, 0x41, 0x49, 0x54];
// Minimum header: 4 magic + 4 cadence + 4 speed + 2 cycle count = 14 bytes.
const MIN_HEADER_LEN: usize = 14;

pub fn parse_gait(data: &[u8]) -> Result<GaitTemplate, GaitError> {
    if data.len() < MIN_HEADER_LEN {
        return Err(GaitError::BufferTooShort {
            need: MIN_HEADER_LEN,
            got: data.len(),
        });
    }
    if data[..4] != MAGIC {
        return Err(GaitError::InvalidMagic);
    }

    let mut cur = Cursor::new(&data[4..]);

    let cadence_steps_per_min = cur.read_f32::<LittleEndian>()?;
    let walking_speed_ms = cur.read_f32::<LittleEndian>()?;
    let cycle_count = cur.read_u16::<LittleEndian>()? as usize;

    let mut cycles = Vec::with_capacity(cycle_count);
    for _ in 0..cycle_count {
        let frame_count = cur.read_u16::<LittleEndian>()? as usize;
        let duration_ms = cur.read_u32::<LittleEndian>()?;

        let mut frames = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            let frame_idx = cur.read_u32::<LittleEndian>()?;
            let mut joints = [JointAngle {
                pitch: 0.0,
                yaw: 0.0,
                roll: 0.0,
            }; NUM_JOINTS];
            for j in joints.iter_mut() {
                j.pitch = cur.read_f32::<LittleEndian>()?;
                j.yaw = cur.read_f32::<LittleEndian>()?;
                j.roll = cur.read_f32::<LittleEndian>()?;
            }
            frames.push(GaitFrame { frame_idx, joints });
        }

        cycles.push(GaitCycle {
            frames,
            duration_ms,
        });
    }

    Ok(GaitTemplate {
        template_id: Uuid::new_v4().to_string(),
        cycles,
        cadence_steps_per_min,
        walking_speed_ms,
        captured_at: Utc::now(),
    })
}

pub fn serialize_gait(template: &GaitTemplate) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&MAGIC);
    buf.write_f32::<LittleEndian>(template.cadence_steps_per_min)
        .expect("vec write never fails");
    buf.write_f32::<LittleEndian>(template.walking_speed_ms)
        .expect("vec write never fails");
    buf.write_u16::<LittleEndian>(template.cycles.len() as u16)
        .expect("vec write never fails");

    for cycle in &template.cycles {
        buf.write_u16::<LittleEndian>(cycle.frames.len() as u16)
            .expect("vec write never fails");
        buf.write_u32::<LittleEndian>(cycle.duration_ms)
            .expect("vec write never fails");
        for frame in &cycle.frames {
            buf.write_u32::<LittleEndian>(frame.frame_idx)
                .expect("vec write never fails");
            for j in &frame.joints {
                buf.write_f32::<LittleEndian>(j.pitch)
                    .expect("vec write never fails");
                buf.write_f32::<LittleEndian>(j.yaw)
                    .expect("vec write never fails");
                buf.write_f32::<LittleEndian>(j.roll)
                    .expect("vec write never fails");
            }
        }
    }

    buf
}
