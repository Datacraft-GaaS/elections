defmodule ObserverWorkspace.MissionsTest do
  use ExUnit.Case, async: true

  alias ObserverWorkspace.Missions

  @tag trace: ["FR-M57-003", "FR-M57-004"]
  test "eel_admin can access any mission workspace" do
    assert :ok == Missions.authorize("eu-eom-2027", "au-observer-2027", :eel_admin)
  end

  @tag trace: ["FR-M57-003"]
  test "mission member can access own workspace" do
    assert :ok == Missions.authorize("eu-eom-2027", "eu-eom-2027", :mission_lead)
  end

  @tag trace: ["FR-M57-004"]
  test "mission member cannot access another mission workspace" do
    assert {:error, :forbidden} ==
             Missions.authorize("eu-eom-2027", "au-observer-2027", :mission_analyst)
  end

  test "list_missions returns known missions" do
    missions = Missions.list_missions()
    assert length(missions) >= 5
    ids = Enum.map(missions, & &1.id)
    assert "eu-eom-2027" in ids
    assert "carter-center-2027" in ids
  end
end
