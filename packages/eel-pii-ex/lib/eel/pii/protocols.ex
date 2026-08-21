defimpl String.Chars, for: Eel.PII.Restricted do
  def to_string(pii), do: Eel.PII.redact(pii)
end

defimpl String.Chars, for: Eel.PII.Sensitive do
  def to_string(pii), do: Eel.PII.redact(pii)
end

defimpl String.Chars, for: Eel.PII.Public do
  def to_string(pii), do: Eel.PII.redact(pii)
end

defimpl Inspect, for: Eel.PII.Restricted do
  def inspect(pii, _opts), do: Eel.PII.redact(pii)
end

defimpl Inspect, for: Eel.PII.Sensitive do
  def inspect(pii, _opts), do: Eel.PII.redact(pii)
end

defimpl Inspect, for: Eel.PII.Public do
  def inspect(pii, _opts), do: Eel.PII.redact(pii)
end
