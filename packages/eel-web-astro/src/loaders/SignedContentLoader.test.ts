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

import { describe, it, expect } from 'vitest';
import {
	StubSignatureVerifier,
	loadSignedContent,
	type SignatureVerifier,
} from './SignedContentLoader.js';

describe('StubSignatureVerifier', () => {
	const v = new StubSignatureVerifier();

	it('verifyCosign returns ok=true', async () => {
		const result = await v.verifyCosign(new Uint8Array(), 'sig');
		expect(result.ok).toBe(true);
		expect(result.snapshotId).toBeDefined();
	});

	it('verifyMerkleProof returns true', async () => {
		const ok = await v.verifyMerkleProof(new Uint8Array(), [], new Uint8Array());
		expect(ok).toBe(true);
	});

	it('checkSnapshotId returns true', async () => {
		const ok = await v.checkSnapshotId('any-id');
		expect(ok).toBe(true);
	});
});

describe('loadSignedContent', () => {
	it('loads with stub verifier and returns manifest', async () => {
		const result = await loadSignedContent({
			tarbballUrl: 'http://localhost/test.tar.zst',
			signatureUrl: 'stub-sig',
			snapshotId: 'snap-001',
			verifier: new StubSignatureVerifier(),
		});
		expect(result.verification.ok).toBe(true);
		expect(result.manifest.snapshotId).toBe('snap-001');
	});

	it('throws when verifier rejects signature', async () => {
		const badVerifier: SignatureVerifier = {
			async verifyCosign() { return { ok: false, error: 'bad sig' }; },
			async verifyMerkleProof() { return false; },
			async checkSnapshotId() { return true; },
		};
		await expect(loadSignedContent({
			tarbballUrl: 'http://localhost/test.tar.zst',
			signatureUrl: 'bad',
			snapshotId: 'snap-002',
			verifier: badVerifier,
		})).rejects.toThrow('Signature verification failed');
	});

	it('throws when snapshot ID not found', async () => {
		const noSnap: SignatureVerifier = {
			async verifyCosign() { return { ok: true }; },
			async verifyMerkleProof() { return true; },
			async checkSnapshotId() { return false; },
		};
		await expect(loadSignedContent({
			tarbballUrl: 'http://localhost/test.tar.zst',
			signatureUrl: 'sig',
			snapshotId: 'missing-snap',
			verifier: noSnap,
		})).rejects.toThrow('not found in transparency log');
	});
});
