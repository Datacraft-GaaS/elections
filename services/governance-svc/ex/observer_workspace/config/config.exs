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

config :observer_workspace,
  ecto_repos: []

config :observer_workspace, ObserverWorkspaceWeb.Endpoint,
  url: [host: "localhost"],
  adapter: Bandit.PhoenixAdapter,
  render_errors: [
    formats: [html: ObserverWorkspaceWeb.ErrorHTML, json: ObserverWorkspaceWeb.ErrorJSON],
    layout: false
  ],
  pubsub_server: ObserverWorkspace.PubSub,
  live_view: [signing_salt: "eel-obs-lv-salt"]

config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :phoenix, :json_library, Jason

# Swoosh: use Finch adapter (no hackney needed)
config :swoosh, :api_client, Swoosh.ApiClient.Finch
config :swoosh, :finch_name, ObserverWorkspace.Finch

import_config "#{config_env()}.exs"
