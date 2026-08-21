import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:uuid/uuid.dart';

import '../sync/offline_queue.dart';
import 'assessment_model.dart';
import 'assessment_repository.dart';

final _assessmentRepoProvider = Provider((ref) {
  return AssessmentRepository(
    dio: Dio(BaseOptions(baseUrl: 'https://training.eel.iebc.go.ke')),
    queue: OfflineQueue(),
  );
});

class AssessmentScreen extends HookConsumerWidget {
  final String moduleId;

  const AssessmentScreen({super.key, required this.moduleId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final submitted = ValueNotifier<bool>(false);

    return Scaffold(
      appBar: AppBar(title: const Text('Assessment')),
      body: ValueListenableBuilder<bool>(
        valueListenable: submitted,
        builder: (context, isDone, _) {
          if (isDone) {
            return const Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.check_circle, color: Colors.green, size: 64),
                  SizedBox(height: 16),
                  Text('Answers saved. Will sync when online.'),
                ],
              ),
            );
          }
          return _AssessmentForm(
            moduleId: moduleId,
            onSubmit: (workerAnswers) async {
              final repo = ref.read(_assessmentRepoProvider);
              final attempt = AssessmentAttempt(
                id: const Uuid().v4(),
                workerId: 'local-worker',
                moduleId: moduleId,
                answers: workerAnswers,
                idempotencyKey: '${moduleId}_${DateTime.now().millisecondsSinceEpoch}',
              );

              await repo.saveAttemptLocally(attempt);

              try {
                await repo.submitAttempt(attempt);
              } on DioException {
                // Offline or server error: already saved locally, sync later
              }

              submitted.value = true;
            },
          );
        },
      ),
    );
  }
}

class _AssessmentForm extends StatefulWidget {
  final String moduleId;
  final Future<void> Function(Map<String, String> answers) onSubmit;

  const _AssessmentForm({required this.moduleId, required this.onSubmit});

  @override
  State<_AssessmentForm> createState() => _AssessmentFormState();
}

class _AssessmentFormState extends State<_AssessmentForm> {
  final _answers = <String, String>{};
  bool _submitting = false;

  // Placeholder questions: real app loads from cached module content
  static const _questions = [
    ('q1', 'What is the opening time for polls?'),
    ('q2', 'How many streams are allowed per polling station?'),
    ('q3', 'What form is used to record votes?'),
  ];

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          ..._questions.map((q) => _QuestionTile(
                questionId: q.$1,
                questionText: q.$2,
                onAnswer: (answer) => _answers[q.$1] = answer,
              )),
          const SizedBox(height: 24),
          FilledButton(
            onPressed: _submitting
                ? null
                : () async {
                    setState(() => _submitting = true);
                    await widget.onSubmit(_answers);
                    if (mounted) setState(() => _submitting = false);
                  },
            child: _submitting
                ? const CircularProgressIndicator()
                : const Text('Submit Assessment'),
          ),
        ],
      ),
    );
  }
}

class _QuestionTile extends StatefulWidget {
  final String questionId;
  final String questionText;
  final void Function(String answer) onAnswer;

  const _QuestionTile({
    required this.questionId,
    required this.questionText,
    required this.onAnswer,
  });

  @override
  State<_QuestionTile> createState() => _QuestionTileState();
}

class _QuestionTileState extends State<_QuestionTile> {
  final _controller = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(widget.questionText,
              style: Theme.of(context).textTheme.bodyLarge),
          const SizedBox(height: 8),
          TextField(
            controller: _controller,
            onChanged: widget.onAnswer,
            decoration: const InputDecoration(
              border: OutlineInputBorder(),
              hintText: 'Your answer',
            ),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }
}
