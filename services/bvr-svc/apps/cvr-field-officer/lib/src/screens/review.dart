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

/// ReviewScreen: pre-submit sanity check showing all extracted fields,
/// image thumbnail, geo coordinates, before final submission.
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../config.dart';
import '../models/capture.dart';
import '../services/api_client.dart';
import '../services/app_identity.dart';
import '../services/offline_queue.dart';

class ReviewScreen extends ConsumerStatefulWidget {
  final CaptureRecord record;

  const ReviewScreen({super.key, required this.record});

  @override
  ConsumerState<ReviewScreen> createState() => _ReviewScreenState();
}

class _ReviewScreenState extends ConsumerState<ReviewScreen> {
  bool _isSubmitting = false;
  String? _errorMessage;

  @override
  Widget build(BuildContext context) {
    final r = widget.record;

    return Scaffold(
      appBar: AppBar(title: const Text('Review Capture')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Image thumbnail
              if (File(r.imagePath).existsSync())
                ClipRRect(
                  borderRadius: BorderRadius.circular(8),
                  child: Image.file(
                    File(r.imagePath),
                    height: 200,
                    width: double.infinity,
                    fit: BoxFit.cover,
                    semanticLabel: 'Captured certificate image',
                  ),
                ),

              const SizedBox(height: 16),

              // Geo
              _Section(
                title: 'Location',
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _Field('Latitude', r.geoLat.toStringAsFixed(6)),
                    _Field('Longitude', r.geoLon.toStringAsFixed(6)),
                    _Field('Accuracy', '${r.geoAccuracyM.toStringAsFixed(1)} m'),
                  ],
                ),
              ),

              const SizedBox(height: 12),

              // OCR extracted fields
              _Section(
                title: 'Extracted Fields',
                child: r.ocrResult == null
                    ? const Text('No OCR result (will be extracted server-side)')
                    : _ExtractedFieldsView(fields: r.ocrResult!),
              ),

              const SizedBox(height: 12),

              // Integrity
              _Section(
                title: 'Integrity',
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _Field('Cert SHA-256', '${r.certSha256.substring(0, 12)}...'),
                    _Field('HSM Sig', '${r.officerHsmSignature.substring(0, 12)}...'),
                    _Field('Confidence', r.ocrConfidence?.toStringAsFixed(2) ?? 'N/A'),
                  ],
                ),
              ),

              const SizedBox(height: 24),

              if (_errorMessage != null)
                Semantics(
                  liveRegion: true,
                  child: Card(
                    color: Theme.of(context).colorScheme.errorContainer,
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Text(
                        _errorMessage!,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.onErrorContainer,
                        ),
                      ),
                    ),
                  ),
                ),

              SizedBox(
                width: double.infinity,
                height: 56,
                child: FilledButton.icon(
                  icon: _isSubmitting
                      ? const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.upload),
                  label: const Text('Submit Capture'),
                  onPressed: _isSubmitting ? null : _submit,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _submit() async {
    setState(() {
      _isSubmitting = true;
      _errorMessage = null;
    });

    try {
      // The capture screen attests before navigation; require the verdict so
      // the SVID header reflects the real device identity.
      await AppIdentity.instance.ensureAttested();
      final client = ApiClient(
        baseUri: AppConfig.apiBaseUri,
        svidProvider: () => AppIdentity.instance.spiffeId,
      );

      await client.submitCapture(widget.record);
      if (!mounted) return;
      Navigator.of(context).popUntil((r) => r.isFirst);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Capture submitted.')),
      );
    } on ApiException {
      // Offline or server error: queue for later
      await OfflineQueueManager().enqueue(widget.record);
      if (!mounted) return;
      Navigator.of(context).popUntil((r) => r.isFirst);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Saved offline. Will sync when connected.')),
      );
    } on Exception catch (e) {
      setState(() => _errorMessage = e.toString());
    } finally {
      if (mounted) setState(() => _isSubmitting = false);
    }
  }
}

class _Section extends StatelessWidget {
  final String title;
  final Widget child;

  const _Section({required this.title, required this.child});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Semantics(
          header: true,
          child: Text(title, style: Theme.of(context).textTheme.titleSmall),
        ),
        const SizedBox(height: 8),
        child,
        const Divider(),
      ],
    );
  }
}

class _Field extends StatelessWidget {
  final String label;
  final String value;

  const _Field(this.label, this.value);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          SizedBox(
            width: 140,
            child: Text(label, style: Theme.of(context).textTheme.bodySmall),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }
}

class _ExtractedFieldsView extends StatelessWidget {
  final ExtractedFields fields;

  const _ExtractedFieldsView({required this.fields});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (fields.deceasedName != null)
          _Field('Deceased Name', fields.deceasedName!),
        if (fields.fullName != null)
          _Field('Full Name', fields.fullName!),
        if (fields.nationalId != null)
          _Field('National ID', fields.nationalId!),
        if (fields.dateOfDeath != null)
          _Field('Date of Death', fields.dateOfDeath!),
        if (fields.dateOfBirth != null)
          _Field('Date of Birth', fields.dateOfBirth!),
        if (fields.placeOfDeath != null)
          _Field('Place of Death', fields.placeOfDeath!),
        if (fields.certSerial != null)
          _Field('Cert Serial', fields.certSerial!),
        if (fields.gender != null)
          _Field('Gender', fields.gender!),
        if (fields.county != null)
          _Field('County', fields.county!),
      ],
    );
  }
}
