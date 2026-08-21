defmodule Eel.Observability.AuditShadow do
  @moduledoc """
  Generates PostgreSQL trigger SQL for the audit_shadow pattern.

  Each trigger routes INSERT/UPDATE/DELETE operations on a table into
  the centralized `audit_shadow` table via a stored function.
  """

  @doc """
  Returns SQL that creates an AFTER trigger on `table_name` within `schema`.

  The trigger calls `audit_shadow_fn()` for every row change.

  ## Example

      iex> sql = Eel.Observability.AuditShadow.trigger_sql("voters")
      iex> String.contains?(sql, "TRIGGER")
      true
  """
  @spec trigger_sql(String.t(), String.t()) :: String.t()
  def trigger_sql(table_name, schema \\ "public") when is_binary(table_name) and is_binary(schema) do
    """
    CREATE OR REPLACE TRIGGER audit_#{table_name}
    AFTER INSERT OR UPDATE OR DELETE ON #{schema}.#{table_name}
    FOR EACH ROW EXECUTE FUNCTION audit_shadow_fn();
    """
  end
end
