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

/// Text field with always-visible label, helper and error slots.
class EelTextField extends StatelessWidget {
  const EelTextField({
    super.key,
    required this.label,
    this.controller,
    this.hintText,
    @Deprecated('Use hintText') this.hint,
    this.helper,
    this.errorText,
    this.obscureText = false,
    this.keyboardType,
    this.textInputAction,
    this.enabled = true,
    this.maxLines = 1,
    this.onChanged,
    this.onSubmitted,
    this.autofillHints,
  });

  final String label;
  final TextEditingController? controller;
  /// Placeholder shown while the field is empty.
  final String? hintText;

  /// Deprecated alias for [hintText], which is the name Flutter's own
  /// [TextField] uses. Reads [hintText] when both are given.
  @Deprecated('Use hintText, to match TextField.hintText')
  final String? hint;
  final String? helper;
  final String? errorText;
  final bool obscureText;
  final TextInputType? keyboardType;
  final TextInputAction? textInputAction;
  final bool enabled;
  final int maxLines;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;
  final Iterable<String>? autofillHints;

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Label always visible (never floating-only) per plan §5.2.
        Text(
          label,
          style: Theme.of(context).textTheme.labelLarge?.copyWith(
                color: c.textSecondary,
              ),
        ),
        const SizedBox(height: EelSpacing.s1),
        TextField(
          controller: controller,
          decoration: InputDecoration(
            hintText: hintText ?? hint,
            errorText: errorText,
            helperText: helper,
            enabled: enabled,
            errorMaxLines: 3,
          ),
          obscureText: obscureText,
          keyboardType: keyboardType,
          textInputAction: textInputAction,
          maxLines: obscureText ? 1 : maxLines,
          onChanged: onChanged,
          onSubmitted: onSubmitted,
          autofillHints: autofillHints,
        ),
      ],
    );
  }
}

/// 6-digit PIN field with masked entry and per-digit boxes.
class EelPinField extends StatefulWidget {
  const EelPinField({
    super.key,
    required this.length,
    required this.onCompleted,
    this.onChanged,
    this.obscure = true,
  });

  final int length;
  final ValueChanged<String> onCompleted;
  final ValueChanged<String>? onChanged;
  final bool obscure;

  @override
  State<EelPinField> createState() => _EelPinFieldState();
}

class _EelPinFieldState extends State<EelPinField> {
  late final TextEditingController _controller;
  late final FocusNode _focus;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController();
    _focus = FocusNode();
    _controller.addListener(_handle);
  }

  void _handle() {
    final text = _controller.text;
    widget.onChanged?.call(text);
    if (text.length == widget.length) {
      widget.onCompleted(text);
    }
  }

  @override
  void dispose() {
    _controller.removeListener(_handle);
    _controller.dispose();
    _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final c = context.eelColors;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        TextField(
          controller: _controller,
          focusNode: _focus,
          keyboardType: TextInputType.number,
          maxLength: widget.length,
          obscureText: widget.obscure,
          autofillHints: const [AutofillHints.password],
          decoration: InputDecoration(
            counterText: '',
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(EelRadius.lg),
            ),
          ),
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: EelSpacing.s2),
        // Digit position indicators, keyboard-accessible state announcement.
        ValueListenableBuilder<TextEditingValue>(
          valueListenable: _controller,
          builder: (context, value, _) => Row(
            children: List.generate(
              widget.length,
              (i) => Padding(
                padding: const EdgeInsets.only(right: EelSpacing.s1),
                child: AnimatedContainer(
                  duration: EelMotion.fast,
                  width: EelSpacing.s3,
                  height: EelSpacing.s3,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: i < value.text.length ? c.textPrimary : c.border,
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
