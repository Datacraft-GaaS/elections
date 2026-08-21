import 'package:flutter/material.dart';

import 'enrolment_workflow.dart';
import 'i18n_delegate.dart';

class BvrKitApp extends StatelessWidget {
  const BvrKitApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'eel BVR Kit',
      localizationsDelegates: eelLocalizationDelegates,
      supportedLocales: eelSupportedLocales,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF006600)),
        useMaterial3: true,
      ),
      home: const EnrolmentWorkflowScreen(),
    );
  }
}
