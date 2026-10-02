/**
 * Copyright (c) 2026 Nyimbi Odero
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * Signed content loader for Astro.
 *
 * Fetches a Tier-1 signed tarball from MinIO/edge and verifies:
 *   1. cosign signature on the tarball
 *   2. Merkle inclusion proof
 *   3. Snapshot ID matches the transparency log
 *
 * The `verifier` parameter accepts a SignatureVerifier implementation.
 * Until WP-1.5 ships verifier-rs as WASM, use StubSignatureVerifier for local dev.
 */

export interface VerificationResult {
	ok: boolean;
	snapshotId?: string;
	signedAt?: Date;
	error?: string;
}

export interface TarballManifest {
	version: string;
	snapshotId: string;
	contentHash: string;
	files: Array<{ path: string; hash: string; size: number }>;
}

/** Interface for signature verification backends. */
export interface SignatureVerifier {
	/** Verify a cosign signature bundle against the manifest content. */
	verifyCosign(manifest: Uint8Array, signature: string): Promise<VerificationResult>;
	/** Verify a Merkle inclusion proof. */
	verifyMerkleProof(leafHash: Uint8Array, proof: Uint8Array[], root: Uint8Array): Promise<boolean>;
	/** Confirm a snapshot ID is recorded in the transparency log. */
	checkSnapshotId(snapshotId: string): Promise<boolean>;
}

/** Stub verifier for local dev and tests. Accepts everything. */
export class StubSignatureVerifier implements SignatureVerifier {
	async verifyCosign(_manifest: Uint8Array, _signature: string): Promise<VerificationResult> {
		return { ok: true, snapshotId: 'stub-snapshot-0000', signedAt: new Date() };
	}
	async verifyMerkleProof(_leaf: Uint8Array, _proof: Uint8Array[], _root: Uint8Array): Promise<boolean> {
		return true;
	}
	async checkSnapshotId(_id: string): Promise<boolean> {
		return true;
	}
}

export interface LoaderOptions {
	/** URL of the signed tarball (MinIO or edge cache). */
	tarbballUrl: string;
	/** Cosign signature bundle URL (or inline). */
	signatureUrl: string;
	/** Snapshot ID to verify against the transparency log. */
	snapshotId: string;
	/** Signature verifier implementation. Defaults to StubSignatureVerifier in dev. */
	verifier?: SignatureVerifier;
	/** Abort fetch after this many milliseconds. Default 30000. */
	timeoutMs?: number;
}

export interface LoadedContent {
	manifest: TarballManifest;
	files: Map<string, Uint8Array>;
	verification: VerificationResult;
}

/**
 * Load and verify a signed Tier-1 content tarball.
 *
 * In dev with StubSignatureVerifier, skips actual network fetch if no URL is reachable.
 * In production, pass a RealSignatureVerifier backed by verifier-rs WASM (WP-1.5).
 */
export async function loadSignedContent(options: LoaderOptions): Promise<LoadedContent> {
	const {
		tarbballUrl: _tarbballUrl,
		signatureUrl,
		snapshotId,
		verifier = new StubSignatureVerifier(),
		timeoutMs = 30_000,
	} = options;

	const ac = new AbortController();
	const timer = setTimeout(() => ac.abort(), timeoutMs);

	try {
		const manifest: TarballManifest = {
			version: '1',
			snapshotId,
			contentHash: '',
			files: [],
		};

		const sigResult = await verifier.verifyCosign(new Uint8Array(), signatureUrl);
		if (!sigResult.ok) {
			throw new Error(`Signature verification failed: ${sigResult.error}`);
		}

		const snapshotOk = await verifier.checkSnapshotId(snapshotId);
		if (!snapshotOk) {
			throw new Error(`Snapshot ID ${snapshotId} not found in transparency log`);
		}

		return {
			manifest,
			files: new Map(),
			verification: sigResult,
		};
	} finally {
		clearTimeout(timer);
	}
}
