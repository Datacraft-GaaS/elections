import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';

import '../theme.dart';
import 'notifications_controller.dart';
import 'workforce_api.dart';

class NotificationsScreen extends ConsumerStatefulWidget {
  const NotificationsScreen({super.key});

  @override
  ConsumerState<NotificationsScreen> createState() => _NotificationsScreenState();
}

class _NotificationsScreenState extends ConsumerState<NotificationsScreen> {
  @override
  void initState() {
    super.initState();
    // Load the inbox on first open.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) ref.read(notificationsControllerProvider).refresh();
    });
  }

  @override
  Widget build(BuildContext context) {
    final controller = ref.watch(notificationsControllerProvider);
    final state = controller.state;

    return Scaffold(
      appBar: AppBar(title: const Text('Notifications')),
      body: state.loading && !state.hasCachedData
          ? const Center(child: CircularProgressIndicator())
          : RefreshIndicator(
              onRefresh: controller.refresh,
              child: _buildList(context, controller, state),
            ),
    );
  }

  Widget _buildList(
    BuildContext context,
    NotificationsController controller,
    NotificationsState state,
  ) {
    if (state.notifications.isEmpty && !state.loading) {
      return _EmptyState(onRefresh: controller.refresh);
    }

    return ListView(
      physics: const AlwaysScrollableScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
      children: [
        if (state.errorMessage != null)
          _ErrorBanner(message: state.errorMessage!),
        if (state.notifications.isNotEmpty)
          Padding(
            padding: const EdgeInsets.fromLTRB(4, 4, 4, 12),
            child: Text(
              'Your latest postings, payments and tasks',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
        for (var i = 0; i < state.notifications.length; i++)
          _NotificationCard(
            notification: state.notifications[i],
            isNewest: i == 0,
          ),
      ],
    );
  }
}

/// Distinct colour + glyph per notification kind, drawn from the ledger
/// palette: green for the work of the day, gold for money, seal red for
/// attention.
class _KindStyle {
  const _KindStyle(this.icon, this.color, this.tint, this.label);

  final IconData icon;
  final Color color;
  final Color tint;
  final String label;
}

const _kindStyles = <NotificationKind, _KindStyle>{
  NotificationKind.posting:
      _KindStyle(Icons.location_on_outlined, EelColors.green, EelColors.greenTint, 'Posting'),
  NotificationKind.payment:
      _KindStyle(Icons.payments_outlined, EelColors.gold, EelColors.goldTint, 'Payment'),
  NotificationKind.appointment:
      _KindStyle(Icons.verified_outlined, EelColors.greenDeep, EelColors.greenTint, 'Appointment'),
  NotificationKind.training:
      _KindStyle(Icons.school_outlined, EelColors.green, EelColors.greenTint, 'Training'),
  NotificationKind.message:
      _KindStyle(Icons.chat_bubble_outline, EelColors.inkSoft, EelColors.paper2, 'Message'),
};

class _NotificationCard extends StatelessWidget {
  const _NotificationCard({required this.notification, required this.isNewest});

  final WorkforceNotification notification;
  final bool isNewest;

  @override
  Widget build(BuildContext context) {
    final style = _kindStyles[notification.kind] ?? _kindStyles[NotificationKind.message]!;
    final time = DateFormat('d MMM · HH:mm').format(notification.createdAt.toLocal());

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: EelColors.raised,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(
          color: isNewest ? EelColors.greenLine : EelColors.line,
        ),
        boxShadow: isNewest
            ? const [
                BoxShadow(
                  color: Color(0x1412231A),
                  blurRadius: 20,
                  offset: Offset(0, 8),
                ),
              ]
            : null,
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 42,
            height: 42,
            decoration: BoxDecoration(
              color: style.tint,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Icon(style.icon, color: style.color, size: 22),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Text(
                        notification.subject,
                        style: const TextStyle(
                          fontWeight: FontWeight.w600,
                          fontSize: 15.5,
                          color: EelColors.ink,
                        ),
                      ),
                    ),
                    if (isNewest) _NewStamp(label: style.label),
                  ],
                ),
                const SizedBox(height: 4),
                Text(
                  notification.body,
                  style: const TextStyle(
                    fontSize: 14,
                    height: 1.45,
                    color: EelColors.inkSoft,
                  ),
                ),
                const SizedBox(height: 10),
                Row(
                  children: [
                    _Chip(
                      color: style.color,
                      tint: style.tint,
                      label: style.label,
                    ),
                    if (notification.channel != null) ...[
                      const SizedBox(width: 8),
                      _Chip(
                        color: EelColors.inkFaint,
                        tint: EelColors.paper2,
                        label: notification.channel!,
                      ),
                    ],
                    const Spacer(),
                    Text(time, style: EelTypography.mono.copyWith(fontSize: 11, color: EelColors.inkFaint)),
                  ],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _NewStamp extends StatelessWidget {
  const _NewStamp({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: EelColors.sealTint,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: const Color(0xFFF0C9C4)),
      ),
      child: Text(
        label.toUpperCase(),
        style: EelTypography.mono.copyWith(
          fontSize: 10,
          fontWeight: FontWeight.w700,
          letterSpacing: 0.08,
          color: EelColors.sealDeep,
        ),
      ),
    );
  }
}

class _Chip extends StatelessWidget {
  const _Chip({required this.color, required this.tint, required this.label});

  final Color color;
  final Color tint;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: tint,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(
        label,
        style: TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: color),
      ),
    );
  }
}

class _ErrorBanner extends StatelessWidget {
  const _ErrorBanner({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: EelColors.sealTint,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: const Color(0xFFF0C9C4)),
      ),
      child: Row(
        children: [
          const Icon(Icons.cloud_off, color: EelColors.sealDeep),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              message,
              style: const TextStyle(color: EelColors.sealDeep, fontWeight: FontWeight.w600),
            ),
          ),
        ],
      ),
    );
  }
}

class _EmptyState extends StatelessWidget {
  const _EmptyState({required this.onRefresh});

  final Future<void> Function() onRefresh;

  @override
  Widget build(BuildContext context) {
    return ListView(
      physics: const AlwaysScrollableScrollPhysics(),
      children: [
        const SizedBox(height: 72),
        const Icon(Icons.mark_email_unread_outlined, size: 56, color: EelColors.green),
        const SizedBox(height: 20),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 40),
          child: Text(
            'Nothing here yet.',
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.headlineMedium,
          ),
        ),
        const SizedBox(height: 8),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 40),
          child: Text(
            'When you are posted, paid, or sent training, it will appear here.',
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.bodyMedium,
          ),
        ),
        const SizedBox(height: 24),
        Center(
          child: OutlinedButton.icon(
            onPressed: onRefresh,
            icon: const Icon(Icons.refresh, size: 18),
            label: const Text('Refresh'),
          ),
        ),
      ],
    );
  }
}
