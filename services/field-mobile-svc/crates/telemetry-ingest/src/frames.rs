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

//! Activity frame downsampler: collects raw movement signals and aggregates
//! into 1 Hz (per-second) activity frames stored in eel_telemetry_activity_frames.

use chrono::{DateTime, Timelike, Utc};
use uuid::Uuid;

use crate::domain::{MotionClass, TelemetryActivityFrame};

/// Raw sample from a movement/activity signal payload.
#[derive(Debug, Clone)]
pub struct MovementSample {
    pub device_id: Uuid,
    pub sampled_at: DateTime<Utc>,
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
    pub battery_pct: i16,
    pub battery_temp_c: f32,
    pub network_class: Option<String>,
    pub lat_coarse: Option<f32>,
    pub lon_coarse: Option<f32>,
}

/// Classify motion from averaged accelerometer magnitude.
pub fn classify_motion(
    accel_x_avg: f32,
    accel_y_avg: f32,
    accel_z_avg: f32,
    max_g: f32,
) -> MotionClass {
    let magnitude =
        (accel_x_avg * accel_x_avg + accel_y_avg * accel_y_avg + accel_z_avg * accel_z_avg).sqrt();

    if max_g > 3.0 {
        return MotionClass::Running;
    }
    if magnitude < 0.15 {
        MotionClass::Stationary
    } else if magnitude < 0.8 {
        MotionClass::Walking
    } else if magnitude < 2.5 {
        MotionClass::Running
    } else {
        MotionClass::Vehicle
    }
}

/// Truncate timestamp to the start of the containing second.
fn truncate_to_second(ts: DateTime<Utc>) -> DateTime<Utc> {
    ts.with_nanosecond(0).unwrap_or(ts)
}

/// Downsample a slice of raw samples into 1 Hz activity frames.
///
/// Samples are grouped by (device_id, second_bucket). Within each group
/// values are averaged and motion is classified.
pub fn downsample(samples: Vec<MovementSample>) -> Vec<TelemetryActivityFrame> {
    use std::collections::HashMap;

    // Group by (device_id, second_bucket).
    let mut groups: HashMap<(Uuid, DateTime<Utc>), Vec<MovementSample>> = HashMap::new();
    for s in samples {
        let bucket = truncate_to_second(s.sampled_at);
        groups.entry((s.device_id, bucket)).or_default().push(s);
    }

    let mut frames = Vec::with_capacity(groups.len());
    for ((device_id, second_bucket), group) in groups {
        let n = group.len() as f32;
        let accel_x_avg = group.iter().map(|s| s.accel_x).sum::<f32>() / n;
        let accel_y_avg = group.iter().map(|s| s.accel_y).sum::<f32>() / n;
        let accel_z_avg = group.iter().map(|s| s.accel_z).sum::<f32>() / n;
        let gyro_x_avg = group.iter().map(|s| s.gyro_x).sum::<f32>() / n;
        let gyro_y_avg = group.iter().map(|s| s.gyro_y).sum::<f32>() / n;
        let gyro_z_avg = group.iter().map(|s| s.gyro_z).sum::<f32>() / n;
        let battery_pct = group.last().map(|s| s.battery_pct).unwrap_or(0);
        let battery_temp_c = group.last().map(|s| s.battery_temp_c).unwrap_or(0.0);
        let network_class = group.iter().rev().find_map(|s| s.network_class.clone());
        let lat_coarse = group.iter().rev().find_map(|s| s.lat_coarse);
        let lon_coarse = group.iter().rev().find_map(|s| s.lon_coarse);

        // Max instantaneous acceleration magnitude.
        let accel_max_g = group
            .iter()
            .map(|s| (s.accel_x * s.accel_x + s.accel_y * s.accel_y + s.accel_z * s.accel_z).sqrt())
            .fold(0.0f32, f32::max);

        let motion_class = classify_motion(accel_x_avg, accel_y_avg, accel_z_avg, accel_max_g);

        frames.push(TelemetryActivityFrame {
            frame_id: Uuid::now_v7(),
            device_id,
            second_bucket,
            accel_x_avg,
            accel_y_avg,
            accel_z_avg,
            accel_max_g,
            gyro_x_avg,
            gyro_y_avg,
            gyro_z_avg,
            motion_class,
            battery_pct,
            battery_temp_c,
            network_class,
            lat_coarse,
            lon_coarse,
        });
    }

    frames
}
