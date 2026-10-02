/*
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

/*
 * Stub implementations of OpenSSL ENGINE API symbols removed in OpenSSL 3.x.
 * librdkafka references these when compiled with ssl support.
 * On macOS with OpenSSL 3.x these symbols are absent; this file provides no-op stubs
 * so the linker can resolve them at dev time.
 * Production Linux builds use OpenSSL 1.1 compat packages that include the real symbols.
 */
#include <stddef.h>
#include <stdint.h>

typedef struct engine_st ENGINE;

ENGINE *ENGINE_by_id(const char *id) { (void)id; return NULL; }
int ENGINE_ctrl_cmd_string(ENGINE *e, const char *cmd_name, const char *arg, int cmd_optional) {
    (void)e; (void)cmd_name; (void)arg; (void)cmd_optional; return 0;
}
int ENGINE_free(ENGINE *e) { (void)e; return 1; }
int ENGINE_init(ENGINE *e) { (void)e; return 0; }
int ENGINE_load_ssl_client_cert(ENGINE *e, void *ssl, void *ca_dn, void *pcert, void *ppkey, void *extra_certs, void *cb, void *cb_data) {
    (void)e; (void)ssl; (void)ca_dn; (void)pcert; (void)ppkey; (void)extra_certs; (void)cb; (void)cb_data;
    return 0;
}
