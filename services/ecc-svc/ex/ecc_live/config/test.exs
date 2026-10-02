# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import Config

config :ecc_live, EccLive.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4013],
  secret_key_base: "test-secret-key-base-ecc-live-at-least-64-chars-xxxxxxxxxxxxxxxxxxxxxxxxxx",
  server: false

config :ecc_live, EccLive.Repo,
  database: "ecc_live_test#{System.get_env("MIX_TEST_PARTITION")}",
  pool: Ecto.Adapters.SQL.Sandbox

config :logger, level: :warning
config :phoenix, :plug_init_mode, :runtime
