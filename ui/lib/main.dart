// The original content is temporarily commented out to allow generating a self-contained demo - feel free to uncomment later.

// // ui/lib/main.dart
//
// import 'package:flutter/material.dart';
// import 'src/mycelium_ffi.dart';
//
// void main() {
//   WidgetsFlutterBinding.ensureInitialized();
//
//   print("➤ Calling nodeStart...");
//   final ok = nodeStart("/ip4/0.0.0.0/tcp/0");
//   print("➤ nodeStart returned $ok");
//
//   runApp(const MyApp());
// }
//
// class MyApp extends StatelessWidget {
//   const MyApp({super.key});
//   @override
//   Widget build(BuildContext context) => MaterialApp(
//         title: 'Mycelium Client',
//         theme: ThemeData(primarySwatch: Colors.blue),
//         home: const PeerPage(),
//       );
// }
//
// class PeerPage extends StatefulWidget {
//   const PeerPage({super.key});
//   @override
//   State<PeerPage> createState() => _PeerPageState();
// }
//
// class _PeerPageState extends State<PeerPage> {
//   List<String> _peers = [];
//   final TextEditingController _addrController = TextEditingController();
//
//   @override
//   void initState() {
//     super.initState();
//     _refreshPeers();
//   }
//
//   void _refreshPeers() {
//     setState(() => _peers = connectedPeers());
//   }
//
//   void _discoverPeers() {
//     print('🔍 [UI] Discover button pressed');
//     final discovered = discoveredNodes();
//     print('🔍 [UI] discoveredNodes returned ${discovered.length} peers');
//     setState(() => _peers = discovered);
//   }
//
//   void _connectPeer() {
//     final addr = _addrController.text.trim();
//     if (addr.isEmpty) {
//       ScaffoldMessenger.of(context).showSnackBar(
//         const SnackBar(content: Text('Please enter a peer multiaddr')),
//       );
//       return;
//     }
//     print('🔌 [UI] Connect to $addr');
//     final success = connectToPeer(addr);
//     print('🔌 [UI] connectToPeer returned $success');
//     if (success) {
//       _refreshPeers();
//     } else {
//       ScaffoldMessenger.of(context).showSnackBar(
//         SnackBar(content: Text('Failed to connect to $addr')),
//       );
//     }
//   }
//
//   void _stopNode() {
//     final stopped = nodeStop();
//     if (!stopped) debugPrint('Failed to stop Mycelium node');
//   }
//
//   @override
//   Widget build(BuildContext context) {
//     return Scaffold(
//       appBar: AppBar(
//         title: const Text('Mycelium Peers'),
//         actions: [
//           IconButton(
//             icon: const Icon(Icons.search),
//             onPressed: _discoverPeers,
//             tooltip: 'Discover via mDNS',
//           ),
//           IconButton(
//             icon: const Icon(Icons.power_settings_new),
//             onPressed: _stopNode,
//             tooltip: 'Stop Node',
//           ),
//         ],
//       ),
//       body: Padding(
//         padding: const EdgeInsets.all(16),
//         child: Column(
//           children: [
//             TextField(
//               controller: _addrController,
//               decoration: const InputDecoration(
//                 labelText: 'Peer Multiaddr',
//                 hintText: '/ip4/…/tcp/…',
//               ),
//             ),
//             const SizedBox(height: 8),
//             ElevatedButton(onPressed: _connectPeer, child: const Text('Connect')),
//             const SizedBox(height: 16),
//             Expanded(
//               child: ListView.builder(
//                 itemCount: _peers.length,
//                 itemBuilder: (_, i) => ListTile(title: Text(_peers[i])),
//               ),
//             ),
//           ],
//         ),
//       ),
//     );
//   }
// }
//

import 'package:flutter/material.dart';
import 'package:mycelium_ui/src/rust/api/simple.dart';
import 'package:mycelium_ui/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('flutter_rust_bridge quickstart')),
        body: Center(
          child: Text(
            'Action: Call Rust `greet("Tom")`\nResult: `${greet(name: "Tom")}`',
          ),
        ),
      ),
    );
  }
}
