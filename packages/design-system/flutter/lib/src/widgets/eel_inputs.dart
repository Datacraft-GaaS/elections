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

import 'package:flutter/material.dart';

import '../theme/eel_theme.dart';

/// Select (dropdown) with always-visible label, matching EelTextField.
class EelSelect<T> extends StatelessWidget {
  const EelSelect({
    super.key,
    required this.label,
    required this.items,
    required this.value,
    required this.onChanged,
    this.hint,
    this.enabled = true,
  });

  final String label;
  final List<DropdownMenuItem<T>> items;
  final T? value;
  final ValueChanged<T?>? onChanged;
  final String? hint;
  final bool enabled;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          label,
          style: Theme.of(context)
              .textTheme
              .labelLarge
              ?.copyWith(color: c.textSecondary),
        ),
        const SizedBox(height: EelSpacing.s1),
        DropdownButtonFormField<T>(
          initialValue: value,
          items: items,
          onChanged: enabled ? onChanged : null,
          hint: hint == null ? null : Text(hint!),
          decoration: InputDecoration(
            enabled: enabled,
            suffixIcon: Icon(Icons.expand_more, color: c.textSecondary),
          ),
          borderRadius: BorderRadius.circular(EelRadius.lg),
        ),
      ],
    );
  }
}

/// Search field with clear button and debounce-friendly onChanged.
class EelSearchField extends StatefulWidget {
  const EelSearchField({
    super.key,
    this.hintText = 'Search',
    this.controller,
    this.onChanged,
    this.onSubmitted,
    this.autofocus = false,
  });

  final String hintText;
  final TextEditingController? controller;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;
  final bool autofocus;

  @override
  State<EelSearchField> createState() => _EelSearchFieldState();
}

class _EelSearchFieldState extends State<EelSearchField> {
  late final TextEditingController _internal;
  TextEditingController get _effective => widget.controller ?? _internal;

  @override
  void initState() {
    super.initState();
    _internal = TextEditingController();
  }

  @override
  void dispose() {
    _internal.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return ValueListenableBuilder<TextEditingValue>(
      valueListenable: _effective,
      builder: (context, value, _) => TextField(
        controller: _effective,
        autofocus: widget.autofocus,
        textInputAction: TextInputAction.search,
        decoration: InputDecoration(
          hintText: widget.hintText,
          prefixIcon: Icon(Icons.search, color: c.textSecondary),
          suffixIcon: value.text.isEmpty
              ? null
              : IconButton(
                  icon: Icon(Icons.clear, size: 20, color: c.textSecondary),
                  tooltip: 'Clear search',
                  onPressed: () {
                    _effective.clear();
                    widget.onChanged?.call('');
                  },
                ),
        ),
        onChanged: widget.onChanged,
        onSubmitted: widget.onSubmitted,
      ),
    );
  }
}
