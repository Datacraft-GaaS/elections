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
 * Client-side SHA-256 verification using WebCrypto and
 * RFC 6962 Merkle inclusion-proof verification.
 */

/**
 * Compute SHA-256 of bytes and return lowercase hex string.
 */
export async function sha256Hex(data: ArrayBuffer): Promise<string> {
  const hashBuf = await crypto.subtle.digest("SHA-256", data);
  return Array.from(new Uint8Array(hashBuf))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * Verify a file's SHA-256 matches the declared hash.
 * Returns true if match, false if tampered.
 */
export async function verifyArtifact(
  file: File,
  declaredHex: string
): Promise<boolean> {
  const buf = await file.arrayBuffer();
  const actual = await sha256Hex(buf);
  return actual.toLowerCase() === declaredHex.toLowerCase();
}

/**
 * RFC 6962 Merkle inclusion-proof verification.
 *
 * @param leafHash  SHA-256 of the leaf value (hex)
 * @param proof     Array of sibling hashes (hex), ordered from leaf to root
 * @param leafIndex Leaf index in the tree
 * @param rootHash  Expected root hash (hex)
 */
export async function verifyInclusionProof(
  leafHash: string,
  proof: string[],
  leafIndex: number,
  rootHash: string
): Promise<boolean> {
  async function hashLeaf(data: Uint8Array): Promise<Uint8Array> {
    const prefixed = new Uint8Array(data.length + 1);
    prefixed[0] = 0x00; // RFC 6962 leaf hash prefix
    prefixed.set(data, 1);
    const hash = await crypto.subtle.digest("SHA-256", prefixed);
    return new Uint8Array(hash);
  }

  async function hashNode(left: Uint8Array, right: Uint8Array): Promise<Uint8Array> {
    const combined = new Uint8Array(left.length + right.length + 1);
    combined[0] = 0x01; // RFC 6962 internal node hash prefix
    combined.set(left, 1);
    combined.set(right, 1 + left.length);
    const hash = await crypto.subtle.digest("SHA-256", combined);
    return new Uint8Array(hash);
  }

  function hexToBytes(hex: string): Uint8Array {
    const arr = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
      arr[i / 2] = parseInt(hex.substring(i, 2), 16);
    }
    return arr;
  }

  let current = hexToBytes(leafHash);
  let idx = leafIndex;

  for (const sibling of proof) {
    const sibBytes = hexToBytes(sibling);
    if (idx % 2 === 0) {
      current = await hashNode(current, sibBytes);
    } else {
      current = await hashNode(sibBytes, current);
    }
    idx = Math.floor(idx / 2);
  }

  const computedRoot = Array.from(current)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");

  return computedRoot.toLowerCase() === rootHash.toLowerCase();
}
