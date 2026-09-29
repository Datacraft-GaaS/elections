/// GSM 23.038 SMS-SUBMIT PDU encoding.
///
/// Covers the three user data encodings an SMS-SUBMIT can carry:
///   DCS 0x00  GSM 7-bit default alphabet, packed 7 bits per septet
///   DCS 0x04  8-bit data, one octet per octet
///   DCS 0x08  UCS-2, UTF-16 big-endian
///
/// Long messages are split at 153 septets (or 67 octets, or 70 UCS-2 units) and
/// each part but the first gets a user-data header carrying the segment number,
/// so the handset can reassemble them. An escape is the 0x1B septet, so
/// septet counting is not the same as character counting and the split point
/// has to respect that.
library;

/// The GSM 7-bit default alphabet, 3GPP TS 23.038 table 3.38.
///
/// Index is the septet value. `null` means the position is the extension
/// escape itself (0x1B), which never appears in encoded text.
const List<String?> _gsmDefault = <String?>[
  '@', '£', '\$', '¥', 'è', 'é', 'ù', 'ì', 'ò', 'Ç', '\n', 'Ø', 'ø', '\r', 'Å', 'å',
  'Δ', '_', 'Φ', 'Γ', 'Λ', 'Ω', 'Π', 'Ψ', 'Σ', 'Θ', 'Ξ', 'Æ', 'æ', 'ß', 'É', // 31
  ' ', '!', '"', '#', '¤', '%', '&', "'", '(', ')', '*', '+', ',', '-', '.', '/', // 47
  '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', // 63
  '¡', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', // 79
  'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'Ä', 'Ö', 'Ñ', 'Ü', '§', // 95
  '¿', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', // 111
  'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'ä', 'ö', 'ñ', 'ü', 'à', // 127
  null, 'Ä', null, 'É', 'æ', 'ß', 'É', null, null, null, null, null, null, null, null, null, // 143
  '^', null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, // 159
  ' ', '€', null, null, null, null, null, null, null, null, null, null, null, null, null, null, // 175
];

/// The 3GPP TS 23.038 table 6.2 extension alphabet, indexed by the septet
/// value the character occupies after the 0x1B escape. Nulls are unassigned.
const List<String?> _gsmExtensionTable = <String?>[
  'Ā', 'ā', 'Ă', 'ă', 'Ą', 'ą', 'Ć', 'ć',
  'Ĉ', 'ĉ', 'Ċ', 'ċ', 'Č', 'č', 'Ď', 'ď',
  'Đ', 'đ', 'Ē', 'ē', 'Ĕ', 'ĕ', 'Ė', 'ė',
  'Ę', 'ę', 'Ě', 'ě', 'Ĝ', 'ĝ', 'Ğ', 'ğ',
  'Ġ', 'ġ', 'Ģ', 'ģ', 'Ĥ', 'ĥ', 'Ħ', 'ħ',
  'Ĩ', 'ĩ', 'Ī', 'ī', 'Ĭ', 'ĭ', 'Į', 'į',
  'İ', 'ı', 'Ĳ', 'ĳ', 'Ĵ', 'ĵ', 'Ķ', 'ķ',
  'Ĺ', 'ĺ', 'Ļ', 'ļ', 'Ľ', 'ľ', 'Ŀ', 'ŀ',
  null, 'Ł', 'ł', 'Ń', 'ń', 'Ņ', 'ņ', 'Ň',
  'ň', 'ŉ', 'Ŋ', 'ŋ', 'Ō', 'ō', 'Ŏ', 'ŏ',
  'Ő', 'ő', null, null, 'Œ', 'œ', null, null,
  null, 'Ŕ', 'ŕ', 'Ŗ', 'ŗ', 'Ř', 'ř', 'Ś',
  'ś', 'Ŝ', 'ŝ', 'Ş', 'ş', 'Š', 'š', 'Ţ',
  'ţ', 'Ť', 'ť', 'Ŧ', 'ŧ', 'Ũ', 'ũ', 'Ū',
  'ū', 'Ŭ', 'ŭ', 'Ů', 'ů', 'Ű', 'ű', 'Ų',
  'ų', 'Ŵ', 'ŵ', 'Ŷ', 'ŷ', 'Ÿ', 'Ź', 'ź',
  'Ż', 'ż', 'Ž', 'ž', 'Ÿ', null, null, null,
];

final Map<String, int> _gsmExtension = <String, int>{
  for (var i = 0; i < _gsmExtensionTable.length; i++)
    if (_gsmExtensionTable[i] != null) _gsmExtensionTable[i]!: i,
};

const int _escape = 0x1B;

/// Septets per SMS-SUBMIT, and per concatenated part.
const int _septetsPerMessage = 160;
const int _septetsPerPart = 153; // 160 minus the 7-octet UDH
const int _octetsPerPart = 140; // 152 user octets plus 1 for the length
const int _ucs2UnitsPerPart = 67; // 134 octets plus the length octet

final Map<int, String> _septetToChar = <int, String>{
  for (var i = 0; i < _gsmDefault.length; i++)
    if (_gsmDefault[i] != null) i: _gsmDefault[i]!,
};

/// Septets for the DEFAULT alphabet only.
///
/// Kept separate from the extension table on purpose: both tables are indexed
/// 0..127, so a single character-to-septet map cannot tell which table a
/// character came from, and encoding it wrongly produces a message that
/// decodes to unrelated text.
final Map<String, int> _defaultToSeptet = <String, int>{
  for (final e in _septetToChar.entries) e.value: e.key,
};

/// The septets a string occupies in the GSM 7-bit alphabet.
List<int> _toSeptets(String text) {
  final out = <int>[];
  for (final ch in text.split('')) {
    final plain = _defaultToSeptet[ch];
    if (plain != null) {
      out.add(plain);
      continue;
    }
    final ext = _gsmExtension[ch];
    if (ext == null) throw _UnsupportedCharacter(ch);
    out
      ..add(_escape)
      ..add(ext);
  }
  return out;
}

class _UnsupportedCharacter implements Exception {
  _UnsupportedCharacter(this.character);
  final String character;
  @override
  String toString() =>
      'Character "$character" cannot be encoded in the GSM 7-bit alphabet';
}

/// Packs septets into octets, 7 bits each, most significant bit first.
List<int> packSeptets(List<int> septets) {
  final out = <int>[];
  var bitBuffer = 0;
  var bitCount = 0;
  for (final s in septets) {
    bitBuffer = (bitBuffer << 7) | (s & 0x7F);
    bitCount += 7;
    while (bitCount >= 8) {
      out.add((bitBuffer >> (bitCount - 8)) & 0xFF);
      bitCount -= 8;
    }
  }
  if (bitCount > 0) {
    out.add((bitBuffer << (8 - bitCount)) & 0xFF);
  }
  return out;
}

/// Unpacks septets from octets. The inverse of [packSeptets].
List<int> unpackSeptets(List<int> octets) {
  final out = <int>[];
  var bitBuffer = 0;
  var bitCount = 0;
  for (final o in octets) {
    bitBuffer = (bitBuffer << 8) | (o & 0xFF);
    bitCount += 8;
    while (bitCount >= 7) {
      out.add((bitBuffer >> (bitCount - 7)) & 0x7F);
      bitCount -= 7;
    }
  }
  return out;
}

/// UTF-16 big-endian units.
List<int> _toUcs2(String text) {
  final out = <int>[];
  for (final r in text.runes) {
    if (r > 0xFFFF) {
      // Outside the BMP, so a surrogate pair.
      final v = r - 0x10000;
      out.add(0xD800 + (v >> 10));
      out.add(0xDC00 + (v & 0x3FF));
    } else {
      out.add(r);
    }
  }
  return out;
}

/// True when [text] fits the GSM 7-bit default alphabet, escapes included.
bool canEncodeAsGsm7(String text) {
  try {
    _toSeptets(text);
    return true;
  } on _UnsupportedCharacter {
    return false;
  }
}

/// Address field: length in semi-octets, type-of-address, then swapped digits.
///
/// International numbers are prefixed with '+' and type 0x91, which is what
/// makes the handset treat them as international.
List<int> _encodeAddress(String number) {
  final international = number.startsWith('+');
  final digits = number.replaceAll(RegExp(r'[^0-9]'), '');
  if (digits.isEmpty) throw ArgumentError('Destination has no digits: $number');
  final semiOctets = digits.length;
  final type = international ? 0x91 : 0x81;
  final out = <int>[semiOctets, type];
  for (var i = 0; i < digits.length; i += 2) {
    final hi = digits.codeUnitAt(i) - 0x30;
    final lo = i + 1 < digits.length ? digits.codeUnitAt(i + 1) - 0x30 : 0xF;
    out.add((lo << 4) | hi);
  }
  return out;
}

String _byte(int b) => b.toRadixString(16).padLeft(2, '0').toUpperCase();

String _hex(List<int> bytes) => bytes.map(_byte).join();

/// Concatenated user-data header: 8-bit ref, total parts, this part.
List<int> _udh(int reference, int total, int part) =>
    <int>[0x05, 0x00, 0x03, reference, total, part];

/// Encodes [payload] as one or more SMS-SUBMIT PDUs addressed to
/// [destinationNumber], returned as uppercase hex without the SMSC prefix.
///
/// [dataCodingScheme] is the DCS octet. 0xF5 is 8-bit data, which is what the
/// GSM modem path uses for its `<STATUS:...>` envelopes.
///
/// The caller prepends the SMSC address length and address and sends the result
/// to `AT+CMGS=<length>`; that prefix depends on the SIM's SMSC and so is not
/// something this function can invent.
List<String> encodeSmsSubmit({
  required String destinationNumber,
  required List<int> payloadBytes,
  int dataCodingScheme = 0xF5,
  int messageReference = 0,
}) {
  final address = _encodeAddress(destinationNumber);
  const pduType = 0x11; // SMS-SUBMIT, VP relative
  const pid = 0x00;

  final List<List<int>> userDataParts;
  final bool isGsm7 = dataCodingScheme == 0x00 || dataCodingScheme == 0xF0;
  if (isGsm7) {
    // Septets, not characters: an escaped character costs two.
    final septets = _toSeptets(String.fromCharCodes(payloadBytes));
    final single = <List<int>>[septets];
    if (septets.length <= _septetsPerMessage) {
      userDataParts = single;
    } else {
      // The first part carries no UDH, so it fits the full 160 septets; every
      // later part gives up 7 octets to the header and takes 153.
      userDataParts = <List<int>>[septets.sublist(0, septets.length.clamp(0, _septetsPerMessage))];
      for (var i = _septetsPerMessage; i < septets.length; i += _septetsPerPart) {
        userDataParts.add(septets.sublist(
            i, (i + _septetsPerPart).clamp(0, septets.length)));
      }
    }
  } else if (dataCodingScheme == 0x08 || dataCodingScheme == 0xF8) {
    final units = <int>[];
    for (final b in payloadBytes) {
      units.addAll(_toUcs2(String.fromCharCode(b)));
    }
    userDataParts = <List<int>>[
      for (var i = 0; i < units.length; i += _ucs2UnitsPerPart)
        units.sublist(i, (i + _ucs2UnitsPerPart).clamp(0, units.length))
    ];
  } else {
    // 8-bit: octets straight through.
    final octets = payloadBytes;
    userDataParts = octets.length <= _octetsPerPart + 12
        ? <List<int>>[octets]
        : <List<int>>[
            for (var i = 0; i < octets.length; i += _octetsPerPart)
              octets.sublist(i, (i + _octetsPerPart).clamp(0, octets.length))
          ];
  }

  final total = userDataParts.length;
  return <String>[
    for (var part = 0; part < total; part++)
      () {
        final body = userDataParts[part];
        // The FIRST part of a concatenated message carries no user-data
        // header, so it uses the full message length and its length field is
        // in the natural unit. Later parts carry a 7-octet UDH and their
        // length is in octets, UDH included (3GPP TS 23.038 §9.2.3.24).
        final isFirst = part == 0;
        final header = isFirst
            ? const <int>[]
            : _udh(messageReference, total, part + 1);

        final List<int> ud;
        final int lengthOctets;
        if (isGsm7) {
          final packed = packSeptets(body);
          ud = <int>[...header, ...packed];
          // Without a UDH the length counts septets; with one, octets.
          lengthOctets = isFirst ? body.length : header.length + packed.length;
        } else if (dataCodingScheme == 0x08 || dataCodingScheme == 0xF8) {
          final octets = <int>[];
          for (final u in body) {
            octets.addAll(<int>[(u >> 8) & 0xFF, u & 0xFF]);
          }
          ud = <int>[...header, ...octets];
          lengthOctets = octets.length + header.length;
        } else {
          ud = <int>[...header, ...body];
          lengthOctets = body.length + header.length;
        }

        return _hex(<int>[
          pduType,
          messageReference & 0xFF,
          ...address,
          pid,
          dataCodingScheme & 0xFF,
          0xAA, // validity period: 4 days
          lengthOctets & 0xFF,
          ...ud,
        ]);
      }(),
  ];
}

/// Decodes GSM 7-bit septets back to text. Present so the encoder is testable
/// against its own output rather than against a hand-written expectation.
String decodeGsm7(List<int> octets, int septetCount) {
  final septets = unpackSeptets(octets).take(septetCount);
  final out = StringBuffer();
  var escaped = false;
  for (final s in septets) {
    if (s == _escape) {
      escaped = true;
      continue;
    }
    if (escaped) {
      out.write(s < _gsmExtensionTable.length ? (_gsmExtensionTable[s] ?? '') : '');
      escaped = false;
      continue;
    }
    out.write(_septetToChar[s] ?? '');
  }
  return out.toString();
}
