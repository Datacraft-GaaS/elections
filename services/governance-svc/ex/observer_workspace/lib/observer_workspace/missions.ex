defmodule ObserverWorkspace.Missions do
  @moduledoc """
  Context for observer missions and mission-scoped RBAC.
  Each mission has an isolated workspace; cross-mission access is denied.
  """

  require Logger

  @type mission_id :: String.t()
  @type user_role :: :mission_lead | :mission_analyst | :mission_observer | :eel_admin

  @doc """
  Verify that the given user role is permitted to access the given mission workspace.
  EEL admins may access all missions; mission members are scoped to their own workspace.
  """
  @spec authorize(String.t(), mission_id(), user_role()) :: :ok | {:error, :forbidden}
  def authorize(user_mission_id, workspace_mission_id, role) do
    case role do
      :eel_admin ->
        :ok
      _ when user_mission_id == workspace_mission_id ->
        :ok
      _ ->
        Logger.warning("authorization denied: user_mission=#{user_mission_id} workspace=#{workspace_mission_id} role=#{role}")
        {:error, :forbidden}
    end
  end

  @doc """
  List all missions (stub; real impl queries the Rust crate API or shared DB).
  """
  @spec list_missions() :: list(map())
  def list_missions do
    [
      %{id: "eu-eom-2027", organisation: "EU Election Observation Mission", status: "active"},
      %{id: "au-observer-2027", organisation: "African Union Observer Mission", status: "active"},
      %{id: "carter-center-2027", organisation: "Carter Center", status: "active"},
      %{id: "ifes-2027", organisation: "IFES", status: "registered"},
      %{id: "eac-2027", organisation: "East African Community Observer Mission", status: "registered"}
    ]
  end

  @doc """
  Get briefings for a mission workspace.
  """
  @spec list_briefings(mission_id()) :: list(map())
  def list_briefings(mission_id) do
    Logger.info("list_briefings mission_id=#{mission_id}")
    []
  end

  @doc """
  Get observation reports for a mission workspace.
  """
  @spec list_reports(mission_id()) :: list(map())
  def list_reports(mission_id) do
    Logger.info("list_reports mission_id=#{mission_id}")
    []
  end
end
