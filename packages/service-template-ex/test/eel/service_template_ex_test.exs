defmodule Eel.ServiceTemplateExTest do
  use ExUnit.Case, async: true

  alias Eel.ServiceTemplateEx.Templates

  describe "Templates.mix_exs_template/1" do
    test "contains the service name" do
      result = Templates.mix_exs_template("my_service")
      assert String.contains?(result, "my_service")
    end

    test "contains the camelized module name" do
      result = Templates.mix_exs_template("my_service")
      assert String.contains?(result, "MyService")
    end
  end

  describe "Templates.router_template/1" do
    test "contains the router module name" do
      result = Templates.router_template("my_service")
      assert String.contains?(result, "MyServiceWeb.Router")
    end

    test "contains health route" do
      result = Templates.router_template("my_service")
      assert String.contains?(result, "/health")
    end

    test "contains ready route" do
      result = Templates.router_template("my_service")
      assert String.contains?(result, "/ready")
    end
  end

  describe "Templates.endpoint_template/1" do
    test "contains the endpoint module name" do
      result = Templates.endpoint_template("ballot_counting")
      assert String.contains?(result, "BallotCountingWeb.Endpoint")
    end
  end

  describe "Templates.k8s_service_template/1" do
    test "contains the service name" do
      result = Templates.k8s_service_template("voter_registry")
      assert String.contains?(result, "voter_registry")
    end
  end

  describe "Mix.Tasks.Eel.Generate" do
    test "task module is compiled" do
      assert Code.ensure_loaded?(Mix.Tasks.Eel.Generate)
    end

    test "task has a @shortdoc module attribute" do
      attrs = Mix.Tasks.Eel.Generate.module_info(:attributes)
      shortdoc = Keyword.get(attrs, :shortdoc, [])
      assert shortdoc != []
    end
  end
end
