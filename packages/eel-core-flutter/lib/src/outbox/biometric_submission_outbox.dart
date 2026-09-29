import 'dart:convert';
import 'package:uuid/uuid.dart';
import 'package:dio/dio.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

const _storage = FlutterSecureStorage(
	aOptions: AndroidOptions(encryptedSharedPreferences: true),
);

class BiometricSubmission {
	final String submissionId;        // UUID7
	final String actorId;
	final String actorRole;
	final String? fpTemplate;         // ISO 19794-2 base64
	final String? irisTemplate;       // ISO 19794-6 base64
	final int fpQualityScore;
	final int irisQualityScore;
	final String whereGps;
	final String auditId;            // Link to audit entry
	final String proofHash;          // SHA-256 of fp+iris canonical
	final String signature;          // Ed25519 signature
	final DateTime createdAt;
	BiometricSubmissionStatus status;
	int uploadAttempts;
	DateTime? lastAttemptAt;
	String? lastError;
	BiometricSubmission({
		required this.submissionId,
		required this.actorId,
		required this.actorRole,
		this.fpTemplate,
		this.irisTemplate,
		required this.fpQualityScore,
		required this.irisQualityScore,
		required this.whereGps,
		required this.auditId,
		required this.proofHash,
		required this.signature,
		required this.createdAt,
		this.status = BiometricSubmissionStatus.queued,
		this.uploadAttempts = 0,
		this.lastAttemptAt,
		this.lastError,
	});

	factory BiometricSubmission.fromJson(Map<String, dynamic> j) => BiometricSubmission(
		submissionId: j['submissionId'] as String,
		actorId: j['actorId'] as String,
		actorRole: j['actorRole'] as String,
		fpTemplate: j['fpTemplate'] as String?,
		irisTemplate: j['irisTemplate'] as String?,
		fpQualityScore: (j['fpQualityScore'] as num).toInt(),
		irisQualityScore: (j['irisQualityScore'] as num).toInt(),
		whereGps: j['whereGps'] as String,
		auditId: j['auditId'] as String,
		proofHash: j['proofHash'] as String,
		signature: j['signature'] as String,
		createdAt: DateTime.parse(j['createdAt'] as String),
		status: BiometricSubmissionStatus.values.firstWhere(
			(s) => s.name == j['status'],
			orElse: () => BiometricSubmissionStatus.queued),
		uploadAttempts: (j['uploadAttempts'] as num).toInt(),
		lastAttemptAt: j['lastAttemptAt'] != null ? DateTime.parse(j['lastAttemptAt'] as String) : null,
		lastError: j['lastError'] as String?,
	);

	Map<String, dynamic> toJson() => {
		'submissionId': submissionId,
		'actorId': actorId,
		'actorRole': actorRole,
		'fpTemplate': fpTemplate,
		'irisTemplate': irisTemplate,
		'fpQualityScore': fpQualityScore,
		'irisQualityScore': irisQualityScore,
		'whereGps': whereGps,
		'auditId': auditId,
		'proofHash': proofHash,
		'signature': signature,
		'createdAt': createdAt.toIso8601String(),
		'status': status.name,
		'uploadAttempts': uploadAttempts,
		'lastAttemptAt': lastAttemptAt?.toIso8601String(),
		'lastError': lastError,
	};
}

enum BiometricSubmissionStatus {
	queued,        // Local dedup not yet run
	deduplicated,  // Duplicate found on same device — rejected
	pendingUpload, // Local dedup passed; awaiting upload
	uploading,     // HTTP request in flight
	uploaded,      // Confirmed by backend
	failed,        // Upload error after retries
}

class DedupResult {
	final bool isDuplicate;
	final String? existingSubmissionId;
	final DedupMatchType matchType;
	const DedupResult({required this.isDuplicate, this.existingSubmissionId, required this.matchType});
}

enum DedupMatchType {
	none,           // No local duplicate
	fingerprint,    // Same fp template seen on this device
	iris,           // Same iris template seen on this device
	both,           // Both fp and iris match existing local submission
}

/// Outbox: stores pending biometric submissions with local dedup + operator-controlled upload.
/// Signature for real biometric template comparison (minutiae/pattern, not bytes).
typedef BiometricMatcher = Future<bool> Function(String captured, String enrolled);

class BiometricSubmissionOutbox {
	final Dio _dio;
	final String _outboxKeyPrefix = 'eel.bio.outbox.';

	/// Real biometric comparison functions — MUST be provided by the caller.
	/// In bvr-kit, these wrap Fap60Driver.match() (minutiae) and AntIrisDriver.match() (texture).
	final BiometricMatcher matchFingerprint;
	final BiometricMatcher matchIris;

	BiometricSubmissionOutbox({
		required this.matchFingerprint,
		required this.matchIris,
		Dio? dio,
		String? baseUrl,
	}) : _dio = dio ?? Dio(BaseOptions(
			baseUrl: baseUrl ?? 'https://bvr.eel.local/api/v1/bvr',
			connectTimeout: const Duration(seconds: 10),
			receiveTimeout: const Duration(seconds: 30),
		));

	/// Enqueue a new submission. Runs LOCAL dedup first.
	/// Returns: { enqueued: bool, dedupResult: DedupResult, submissionId: String? }
	Future<({bool enqueued, DedupResult dedupResult, String? submissionId})> enqueue({
		required String actorId,
		required String actorRole,
		required String? fpTemplate,
		required String? irisTemplate,
		required int fpQualityScore,
		required int irisQualityScore,
		required String whereGps,
		required String auditId,
		required String proofHash,
		required String signature,
	}) async {
		// Run local dedup via REAL biometric match (minutiae/texture)
		final dedup = await _checkLocalDedup(fpTemplate ?? '', irisTemplate ?? '');
		if (dedup.isDuplicate) {
			final rejected = BiometricSubmission(
				submissionId: const Uuid().v7().toString(),
				actorId: actorId, actorRole: actorRole,
				fpTemplate: fpTemplate, irisTemplate: irisTemplate,
				fpQualityScore: fpQualityScore, irisQualityScore: irisQualityScore,
				whereGps: whereGps, auditId: auditId,
				proofHash: proofHash, signature: signature,
				createdAt: DateTime.now().toUtc(),
				status: BiometricSubmissionStatus.deduplicated,
			);
			await _store(rejected);
			return (enqueued: false, dedupResult: dedup, submissionId: null);
		}

		final sub = BiometricSubmission(
			submissionId: const Uuid().v7().toString(),
			actorId: actorId, actorRole: actorRole,
			fpTemplate: fpTemplate, irisTemplate: irisTemplate,
			fpQualityScore: fpQualityScore, irisQualityScore: irisQualityScore,
			whereGps: whereGps, auditId: auditId,
			proofHash: proofHash, signature: signature,
			createdAt: DateTime.now().toUtc(),
			status: BiometricSubmissionStatus.pendingUpload,
		);
		await _store(sub);
		// Template stored on submission; read on dedup; no separate index needed

		return (enqueued: true, dedupResult: dedup, submissionId: sub.submissionId);
	}

	/// Local dedup: REAL biometric comparison via FAP60 (minutiae) + AntIris (texture).
	/// Uses stored templates from prior submissions + driver.match() — not hash equality.
	Future<DedupResult> _checkLocalDedup(String fpTemplate, String irisTemplate) async {
		if (fpTemplate.isEmpty && irisTemplate.isEmpty) {
			return const DedupResult(isDuplicate: false, matchType: DedupMatchType.none);
		}
		final previous = await _readAll();
		final candidates = previous.where((s) =>
			s.status == BiometricSubmissionStatus.uploaded ||
			s.status == BiometricSubmissionStatus.pendingUpload ||
			s.status == BiometricSubmissionStatus.queued);

		String? foundId;
		bool fpMatched = false;
		bool irisMatched = false;

		// Real biometric comparison: minutiae (FAP60) + texture (AntIris), not byte equality
		for (final s in candidates) {
			if (fpTemplate.isNotEmpty && s.fpTemplate != null && s.fpTemplate!.isNotEmpty) {
				fpMatched = await matchFingerprint(fpTemplate, s.fpTemplate!);
			}
			if (irisTemplate.isNotEmpty && s.irisTemplate != null && s.irisTemplate!.isNotEmpty) {
				irisMatched = await matchIris(irisTemplate, s.irisTemplate!);
			}
			if (fpMatched || irisMatched) {
				foundId = s.submissionId;
				break;
			}
		}

		if (!fpMatched && !irisMatched) return const DedupResult(isDuplicate: false, matchType: DedupMatchType.none);
		if (fpMatched && irisMatched) return DedupResult(isDuplicate: true, existingSubmissionId: foundId, matchType: DedupMatchType.both);
		if (fpMatched) return DedupResult(isDuplicate: true, existingSubmissionId: foundId, matchType: DedupMatchType.fingerprint);
		return DedupResult(isDuplicate: true, existingSubmissionId: foundId, matchType: DedupMatchType.iris);
	}


	/// Operator-initiated upload. Drains outbox in chronological order.
	/// Called from UI button "Upload now" after network connectivity is confirmed.
	Future<UploadResult> uploadAll({
		String? authToken,
		void Function(int total, int current)? onProgress,
	}) async {
		final all = await _readAll();
		final pending = all.where((s) =>
			s.status == BiometricSubmissionStatus.pendingUpload ||
			(s.status == BiometricSubmissionStatus.failed && s.uploadAttempts < 5)
		).toList()..sort((a, b) => a.createdAt.compareTo(b.createdAt));

		if (pending.isEmpty) {
			return const UploadResult(total: 0, uploaded: 0, failed: 0, deduplicated: 0);
		}

		int uploaded = 0;
		int failed = 0;
		int deduplicated = 0;
		for (var i = 0; i < pending.length; i++) {
			final s = pending[i];
			onProgress?.call(pending.length, i);
			s.status = BiometricSubmissionStatus.uploading;
			s.uploadAttempts++;
			s.lastAttemptAt = DateTime.now().toUtc();
			await _store(s);
			try {
				final r = await _dio.post<Map<String, dynamic>>(
					'/enrol',
					data: s.toJson(),
					options: Options(headers: authToken != null ? {'Authorization': 'Bearer $authToken'} : null),
				);
				final d = r.data;
				if (d != null && d['status'] == 'duplicate') {
					// Server says: this template already exists in central registry
					s.status = BiometricSubmissionStatus.deduplicated;
					s.lastError = 'server_duplicate:${d['existing_voter_id']}';
					deduplicated++;
				} else {
					s.status = BiometricSubmissionStatus.uploaded;
					s.lastError = null;
					uploaded++;
				}
			} on DioException catch (e) {
				s.status = BiometricSubmissionStatus.failed;
				s.lastError = 'http_${e.response?.statusCode}:${e.message}';
				failed++;
			} catch (e) {
				s.status = BiometricSubmissionStatus.failed;
				s.lastError = e.toString();
				failed++;
			}
			await _store(s);
		}
		onProgress?.call(pending.length, pending.length);
		return UploadResult(
			total: pending.length, uploaded: uploaded, failed: failed, deduplicated: deduplicated,
		);
	}

	/// Status summary for UI.
	Future<OutboxStatus> status() async {
		final all = await _readAll();
		return OutboxStatus(
			total: all.length,
			queued: all.where((s) => s.status == BiometricSubmissionStatus.queued).length,
			pendingUpload: all.where((s) => s.status == BiometricSubmissionStatus.pendingUpload).length,
			uploading: all.where((s) => s.status == BiometricSubmissionStatus.uploading).length,
			uploaded: all.where((s) => s.status == BiometricSubmissionStatus.uploaded).length,
			failed: all.where((s) => s.status == BiometricSubmissionStatus.failed).length,
			deduplicated: all.where((s) => s.status == BiometricSubmissionStatus.deduplicated).length,
		);
	}

	/// Remove uploaded + deduplicated entries older than 30 days (retention policy).
	Future<void> cleanup() async {
		final all = await _readAll();
		final cutoff = DateTime.now().toUtc().subtract(const Duration(days: 30));
		for (final s in all) {
			if ((s.status == BiometricSubmissionStatus.uploaded ||
				s.status == BiometricSubmissionStatus.deduplicated) &&
				s.createdAt.isBefore(cutoff)) {
				await _storage.delete(key: _outboxKeyPrefix + s.submissionId);
			}
		}
	}

	// ── Storage helpers (encrypted secure storage) ─────────────────────────────

	Future<void> _store(BiometricSubmission s) async =>
		_storage.write(key: _outboxKeyPrefix + s.submissionId, value: jsonEncode(s.toJson()));

	Future<List<BiometricSubmission>> _readAll() async {
		final all = await _storage.readAll();
		final submissions = <BiometricSubmission>[];
		for (final e in all.entries) {
			if (e.key.startsWith(_outboxKeyPrefix) && e.value.isNotEmpty) {
				try {
					submissions.add(BiometricSubmission.fromJson(jsonDecode(e.value)));
				} catch (_) {/* skip corrupt entry */}
			}
		}
		return submissions;
	}

	// Real biometric matching (minutiae/texture) replaces hash-based dedup.
}

class OutboxStatus {
	final int total;
	final int queued;
	final int pendingUpload;
	final int uploading;
	final int uploaded;
	final int failed;
	final int deduplicated;
	const OutboxStatus({
		required this.total, required this.queued, required this.pendingUpload,
		required this.uploading, required this.uploaded,
		required this.failed, required this.deduplicated,
	});
}

class UploadResult {
	final int total;
	final int uploaded;
	final int failed;
	final int deduplicated;
	const UploadResult({
		required this.total, required this.uploaded,
		required this.failed, required this.deduplicated,
	});
}
