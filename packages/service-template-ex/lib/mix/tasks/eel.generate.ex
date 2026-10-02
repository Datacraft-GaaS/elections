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

defmodule Mix.Tasks.Eel.Generate do
  @shortdoc "Scaffolds a Phoenix + LiveView the Commission IEMS service"

  @moduledoc """
  Generates a complete Phoenix + LiveView service skeleton for a Commission IEMS microservice.

  ## Usage

      mix eel.generate <service_name>

  `service_name` must be snake_case (e.g. `ballot_counting`).

  ## Generated structure

      services/<service_name>/ex/
        mix.exs
        lib/<service_name>_web.ex
        lib/<service_name>_web/router.ex
        lib/<service_name>_web/endpoint.ex
        lib/<service_name>_web/plugs/auth_plug.ex
        lib/<service_name>_web/channels/broadway_pipeline.ex
        priv/repo/migrations/001_create_outbox.exs
        priv/repo/migrations/002_create_idempotency.exs
        priv/repo/migrations/003_create_dedup.exs
        deploy/service.yaml
        .woodpecker/<service_name>.yaml
        docs/README.md
  """

  use Mix.Task

  alias Eel.ServiceTemplateEx.Templates

  @impl Mix.Task
  def run([service_name]) do
    validate_name!(service_name)

    base = Path.join(["services", service_name, "ex"])

    files = [
      {Path.join(base, "mix.exs"), Templates.mix_exs_template(service_name)},
      {Path.join([base, "lib", "#{service_name}_web.ex"]), Templates.app_module_template(service_name)},
      {Path.join([base, "lib", "#{service_name}_web", "router.ex"]), Templates.router_template(service_name)},
      {Path.join([base, "lib", "#{service_name}_web", "endpoint.ex"]), Templates.endpoint_template(service_name)},
      {Path.join([base, "lib", "#{service_name}_web", "plugs", "auth_plug.ex"]),
       Templates.auth_plug_template(service_name)},
      {Path.join([base, "lib", "#{service_name}_web", "channels", "broadway_pipeline.ex"]),
       Templates.broadway_pipeline_template(service_name)},
      {Path.join([base, "priv", "repo", "migrations", "001_create_outbox.exs"]),
       Templates.outbox_migration_template(service_name)},
      {Path.join([base, "priv", "repo", "migrations", "002_create_idempotency.exs"]),
       Templates.idempotency_migration_template(service_name)},
      {Path.join([base, "priv", "repo", "migrations", "003_create_dedup.exs"]),
       Templates.dedup_migration_template(service_name)},
      {Path.join([base, "deploy", "service.yaml"]), Templates.k8s_service_template(service_name)},
      {Path.join([base, ".woodpecker", "#{service_name}.yaml"]), Templates.woodpecker_ci_template(service_name)},
      {Path.join([base, "docs", "README.md"]), Templates.readme_template(service_name)}
    ]

    Enum.each(files, fn {path, content} ->
      path |> Path.dirname() |> File.mkdir_p!()
      File.write!(path, content)
      Mix.shell().info("Created #{path}")
    end)

    Mix.shell().info("\nService #{service_name} scaffolded. Run:")
    Mix.shell().info("  cd #{base} && mix deps.get && mix ecto.create && mix ecto.migrate && mix phx.server")
  end

  def run(_) do
    Mix.raise("Usage: mix eel.generate <service_name>")
  end

  defp validate_name!(name) do
    unless Regex.match?(~r/^[a-z][a-z0-9_]*$/, name) do
      Mix.raise("service_name must be snake_case (e.g. ballot_counting). Got: #{inspect(name)}")
    end
  end
end
