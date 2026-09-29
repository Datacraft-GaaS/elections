/// Outbox panel — UI component for operator-controlled biometric upload.
/// Shows count of pending/failed/uploaded/deduplicated submissions.
/// Operator taps "Upload now" to drain the queue (when network available).
import 'package:eel_core_flutter/eel_core_flutter.dart';
import 'package:eel_design/eel_design.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class BiometricOutboxPanel extends ConsumerStatefulWidget {
	const BiometricOutboxPanel({super.key, required this.outbox, required this.authToken});

	final BiometricSubmissionOutbox outbox;
	final String authToken;

	@override
	ConsumerState<BiometricOutboxPanel> createState() => _BiometricOutboxPanelState();
}

class _BiometricOutboxPanelState extends ConsumerState<BiometricOutboxPanel> {
	OutboxStatus? _status;
	bool _uploading = false;
	String? _lastResult;

	@override
	void initState() {
		super.initState();
		_refresh();
	}

	Future<void> _refresh() async {
		final s = await widget.outbox.status();
		setState(() => _status = s);
	}

	Future<void> _uploadNow() async {
		setState(() {
			_uploading = true;
			_lastResult = null;
		});
		try {
			final r = await widget.outbox.uploadAll(authToken: widget.authToken);
			setState(() {
				_lastResult = 'Uploaded ${r.uploaded}/${r.total}, '
					'failed ${r.failed}, server-deduplicated ${r.deduplicated}';
			});
			await _refresh();
		} catch (e) {
			setState(() => _lastResult = 'Upload error: $e');
		} finally {
			setState(() => _uploading = false);
		}
	}

	@override
	Widget build(BuildContext context) {
		final s = _status;
		return EelScaffold(
			title: 'Biometric Outbox',
			variant: EelScaffoldVariant.scrollable,
			padding: const EdgeInsets.all(16),
			body: s == null
				? const EelLoadingState()
				: Column(
					crossAxisAlignment: CrossAxisAlignment.stretch,
					children: [
						Text('Outbox Status', style: EelTextStyle.titleLarge),
						const SizedBox(height: 16),
						EelResultCard(
							leading: Icon(Icons.outbox, color: EelColors.of(context).primary, size: 32),
							title: Text('Total submissions: ${s.total}', style: EelTextStyle.titleMedium),
						),
						const SizedBox(height: 12),
						Row(children: [
							Expanded(child: EelStatusChip(label: 'Pending: ${s.pendingUpload}', severity: EelSeverity.info)),
							const SizedBox(width: 8),
							Expanded(child: EelStatusChip(label: 'Uploaded: ${s.uploaded}', severity: EelSeverity.success)),
						]),
						const SizedBox(height: 8),
						Row(children: [
							Expanded(child: EelStatusChip(label: 'Failed: ${s.failed}', severity: EelSeverity.danger)),
							const SizedBox(width: 8),
							Expanded(child: EelStatusChip(label: 'Deduplicated: ${s.deduplicated}', severity: EelSeverity.warning)),
						]),
						const SizedBox(height: 24),
						if (s.pendingUpload > 0)
							EelBanner(
								message: '${s.pendingUpload} biometric submission(s) awaiting upload. Connect to network and tap "Upload now".',
								severity: EelSeverity.info,
							),
						if (s.failed > 0)
							EelBanner(
								message: '${s.failed} submission(s) failed upload. Will retry up to 5 times.',
								severity: EelSeverity.danger,
							),
						if (s.deduplicated > 0)
							EelBanner(
								message: '${s.deduplicated} submission(s) deduplicated (same biometric already on this device).',
								severity: EelSeverity.warning,
							),
						const SizedBox(height: 24),
						EelButton(
							label: _uploading ? 'Uploading...' : 'Upload now',
							icon: Icons.cloud_upload,
							fullWidth: true,
							onPressed: (_uploading || s.pendingUpload == 0) ? null : _uploadNow,
						),
						const SizedBox(height: 12),
						EelButton(
							label: 'Refresh',
							icon: Icons.refresh,
							variant: EelButtonVariant.outlined,
							fullWidth: true,
							onPressed: _uploading ? null : _refresh,
						),
						if (_lastResult != null) ...[
							const SizedBox(height: 16),
							EelBanner(message: _lastResult!, severity: EelSeverity.info),
						],
					],
				),
		);
	}
}
