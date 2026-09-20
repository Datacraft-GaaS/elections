/// GSM 03.40 SMS-SUBMIT PDU encoder and decoder.
/// Encodes Form 34A CBOR+zstd payload into SMS segments (7-bit GSM alphabet or 8-bit binary).
/// Each segment: 140 bytes payload max (160 chars × 7 bits / 8 bits per SMS frame).
/// For multi-segment: UDH (User Data Header) with 6-byte concatenated-SMS IEI.
library;

/// Segment a CBOR+zstd payload into SMS-SUBMIT PDU frames.
List<String> encodeSmsSubmit({
  required String destinationNumber, // E.164 format e.g. "+254700000000"
  required List<int> payloadBytes, // CBOR+zstd compressed payload
  int dataCodingScheme = 0xF5, // 8-bit data (DCS=11110101)
  int validityPeriod = 167,    // TP-VP: relative format, 167 = 24 hours
}) {
  final segments = <String>[];
  const maxPayloadPerSegment = 140; // 140 octets for 8-bit data

  for (int offset = 0; offset < payloadBytes.length; offset += maxPayloadPerSegment) {
    final segment = payloadBytes.sublist(
      offset,
      (offset + maxPayloadPerSegment).clamp(0, payloadBytes.length),
    );
    final isSingle = payloadBytes.length <= maxPayloadPerSegment;
    final segmentCount = (payloadBytes.length / maxPayloadPerSegment).ceil();
    final currentIndex = offset ~/ maxPayloadPerSegment;

    final udh = isSingle
        ? <int>[]
        : [
            // UDH for concatenated SMS (8-bit reference)
            0x00,                // UDH length
            0x08,                // IEI: concatenated SMS, 8-bit reference
            0x03,                // IEI length
            0x00,                // reference (MSB) — fill with 0 for now
            0x00,                // reference (LSB) — fill with 0 for now
            segmentCount,        // total segments
            currentIndex + 1,    // current segment number
          ];

    final tpUdhi = udh.isNotEmpty ? 0x40 : 0x00;

    // Build TP-UD (User Data)
    final tpUd = [...udh, ...segment];
    final tpUdLength = tpUd.length;

    // Build TP-DCS (Data Coding Scheme)
    // 0xF5 = 11110101: 8-bit data, uncompressed, no class
    final tpDcs = dataCodingScheme;

    // Build TP-VP (Validity Period — relative, 24h = 167)
    final tpVp = validityPeriod;

    // Build TP-MR (Message Reference — random)
    final tpMr = DateTime.now().millisecondsSinceEpoch % 255;

    // Build TP-OA (Originating Address — empty/unsupported for submit)
    // Build TP-DA (Destination Address)
    final daOctets = _encodeAddressOctlets(destinationNumber);
    final daLength = daOctets.length * 2; // semi-octets

    // Build full PDU
    final pdu = <int>[
      0x01,             // TP-MTI: SMS-SUBMIT
      tpUdhi | 0x01,    // TP-RD: reject duplicates (1) + TP-UDHI (UDH present if multi)
      tpMr,             // TP-Message-Reference
      daLength,         // TP-Destination-Address length (semi-octets)
      ...daOctets,      // TP-DA (semi-octets, BCD-inverted)
      0x00,             // TP-PID: protocol identifier (0 = default)
      tpDcs,            // TP-DCS: data coding scheme
      tpVp,             // TP-Validity-Period (relative)
      tpUdLength,       // TP-User-Data-Length (in septets for 7-bit, octets for 8-bit)
      ...tpUd,          // TP-User-Data
    ];

    segments.add(pdu.map((b) => b.toRadixString(16).padLeft(2, '0').toUpperCase()).join(''));
  }

  return segments;
}

/// Encode E.164 phone number to BCD semi-octets (inverted for GSM spec).
List<int> _encodeAddressOctlets(String number) {
  final cleaned = number.startsWith('+') ? number.substring(1) : number;
  final padded = cleaned.length.isOdd ? '${cleaned}0' : cleaned;
  final result = <int>[];
  for (int i = 0; i < padded.length; i += 2) {
    final hi = int.parse(padded[i]);     // most significant digit
    final lo = int.parse(padded[i + 1]); // least significant digit
    result.add((lo << 4) | hi);          // GSM 03.40: low nibble first
  }
  return result;
}

/// Decode GSM 7-bit alphabet string from byte array (Septets → String).
String decode7BitGsm(String pduHex) {
  final bytes = _hexToBytes(pduHex);
  final bits = bytes.expand((b) => [for (int i = 7; i >= 0; i--) (b >> i) & 1]).toList();
  // GSM 7-bit default alphabet index → char
  const gsm7 = '@£\$¥èéùìòÇ\nØø\rÅåΔ_ΦΓΛΩΠΨΣΘΞ\x1bC£€ÆæßÉ !"#¤%&\'()*+,-./0123456789:;<=>?¡ABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÑÜ§¿abcdefghijklmnopqrstuvwxyzäöñüà';
  // Decode septets → string
  final chars = <String>[];
  for (int i = 0; i + 7 <= bits.length; i += 7) {
    int idx = 0;
    for (int j = 0; j < 7; j++) {
      idx |= (bits[i + j] << j);
    }
    chars.add(idx < gsm7.length ? gsm7[idx] : '?');
  }
  return chars.join();
}

List<int> _hexToBytes(String hex) {
  final result = <int>[];
  for (int i = 0; i < hex.length; i += 2) {
    result.add(int.parse(hex.substring(i, i + 2), radix: 16));
  }
  return result;
}
