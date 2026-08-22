/// Custody mobile app root widget and navigation configuration.
library app;

import 'package:flutter/material.dart';

import 'screens/rfid_scan.dart';
import 'services/rfid_reader_service.dart';

class CustodyApp extends StatelessWidget {
  const CustodyApp({super.key, this.readerModel = RfidReaderModel.zebraRfd40});

  final RfidReaderModel readerModel;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Custody Mobile',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: const Color(0xFF1A5276)),
        useMaterial3: true,
      ),
      initialRoute: '/',
      routes: {
        '/': (ctx) => const _HomeScreen(),
        '/rfid-scan': (ctx) => RfidScanScreen(
              officerId: ModalRoute.of(ctx)?.settings.arguments as String? ?? '',
              readerService: createReaderService(readerModel),
            ),
      },
    );
  }
}

class _HomeScreen extends StatelessWidget {
  const _HomeScreen();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Custody Mobile')),
      body: ListView(
        children: [
          ListTile(
            leading: const Icon(Icons.nfc),
            title: const Text('RFID Cover Scan'),
            subtitle: const Text('Scan and verify ballot box cover tags'),
            onTap: () => Navigator.pushNamed(
              context,
              '/rfid-scan',
              arguments: 'OFF-UNKNOWN', // replaced at runtime from provisioned profile
            ),
          ),
        ],
      ),
    );
  }
}
