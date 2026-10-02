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
  url: [host: "localhost"],
  render_errors: [
    formats: [html: EccLive.ErrorHTML, json: EccLive.ErrorJSON],
    layout: false
  ],
  pubsub_server: EccLive.PubSub,
  live_view: [signing_salt: "eel-ecc-lv-salt"]

config :ecc_live, EccLive.Repo,
  database: "ecc_live_dev",
  username: "postgres",
  password: "postgres",
  hostname: "localhost",
  pool_size: 10

config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :phoenix, :json_library, Jason

import_config "#{config_env()}.exs"
