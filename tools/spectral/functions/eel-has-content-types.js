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
 * eel-has-content-types
 *
 * Spectral custom function: checks that Tier 2 and Tier 5 endpoints declare
 * both application/json and application/cbor in their response content map
 * per ADR-0011 (content negotiation for mobile CBOR).
 *
 * Usage in ruleset:
 *   then:
 *     function: eel-has-content-types
 */
export default function eelHasContentTypes(responses) {
  const results = [];

  if (!responses || typeof responses !== "object") return results;

  for (const [statusCode, response] of Object.entries(responses)) {
    if (!statusCode.startsWith("2")) continue;
    const content = response?.content;
    if (!content) {
      results.push({
        message: `Response ${statusCode} declares no content types; expected application/json and application/cbor`,
      });
      continue;
    }
    const types = Object.keys(content);
    if (!types.includes("application/json")) {
      results.push({
        message: `Response ${statusCode} missing application/json content type`,
      });
    }
    if (!types.includes("application/cbor")) {
      results.push({
        message: `Response ${statusCode} missing application/cbor content type (required for Tier 2/5 mobile per ADR-0011)`,
      });
    }
  }

  return results;
}
