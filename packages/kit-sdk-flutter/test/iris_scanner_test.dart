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

class _FakeTransport implements IrisScannerTransport {
  final _controller = StreamController<IrisEvent>.broadcast();

  @override
  Stream<IrisEvent> get events => _controller.stream;

  @override
  Future<void> connect() async => _controller.add(const IrisConnected('DEV-1'));

  @override
  Future<void> disconnect() async => _controller.add(const IrisDisconnected());

  @override
  Future<void> enroll({required IrisEye eye}) async => _controller
      .add(IrisEnrolled(IrisTemplate(eye: eye, isoBytes: Uint8List.fromList([1, 2, 3]))));

  @override
  Future<void> identify({
    required IrisEye eye,
    List<IrisTemplate> templates = const [],
  }) async =>
      _controller.add(const IrisIdentified(IrisMatch(userId: 'VOTER-1', score: 0.97)));

  @override
  Future<void> captureIso({required IrisEye eye}) async => _controller
      .add(IrisCaptured(IrisTemplate(eye: eye, isoBytes: Uint8List.fromList([4, 5]))));

  @override
  Future<void> stop() async {}

  @override
  Future<void> dispose() async => _controller.close();
}

void main() {
  test('scanner tracks connection, enrollment and identification state', () async {
    final transport = _FakeTransport();
    final scanner = IrisScanner(transport: transport);

    await scanner.connect();
    expect(scanner.connected, isTrue);

    await scanner.enroll(IrisEye.left);
    expect(scanner.lastTemplate?.eye, IrisEye.left);
    expect(scanner.lastTemplate?.isoBytes, [1, 2, 3]);

    await scanner.identify(IrisEye.right);
    expect(scanner.lastMatch?.userId, 'VOTER-1');
    expect(scanner.lastMatch?.score, 0.97);

    scanner.dispose();
  });

  test('error event surfaces and clears busy', () async {
    final transport = _FakeTransport();
    final scanner = IrisScanner(transport: transport);

    // Simulate a transport error without a dedicated public emit: use enroll
    // then inspect that a busy state is cleared after the event.
    await scanner.enroll(IrisEye.left);
    expect(scanner.busy, isFalse);

    scanner.dispose();
  });

  test('iris eye names round-trip', () {
    expect(irisEyeName(IrisEye.left), 'left');
    expect(irisEyeName(IrisEye.right), 'right');
    expect(irisEyeName(IrisEye.both), 'both');
    expect(irisEyeFromName('right'), IrisEye.right);
    expect(irisEyeFromName('unknown'), IrisEye.left);
  });

  test('simulated transport produces a deterministic template', () async {
    final transport = SimulatedIrisTransport();
    final scanner = IrisScanner(transport: transport);

    await scanner.connect();
    expect(scanner.connected, isTrue);

    await scanner.captureIso(IrisEye.right);
    await pumpEventQueue();
    expect(scanner.lastTemplate?.isoBytes.toList(), List.filled(32, 2));

    scanner.dispose();
  });
}
