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

/// Public barrel for the eel telemetry sub-module.
///
/// Apps import this single file:
///   import 'package:eel_kit_sdk/eel_kit_sdk.dart';
library eel_kit_sdk_telemetry;

export 'src/telemetry/signal_sink.dart';
export 'src/telemetry/signal_types.dart';
export 'src/telemetry/adaptive_sampling.dart';
export 'src/telemetry/activity_tracker.dart';
export 'src/telemetry/attestation_pulse.dart';
export 'src/telemetry/battery_monitor.dart';
export 'src/telemetry/drop_detector.dart';
export 'src/telemetry/hash_chain.dart';
export 'src/telemetry/location_service.dart';
export 'src/telemetry/network_watcher.dart';
export 'src/telemetry/offline_queue.dart';
export 'src/telemetry/redactor.dart';
export 'src/telemetry/rfid_proximity_service.dart';
export 'src/telemetry/sensors_service.dart';
export 'src/telemetry/session_recorder.dart';
export 'src/telemetry/signer.dart';
export 'src/telemetry/sink_dispatcher.dart';
export 'src/telemetry/telemetry_service.dart';
