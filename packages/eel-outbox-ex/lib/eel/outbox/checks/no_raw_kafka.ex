if Code.ensure_loaded?(Credo.Check) do
  defmodule Eel.Outbox.Checks.NoRawKafka do
    @moduledoc """
    Credo check that flags direct use of raw Kafka producer libraries.

    The following module/atom references trigger a warning:
    - `:brod`
    - `:kafka_ex`
    - `KafkaEx`
    - `Kafkaex`

    Use `Eel.Outbox.emit/2` instead to guarantee the transactional outbox
    pattern and avoid dual-write hazards.
    """

    use Credo.Check,
      category: :warning,
      base_priority: :high,
      explanations: [
        check: """
        Use Eel.Outbox.emit/2 instead of a raw Kafka producer.

        Calling Kafka directly bypasses the transactional outbox pattern and
        risks publishing events without a matching database commit (dual-write
        hazard). Route all event publishing through Eel.Outbox.emit/2.
        """
      ]

    @kafka_modules [:KafkaEx, :Kafkaex]
    @kafka_atoms [:brod, :kafka_ex]

    @impl Credo.Check
    def run(%Credo.SourceFile{} = source_file, params) do
      issue_meta = IssueMeta.for(source_file, params)
      Credo.Code.prewalk(source_file, &traverse(&1, &2, issue_meta))
    end

    # Match aliases like KafkaEx, Kafkaex (and sub-modules, e.g. KafkaEx.Producer).
    defp traverse(
           {:__aliases__, meta, [module | _rest]} = ast,
           issues,
           issue_meta
         )
         when module in @kafka_modules do
      issue =
        format_issue(issue_meta,
          message: "Use Eel.Outbox.emit/2 instead of a raw Kafka producer.",
          line_no: meta[:line]
        )

      {ast, [issue | issues]}
    end

    # Match Erlang-style calls: :brod.produce(...) or :kafka_ex.produce(...)
    defp traverse(
           {{:., meta, [{atom, _, nil}, _func]}, _call_meta, _args} = ast,
           issues,
           issue_meta
         )
         when atom in @kafka_atoms do
      issue =
        format_issue(issue_meta,
          message: "Use Eel.Outbox.emit/2 instead of a raw Kafka producer.",
          line_no: meta[:line]
        )

      {ast, [issue | issues]}
    end

    defp traverse(ast, issues, _issue_meta), do: {ast, issues}
  end
end
