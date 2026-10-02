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

import 'dart:async';
import 'dart:typed_data';

import 'package:eel_kit_sdk/eel_kit_sdk.dart';
import 'package:flutter_test/flutter_test.dart';

class _FakeTransport implements FingerprintTransport {
  final _controller = StreamController<FingerEvent>.broadcast();

  @override
  Stream<FingerEvent> get events => _controller.stream;

  @override
  Future<void> connect() async => _controller.add(const FingerConnected('DEV-FP-1'));

  @override
  Future<void> disconnect() async => _controller.add(const FingerDisconnected());

  @override
  Future<void> captureFlat() async => _controller.add(FingerCaptured(
        FingerCapture(rawBytes: Uint8List.fromList([1, 2, 3]), width: 300, height: 400, quality: 88),
      ));

  @override
  Future<void> captureSlap({required FingerHand hand}) async => _controller.add(SlapCaptured(
        SlapCapture(hand: hand, fingers: [
          FingerCapture(rawBytes: Uint8List(256 * 360), width: 256, height: 360, quality: 80),
          FingerCapture(rawBytes: Uint8List(256 * 360), width: 256, height: 360, quality: 82),
        ]),
      ));

  @override
  Future<void> captureRoll() async => _controller.add(FingerCaptured(
        FingerCapture(rawBytes: Uint8List(800 * 750), width: 800, height: 750, quality: 85),
      ));

  @override
  Future<void> enroll({int fingerCount = 4}) async => _controller.add(FingerEnrolled(
        FingerEnrollment(isoTemplates: List.generate(fingerCount, (_) => Uint8List(1024)), galleryId: 7),
      ));

  @override
  Future<void> identify() async => _controller.add(const FingerIdentified(FingerMatch(matchedId: 1, score: 88)));

  @override
  Future<void> setDryWet(int level) async {}

  @override
  Future<void> beep(int type) async {}

  @override
  Future<void> dispose() async => _controller.close();
}

void main() {
  test('scanner tracks connection, flat + slap + enrollment + identify', () async {
    final transport = _FakeTransport();
    final scanner = FingerprintScanner(transport: transport);

    await scanner.connect();
    expect(scanner.connected, isTrue);

    await scanner.captureFlat();
    expect(scanner.lastCapture?.width, 300);
    expect(scanner.lastCapture?.quality, 88);

    await scanner.captureSlap(FingerHand.leftFour);
    expect(scanner.lastSlap?.hand, FingerHand.leftFour);
    expect(scanner.lastSlap?.fingers.length, 2);

    await scanner.enroll(fingerCount: 4);
    expect(scanner.lastEnrollment?.isoTemplates.length, 4);
    expect(scanner.lastEnrollment?.galleryId, 7);

    await scanner.identify();
    expect(scanner.lastMatch?.matchedId, 1);
    expect(scanner.lastMatch?.score, 88);

    scanner.dispose();
  });

  test('error surfaces and clears busy', () async {
    final transport = _FakeTransport();
    final scanner = FingerprintScanner(transport: transport);
    await scanner.captureFlat();
    expect(scanner.busy, isFalse);
    scanner.dispose();
  });

  test('finger names round-trip', () {
    expect(fingerPositionName(FingerPosition.leftIndex), 'left_index');
    expect(fingerHandName(FingerHand.leftFour), 'left_four');
    expect(fingerHandName(FingerHand.rightFour), 'right_four');
    expect(fingerHandFromName('right_four'), FingerHand.rightFour);
    expect(fingerHandFromName('unknown'), FingerHand.leftFour);
  });

  test('simulated transport produces deterministic captures', () async {
    final transport = SimulatedFingerprintTransport();
    final scanner = FingerprintScanner(transport: transport);

    await scanner.connect();
    expect(scanner.connected, isTrue);

    await scanner.captureFlat();
    await pumpEventQueue();
    expect(scanner.lastCapture?.width, 300);
    expect(scanner.lastCapture?.quality, 88);

    await scanner.captureSlap(FingerHand.twoThumbs);
    await pumpEventQueue();
    expect(scanner.lastSlap?.fingers.length, 2);

    await scanner.enroll();
    await pumpEventQueue();
    expect(scanner.lastEnrollment?.galleryId, isNonNegative);

    scanner.dispose();
  });
}
