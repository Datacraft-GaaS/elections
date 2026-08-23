defmodule BvvWeb.ObjectionsTest do
  @moduledoc "Unit tests for BvvWeb.Objections context changesets."
  use ExUnit.Case, async: true

  alias BvvWeb.Objections
  alias BvvWeb.Objections.Objection

  @valid_attrs %{
    objector_session_hash: String.duplicate("a", 64),
    target_voter_id: "21223344556",
    objection_type: "rectification",
    justification: "The register lists a voter who moved out of this polling station."
  }

  # ---------------------------------------------------------------------------
  # Module-level sanity
  # ---------------------------------------------------------------------------

  test "context modules are loaded" do
    assert Code.ensure_loaded?(Objections)
    assert Code.ensure_loaded?(Objection)
  end

  # ---------------------------------------------------------------------------
  # Changeset behaviour
  # ---------------------------------------------------------------------------

  test "changeset with valid attributes is valid" do
    changeset = Objection.changeset(%Objection{}, @valid_attrs)

    assert changeset.valid?
  end

  test "changeset requires objector hash, voter id, type and justification" do
    changeset = Objection.changeset(%Objection{}, %{})
    errors = errors_on(changeset)

    for field <- ~w(objector_session_hash target_voter_id objection_type justification)a do
      assert errors[field], "expected :#{field} to be required"
    end
  end

  test "changeset rejects unknown objection types" do
    changeset = Objection.changeset(%Objection{}, Map.put(@valid_attrs, :objection_type, "harassment"))

    assert %{objection_type: ["is invalid"]} = errors_on(changeset)
  end

  test "changeset accepts all three contract objection types" do
    for type <- Objection.objection_types() do
      changeset = Objection.changeset(%Objection{}, Map.put(@valid_attrs, :objection_type, type))
      assert changeset.valid?, "expected #{type} to be accepted"
    end
  end

  test "changeset enforces minimum justification length" do
    changeset = Objection.changeset(%Objection{}, Map.put(@valid_attrs, :justification, "too short"))

    assert %{justification: [msg]} = errors_on(changeset)
    assert msg =~ "at least"
  end

  test "changeset leaves evidence_ref optional" do
    changeset =
      Objection.changeset(%Objection{}, Map.put(@valid_attrs, :evidence_ref, "photo-0012.jpg"))

    assert changeset.valid?
  end

  test "new objections default to pending status" do
    assert %Objection{status: "pending"} = struct(Objection)
  end

  defp errors_on(changeset) do
    Ecto.Changeset.traverse_errors(changeset, fn {msg, _opts} -> msg end)
  end
end
