import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

/// Result of a verifier FFI call.
class VerifierResult {
  const VerifierResult({required this.ok, this.error});

  final bool ok;
  final String? error;
}

// Native function typedefs.
typedef _VerifyJsonNative = Int32 Function(Pointer<Uint8> json, IntPtr jsonLen);
typedef _VerifyJsonDart = int Function(Pointer<Uint8> json, int jsonLen);
typedef _LastErrorNative = Pointer<Uint8> Function();
typedef _LastErrorDart = Pointer<Uint8> Function();

/// FFI bindings to libeel-verifier.
///
/// Call [EelVerifierFfi.load] before any verify method.
/// On Linux (KIEMS kit target): libeel_verifier.so.
/// On Android: loaded from the APK native library directory.
/// On macOS (dev): libeel_verifier.dylib.
class EelVerifierFfi {
  EelVerifierFfi._(this._lib);

  final DynamicLibrary _lib;
  late final _VerifyJsonDart _inclusion;
  late final _VerifyJsonDart _consistency;
  late final _VerifyJsonDart _sthSignature;
  late final _VerifyJsonDart _witnessCosign;
  late final _LastErrorDart _lastError;

  static EelVerifierFfi load([String? path]) {
    final lib = path != null
        ? DynamicLibrary.open(path)
        : Platform.isAndroid
            ? DynamicLibrary.open('libeel_verifier.so')
            : Platform.isMacOS
                ? DynamicLibrary.open('libeel_verifier.dylib')
                : DynamicLibrary.open('libeel_verifier.so');
    final ffi = EelVerifierFfi._(lib);
    ffi._bind();
    return ffi;
  }

  void _bind() {
    _inclusion = _lib.lookupFunction<_VerifyJsonNative, _VerifyJsonDart>(
        'eel_verifier_inclusion');
    _consistency = _lib.lookupFunction<_VerifyJsonNative, _VerifyJsonDart>(
        'eel_verifier_consistency');
    _sthSignature = _lib.lookupFunction<_VerifyJsonNative, _VerifyJsonDart>(
        'eel_verifier_sth_signature');
    _witnessCosign = _lib.lookupFunction<_VerifyJsonNative, _VerifyJsonDart>(
        'eel_verifier_witness_cosign');
    _lastError = _lib
        .lookupFunction<_LastErrorNative, _LastErrorDart>(
            'eel_verifier_last_error_message');
  }

  VerifierResult _call(_VerifyJsonDart fn, Map<String, Object?> args) {
    final bytes = utf8.encode(jsonEncode(args));
    final ptr = calloc<Uint8>(bytes.length);
    try {
      for (var i = 0; i < bytes.length; i++) {
        ptr[i] = bytes[i];
      }
      final rc = fn(ptr, bytes.length);
      if (rc == 0) return const VerifierResult(ok: true);
      final errPtr = _lastError();
      final errMsg =
          errPtr == nullptr ? 'unknown error' : _readCString(errPtr);
      return VerifierResult(ok: false, error: errMsg);
    } finally {
      calloc.free(ptr);
    }
  }

  String _readCString(Pointer<Uint8> ptr) {
    final bytes = <int>[];
    var i = 0;
    while (ptr[i] != 0) {
      bytes.add(ptr[i]);
      i++;
    }
    return utf8.decode(bytes);
  }

  VerifierResult verifyInclusion(Map<String, Object?> args) =>
      _call(_inclusion, args);

  VerifierResult verifyConsistency(Map<String, Object?> args) =>
      _call(_consistency, args);

  VerifierResult verifySthSignature(Map<String, Object?> args) =>
      _call(_sthSignature, args);

  VerifierResult verifyWitnessCosign(Map<String, Object?> args) =>
      _call(_witnessCosign, args);
}
