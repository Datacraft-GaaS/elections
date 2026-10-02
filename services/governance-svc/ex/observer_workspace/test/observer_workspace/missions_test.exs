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
