// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// HistoryScreen: paginated list of captures with status badges.
///
/// Data comes from GET /v1/cvr/field-notifications via [ApiClient]. When the
/// device is offline or the API errors out, previously loaded entries stay
/// visible and a retry affordance is shown.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../config.dart';
import '../services/api_client.dart';
import '../services/app_identity.dart';

const int _defaultPageSize = 20;

class HistoryScreen extends ConsumerStatefulWidget {
  const HistoryScreen({super.key});

  @override
  ConsumerState<HistoryScreen> createState() => _HistoryScreenState();
}

class _HistoryScreenState extends ConsumerState<HistoryScreen> {
  final List<Map<String, dynamic>> _captures = [];
  bool _isLoading = false;
  bool _hasMore = true;
  bool _didInitialLoad = false;
  int _page = 0;
  String? _errorMessage;

  @override
  void initState() {
    super.initState();
    _loadNextPage();
  }

  Future<void> _loadNextPage() async {
    if (_isLoading || !_hasMore) return;
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    try {
      // The SVID header is derived from the kit-sdk attestation verdict.
      await AppIdentity.instance.ensureAttested();
      final client = ApiClient(
        baseUri: AppConfig.apiBaseUri,
        svidProvider: () => AppIdentity.instance.spiffeId,
      );
      final nextPage = _page + 1;
      final results = await client.listCaptures(
        officerId: AppIdentity.instance.officerId,
        page: nextPage,
        pageSize: _defaultPageSize,
      );
      if (!mounted) return;
      setState(() {
        _captures.addAll(results);
        _page = nextPage;
        _hasMore = results.length >= _defaultPageSize;
        _isLoading = false;
        _didInitialLoad = true;
      });
    } on Exception catch (e) {
      if (!mounted) return;
      setState(() {
        _errorMessage =
            'Could not load captures ($e). Pull down or tap retry once you '
            'are connected.';
        _isLoading = false;
        _hasMore = false;
        _didInitialLoad = true;
      });
    }
  }

  Future<void> _retryFromStart() async {
    setState(() {
      _captures.clear();
      _page = 0;
      _hasMore = true;
    });
    await _loadNextPage();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Capture History')),
      body: SafeArea(
        child: !_didInitialLoad
            ? const Center(child: CircularProgressIndicator())
            : RefreshIndicator(
                onRefresh: _retryFromStart,
                child: _buildBody(context),
              ),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    if (_captures.isEmpty && _errorMessage != null) {
      return ListView(
        children: [
          Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              children: [
                Semantics(
                  liveRegion: true,
                  child: Text(
                    _errorMessage!,
                    textAlign: TextAlign.center,
                  ),
                ),
                const SizedBox(height: 12),
                FilledButton.icon(
                  icon: const Icon(Icons.refresh),
                  label: const Text('Retry'),
                  onPressed: _isLoading ? null : _retryFromStart,
                ),
              ],
            ),
          ),
        ],
      );
    }

    if (_captures.isEmpty) {
      return ListView(children: const [
        Padding(
          padding: EdgeInsets.all(24),
          child: Center(child: Text('No captures yet.')),
        ),
      ]);
    }

    return ListView.builder(
      itemCount: _captures.length + (_hasMore ? 1 : 0),
      itemBuilder: (context, index) {
        if (index >= _captures.length) {
          // Sentinel row: loads the next page when scrolled into view. The
          // fetch is deferred out of the build phase.
          WidgetsBinding.instance.addPostFrameCallback(
            (_) => _loadNextPage(),
          );
          return const Padding(
            padding: EdgeInsets.symmetric(vertical: 16),
            child: Center(child: CircularProgressIndicator()),
          );
        }
        final capture = _captures[index];
        return Semantics(
          label:
              '${_modalityLabel(capture)}, ${_verdictLabel(capture)}',
          child: ListTile(
            leading: Icon(_modalityIcon(capture)),
            title: Text(_modalityLabel(capture)),
            subtitle: Text(_capturedAtLabel(capture)),
            trailing: _StatusBadge(status: _verdictLabel(capture)),
          ),
        );
      },
    );
  }

  String _modalityLabel(Map<String, dynamic> capture) =>
      switch (capture['modality'] as String?) {
        'DEATH_CERTIFICATE' || 'deathCertificate' => 'Death Certificate',
        'ID_CORRECTION' || 'idCorrection' => 'ID Correction',
        _ => 'Capture',
      };

  IconData _modalityIcon(Map<String, dynamic> capture) =>
      switch (capture['modality'] as String?) {
        'DEATH_CERTIFICATE' || 'deathCertificate' => Icons.article_outlined,
        'ID_CORRECTION' || 'idCorrection' => Icons.badge_outlined,
        _ => Icons.description_outlined,
      };

  /// Server-side review status; falls back through several key spellings so
  /// minor contract revisions do not blank out the badges.
  String _verdictLabel(Map<String, dynamic> capture) {
    final raw = (capture['verdict'] ?? capture['status'] ?? 'pending')
        .toString()
        .toLowerCase();
    return switch (raw) {
      'accepted' => 'Accepted',
      'rejected' => 'Rejected',
      'manual_review' || 'manualReview' || 'manual' => 'Manual Review',
      _ => 'Pending',
    };
  }

  String _capturedAtLabel(Map<String, dynamic> capture) {
    final raw = capture['capturedAt'] as String?;
    if (raw == null) return '';
    return DateTime.tryParse(raw)?.toLocal().toString() ?? raw;
  }
}

class _StatusBadge extends StatelessWidget {
  final String status;

  const _StatusBadge({required this.status});

  @override
  Widget build(BuildContext context) {
    final (label, color) = switch (status) {
      'Accepted' => ('Accepted', Colors.green),
      'Rejected' => ('Rejected', Colors.red),
      'Manual Review' => ('Manual', Colors.blue),
      _ => ('Pending', Colors.orange),
    };

    return Chip(
      label: Text(
        label,
        style: const TextStyle(fontSize: 11, color: Colors.white),
      ),
      backgroundColor: color,
      padding: EdgeInsets.zero,
    );
  }
}
