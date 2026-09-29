import 'package:flutter_test/flutter_test.dart';
import 'package:eel_rts_kit/src/core/at_command/sms_pdu.dart';

/// A worked reference: "Hello" to +447700900000, GSM 7-bit, no UDH.
///
/// Hand-checked against 3GPP TS 23.038 §9.1.2.5: TPDU type 0x11, MR 0x00,
/// DA length 12 semi-octets with type 0x91, each digit pair stored with the
/// low nibble first, PID 0, DCS 0, VP 0xAA, then the UD length and the packed
/// septets.
void main() {
  group('GSM 7-bit alphabet', () {
    test('encodes the reference message', () {
      final pdus = encodeSmsSubmit(
        destinationNumber: '+447700900000',
        payloadBytes: 'Hello'.codeUnits,
        dataCodingScheme: 0x00,
      );
      expect(pdus, hasLength(1));

      final pdu = pdus.single;
      // Header: type, MR, address length/type, digits, PID, DCS, VP, UD length.
      expect(pdu.substring(0, 4), '1100');
      // +447700900000 is 12 digits, so 12 semi-octets (0x0C).
      expect(pdu.substring(4, 8), '0C91');
      // Pairs (4,4) (7,7) (0,0) (9,0) (0,0) (0,0), each stored low nibble
      // first: 44 77 00 09 00 00.
      expect(pdu.substring(8, 20), '447700090000');
      expect(pdu.substring(20, 22), '00'); // PID
      expect(pdu.substring(22, 24), '00'); // DCS
      expect(pdu.substring(24, 26), 'AA'); // validity period
      expect(pdu.substring(26, 28), '05'); // 5 septets
    });

    test('round-trips text through pack and unpack', () {
      const text = 'Hello, world! 12345 @£\$¥ èéù';
      final pdus = encodeSmsSubmit(
        destinationNumber: '+254700000000',
        payloadBytes: text.codeUnits,
        dataCodingScheme: 0x00,
      );
      final pdu = pdus.single;
      // The UD length is at byte 13; the septets follow it.
      final septetCount = int.parse(pdu.substring(26, 28), radix: 16);
      final udHex = pdu.substring(28);
      final octets = <int>[
        for (var i = 0; i < udHex.length; i += 2)
          int.parse(udHex.substring(i, i + 2), radix: 16),
      ];
      expect(decodeGsm7(octets, septetCount), text);
    });

    test('an escaped character costs two septets, not one', () {
      // 'L with a stroke' is in the extension table, so it needs the 0x1B
      // escape: one character, two septets. ('[' has no GSM 7-bit form at
      // all, which the rejection test below covers.)
      const withEscape = 'a\u0141bc';
      final pdus = encodeSmsSubmit(
        destinationNumber: '+254700000000',
        payloadBytes: withEscape.codeUnits,
        dataCodingScheme: 0x00,
      );
      final pdu = pdus.single;
      final septetCount = int.parse(pdu.substring(26, 28), radix: 16);
      expect(septetCount, 5, reason: '4 characters, one escaped, so 5 septets');

      final udHex = pdu.substring(28);
      final octets = <int>[
        for (var i = 0; i < udHex.length; i += 2)
          int.parse(udHex.substring(i, i + 2), radix: 16),
      ];
      expect(decodeGsm7(octets, septetCount), withEscape);
    });

    test('splits a long message and marks all but the first with a UDH', () {
      final long = 'A' * 400;
      final pdus = encodeSmsSubmit(
        destinationNumber: '+254700000000',
        payloadBytes: long.codeUnits,
        dataCodingScheme: 0x00,
      );
      expect(pdus, hasLength(3), reason: '400 septets is 160 + 153 + 87');
      // The first part carries no UDH, so it takes all 160 septets and its
      // length field counts septets, not octets.
      expect(pdus[0].substring(26, 28), 'A0');
      expect(pdus[0].substring(28, 30), isNot('05'), reason: 'no UDH on part 1');
      // Later parts carry a 6-octet UDH (len, ref-ie, 8-bit ref, ref, total,
      // part) and TP-UDL counts it, so 6 + 134 packed octets.
      expect(pdus[1].substring(26, 28), '8C');
      expect(pdus[1].substring(28, 40), '050003000302');
      expect(pdus[2].substring(26, 28), '53');
      expect(pdus[2].substring(28, 40), '050003000303');
    });

    test('rejects a character outside the alphabet', () {
      expect(
        () => encodeSmsSubmit(
          destinationNumber: '+254700000000',
          payloadBytes: '☃'.codeUnits, // snowman
          dataCodingScheme: 0x00,
        ),
        throwsA(isA<Exception>()),
      );
      expect(canEncodeAsGsm7('plain ascii'), isTrue);
      expect(canEncodeAsGsm7('snowman ☃'), isFalse);
    });
  });

  group('8-bit data', () {
    test('carries octets straight through', () {
      final pdus = encodeSmsSubmit(
        destinationNumber: '+254700000000',
        payloadBytes: '<STATUS:OK>'.codeUnits,
        dataCodingScheme: 0xF5,
      );
      expect(pdus, hasLength(1));
      final pdu = pdus.single;
      expect(pdu.substring(22, 24), 'F5', reason: 'the DCS as given');
      expect(pdu.substring(26, 28), '0B', reason: '11 octets of payload');
      expect(pdu.substring(28), '3C5354415455533A4F4B3E');
    });
  });

  group('UCS-2', () {
    test('encodes a non-Latin character as UTF-16 big endian', () {
      final pdus = encodeSmsSubmit(
        destinationNumber: '+254700000000',
        payloadBytes: 'Ā'.codeUnits,
        dataCodingScheme: 0x08,
      );
      expect(pdus, hasLength(1));
      // One UTF-16 unit for U+0100, big endian.
      expect(pdus.single.substring(28), '0100');
    });
  });

  group('address encoding', () {
    test('a national number is type 0x81, an international one 0x91', () {
      final national = encodeSmsSubmit(
        destinationNumber: '0700000000',
        payloadBytes: 'x'.codeUnits,
        dataCodingScheme: 0xF5,
      ).single;
      final international = encodeSmsSubmit(
        destinationNumber: '+254700000000',
        payloadBytes: 'x'.codeUnits,
        dataCodingScheme: 0xF5,
      ).single;
      expect(national.substring(4, 8), '0A81');
      expect(international.substring(4, 8), '0C91');
    });

    test('digits are swapped within each octet', () {
      final pdu = encodeSmsSubmit(
        destinationNumber: '+1234567890',
        payloadBytes: 'x'.codeUnits,
        dataCodingScheme: 0xF5,
      ).single;
      // 1234567890 -> 21 43 65 87 09
      expect(pdu.substring(8, 18), '2143658709');
    });
  });
}
