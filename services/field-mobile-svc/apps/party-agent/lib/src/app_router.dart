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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'features/auth/login_screen.dart';
import 'features/check_in/check_in_screen.dart';
import 'features/qr_verify/qr_verify_screen.dart';
import 'features/countersign/countersign_screen.dart';
import 'features/dispute/dispute_screen.dart';

final appRouterProvider = Provider<GoRouter>((ref) {
	return GoRouter(
		initialLocation: '/login',
		routes: [
			GoRoute(
				path: '/login',
				builder: (context, state) => const LoginScreen(),
			),
			GoRoute(
				path: '/check-in',
				builder: (context, state) => const CheckInScreen(),
			),
			GoRoute(
				path: '/verify',
				builder: (context, state) => const QRVerifyScreen(),
			),
			GoRoute(
				path: '/countersign',
				builder: (context, state) => const CountersignScreen(),
			),
			GoRoute(
				path: '/dispute',
				builder: (context, state) => const DisputeScreen(),
			),
		],
	);
});
