import 'package:flutter/material.dart';

// Fixture: positive case for dart_stub_ui scanner rule.
// This file should trigger findings when scanned.

class StubScreen extends StatelessWidget {
  const StubScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Stub App')),
      body: const Center(child: Text('GPS station visit log stub')),
    );
  }
}

class AnotherStub extends StatelessWidget {
  const AnotherStub({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(child: Text('Mission PDF export stub')),
      floatingActionButton: FloatingActionButton(
        onPressed: () {},
        child: const Icon(Icons.add),
      ),
    );
  }
}
