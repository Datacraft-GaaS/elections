/// Voter Lookup — Beautiful, ergonomic, accessible.
library;
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:eel_design/eel_design.dart';

class VoterLookupScreen extends ConsumerStatefulWidget {
  const VoterLookupScreen({super.key});

  @override
  ConsumerState<VoterLookupScreen> createState() => _VoterLookupScreenState();
}

class _VoterLookupScreenState extends ConsumerState<VoterLookupScreen> {
  final _query = TextEditingController();

  @override
  void dispose() {
    _query.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return EelScaffold(
      title: 'Voter Lookup',
      variant: EelScaffoldVariant.scrollable,
      padding: const EdgeInsets.all(24),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Text('Voter Lookup', style: EelTextStyle.titleLarge),
          const SizedBox(height: 8),
          EelTextField(
            label: 'Search voter',
            hintText: 'Enter voter ID or name',
            controller: _query,
          ),
          const SizedBox(height: 24),
          // This screen used to render a fixed card reading "Voter Verified —
          // Station KE-NBI-001 • Registered" above a dead Confirm Lookup button,
          // so a poll clerk saw a verified voter who had never been looked up.
          //
          // It is not wired to the BFF's /voter/verification because that
          // handler does not verify: it defaults the voter id to
          // "VOTER-12345" and returns {"verified": true} for any input, with
          // or without a query parameter. Calling it would render a real-looking
          // verification from a source that always agrees, which is worse than
          // saying so. Until the handler consults the register, this says that
          // rather than inventing a result.
          const EelBanner(
            message:
                'Voter verification is unavailable. The lookup endpoint is not '
                'yet wired to the register, so it cannot confirm who a voter is.',
            severity: EelSeverity.warning,
          ),
          const SizedBox(height: 16),
          EelButton(
            label: 'Confirm Lookup',
            fullWidth: true,
            onPressed: () => ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text(
                  'Verification unavailable: the register lookup backend is not implemented.',
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
