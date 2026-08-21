import 'package:flutter/material.dart';

void main() {
  runApp(const EelHelloApp());
}

class EelHelloApp extends StatelessWidget {
  const EelHelloApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'eel hello',
      home: Scaffold(
        appBar: AppBar(title: const Text('eel hello')),
        body: const Center(
          child: Text('ok: true'),
        ),
      ),
    );
  }
}
