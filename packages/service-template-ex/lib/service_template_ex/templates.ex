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

defmodule Eel.ServiceTemplateEx.Templates do
  @moduledoc """
  Template string generators for scaffolded Phoenix services.

  Each function returns the content of a generated file for a given service name.
  `service_name` is expected to be snake_case (e.g. `"ballot_counting"`).
  """

  @doc """
  Returns the content of `mix.exs` for a new Phoenix service.
  """
  @spec mix_exs_template(String.t()) :: String.t()
  def mix_exs_template(service_name) do
    mod = Macro.camelize(service_name)

    """
    defmodule #{mod}.MixProject do
      use Mix.Project

      def project do
        [
          app: :#{service_name},
          version: "0.1.0",
          elixir: "~> 1.17",
          start_permanent: Mix.env() == :prod,
          deps: deps()
        ]
      end

      def application do
        [
          mod: {#{mod}.Application, []},
          extra_applications: [:logger, :runtime_tools]
        ]
      end

      defp deps do
        [
          {:phoenix, "~> 1.7"},
          {:phoenix_live_view, "~> 0.20"},
          {:ecto_sql, "~> 3.12"},
          {:postgrex, "~> 0.17"},
          {:jason, "~> 1.4"},
          {:plug_cowboy, "~> 2.7"},
          {:eel_resilience, path: "../../../../packages/eel-resilience-ex"},
          {:eel_observability, path: "../../../../packages/eel-observability-ex"}
        ]
      end
    end
    """
  end

  @doc """
  Returns the content of the Phoenix router for a new service.
  """
  @spec router_template(String.t()) :: String.t()
  def router_template(service_name) do
    mod = Macro.camelize(service_name)

    """
    defmodule #{mod}Web.Router do
      use #{mod}Web, :router

      pipeline :api do
        plug :accepts, ["json"]
      end

      scope "/", #{mod}Web do
        pipe_through :api

        get "/health", HealthController, :index
        get "/ready", HealthController, :ready
      end
    end
    """
  end

  @doc """
  Returns the content of the Phoenix endpoint for a new service.
  """
  @spec endpoint_template(String.t()) :: String.t()
  def endpoint_template(service_name) do
    mod = Macro.camelize(service_name)

    """
    defmodule #{mod}Web.Endpoint do
      use Phoenix.Endpoint, otp_app: :#{service_name}

      plug Plug.RequestId
      plug Plug.Telemetry, event_prefix: [:phoenix, :endpoint]

      plug Plug.Parsers,
        parsers: [:urlencoded, :multipart, :json],
        pass: ["*/*"],
        json_decoder: Phoenix.json_library()

      plug Plug.MethodOverride
      plug Plug.Head
      plug #{mod}Web.Router
    end
    """
  end

  @doc """
  Returns the content of the Phoenix app module for a new service.
  """
  @spec app_module_template(String.t()) :: String.t()
  def app_module_template(service_name) do
    mod = Macro.camelize(service_name)

    """
    defmodule #{mod}Web do
      @moduledoc \"\"\"
      The entrypoint for defining your web interface.
      \"\"\"

      def controller do
        quote do
          use Phoenix.Controller, namespace: #{mod}Web
          import Plug.Conn
        end
      end

      def router do
        quote do
          use Phoenix.Router, helpers: false
          import Plug.Conn
          import Phoenix.Controller
        end
      end

      defmacro __using__(which) when is_atom(which) do
        apply(__MODULE__, which, [])
      end
    end
    """
  end

  @doc """
  Returns the content of the auth plug for a new service.
  """
  @spec auth_plug_template(String.t()) :: String.t()
  def auth_plug_template(service_name) do
    mod = Macro.camelize(service_name)

    """
    defmodule #{mod}Web.Plugs.AuthPlug do
      @moduledoc \"\"\"
      Authentication plug. Delegates to Eel.Auth.Plug when available.
      \"\"\"

      import Plug.Conn

      @behaviour Plug

      @impl Plug
      def init(opts), do: opts

      @impl Plug
      def call(conn, _opts) do
        conn
      end
    end
    """
  end

  @doc """
  Returns the content of a Broadway pipeline template.
  """
  @spec broadway_pipeline_template(String.t()) :: String.t()
  def broadway_pipeline_template(service_name) do
    mod = Macro.camelize(service_name)

    """
    defmodule #{mod}Web.Channels.BroadwayPipeline do
      @moduledoc \"\"\"
      Broadway pipeline template for #{service_name}.
      Replace the producer and processor logic with your domain-specific implementation.
      \"\"\"

      use Broadway

      def start_link(opts) do
        Broadway.start_link(__MODULE__,
          name: __MODULE__,
          producer: [
            module: Keyword.fetch!(opts, :producer)
          ],
          processors: [
            default: [concurrency: 10]
          ]
        )
      end

      @impl Broadway
      def handle_message(_processor, message, _context) do
        message
      end
    end
    """
  end

  @doc """
  Returns the outbox migration SQL for a new service.
  """
  @spec outbox_migration_template(String.t()) :: String.t()
  def outbox_migration_template(service_name) do
    """
    defmodule #{Macro.camelize(service_name)}.Repo.Migrations.CreateOutbox do
      use Ecto.Migration

      def change do
        create table(:outbox, primary_key: false) do
          add :id, :uuid, primary_key: true, default: fragment("gen_random_uuid()")
          add :aggregate_id, :string, null: false
          add :event_type, :string, null: false
          add :payload, :jsonb, null: false
          add :published, :boolean, default: false, null: false

          timestamps(type: :utc_datetime_usec)
        end

        create index(:outbox, [:published, :inserted_at])
      end
    end
    """
  end

  @doc """
  Returns the idempotency migration SQL for a new service.
  """
  @spec idempotency_migration_template(String.t()) :: String.t()
  def idempotency_migration_template(service_name) do
    """
    defmodule #{Macro.camelize(service_name)}.Repo.Migrations.CreateIdempotency do
      use Ecto.Migration

      def change do
        create table(:idempotency_keys, primary_key: false) do
          add :key, :string, primary_key: true
          add :body_hash, :string, null: false
          add :status_code, :integer, null: false
          add :response_body, :text
          add :expires_at, :utc_datetime_usec, null: false

          timestamps(type: :utc_datetime_usec, updated_at: false)
        end

        create index(:idempotency_keys, [:expires_at])
      end
    end
    """
  end

  @doc """
  Returns the event dedup migration SQL for a new service.
  """
  @spec dedup_migration_template(String.t()) :: String.t()
  def dedup_migration_template(service_name) do
    """
    defmodule #{Macro.camelize(service_name)}.Repo.Migrations.CreateDedup do
      use Ecto.Migration

      def change do
        create table(:event_dedup, primary_key: false) do
          add :event_id, :string, primary_key: true
          add :processed_at, :utc_datetime_usec, null: false
        end

        create index(:event_dedup, [:processed_at])
      end
    end
    """
  end

  @doc """
  Returns the Kubernetes service manifest YAML for a new service.
  """
  @spec k8s_service_template(String.t()) :: String.t()
  def k8s_service_template(service_name) do
    """
    apiVersion: v1
    kind: Service
    metadata:
      name: #{service_name}
      labels:
        app: #{service_name}
        part-of: commission-iems
    spec:
      selector:
        app: #{service_name}
      ports:
        - protocol: TCP
          port: 80
          targetPort: 4000
    ---
    apiVersion: apps/v1
    kind: Deployment
    metadata:
      name: #{service_name}
      labels:
        app: #{service_name}
        part-of: commission-iems
    spec:
      replicas: 2
      selector:
        matchLabels:
          app: #{service_name}
      template:
        metadata:
          labels:
            app: #{service_name}
        spec:
          containers:
            - name: #{service_name}
              image: registry.eel.local/#{service_name}:latest
              ports:
                - containerPort: 4000
              readinessProbe:
                httpGet:
                  path: /ready
                  port: 4000
              livenessProbe:
                httpGet:
                  path: /health
                  port: 4000
    """
  end

  @doc """
  Returns the Woodpecker CI config YAML for a new service.
  """
  @spec woodpecker_ci_template(String.t()) :: String.t()
  def woodpecker_ci_template(service_name) do
    """
    when:
      - event: push
        branch: main
      - event: pull_request

    steps:
      - name: test
        image: hexpm/elixir:1.17.3-erlang-27.1-alpine-3.20.3
        environment:
          MIX_ENV: test
        commands:
          - mix deps.get
          - mix test

      - name: build
        image: docker:latest
        commands:
          - docker build -t registry.eel.local/#{service_name}:$CI_COMMIT_SHA .
          - docker push registry.eel.local/#{service_name}:$CI_COMMIT_SHA

      - name: deploy
        image: bitnami/kubectl:latest
        commands:
          - kubectl set image deployment/#{service_name} #{service_name}=registry.eel.local/#{service_name}:$CI_COMMIT_SHA
    """
  end

  @doc """
  Returns the README content for a new service.
  """
  @spec readme_template(String.t()) :: String.t()
  def readme_template(service_name) do
    mod = Macro.camelize(service_name)

    """
    # #{mod}

    the Commission IEMS service generated by `mix eel.generate #{service_name}`.

    ## Running

    ```bash
    mix deps.get
    mix ecto.create
    mix ecto.migrate
    mix phx.server
    ```

    ## Health checks

    - `GET /health` - liveness probe
    - `GET /ready` - readiness probe

    ## Tests

    ```bash
    mix test
    ```
    """
  end
end
