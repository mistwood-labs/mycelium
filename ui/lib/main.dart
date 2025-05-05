import 'package:flutter/material.dart';
import 'src/mycelium_ffi.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();

  // Start the node on all interfaces (auto-select port)
  final started = nodeStart('/ip4/0.0.0.0/tcp/0');
  if (!started) {
    debugPrint('Failed to start Mycelium node');
  }

  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Mycelium Client',
      theme: ThemeData(primarySwatch: Colors.blue),
      home: const PeerPage(),
    );
  }
}

class PeerPage extends StatefulWidget {
  const PeerPage({super.key});

  @override
  State<PeerPage> createState() => _PeerPageState();
}

class _PeerPageState extends State<PeerPage> {
  List<String> _peers = [];
  final TextEditingController _addrController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _refreshPeers();
  }

  void _refreshPeers() {
    setState(() {
      _peers = connectedPeers();
    });
  }

  void _discoverPeers() {
    final discovered = discoveredNodes();
    setState(() {
      _peers = discovered;
    });
  }

  void _connectPeer() {
    final addr = _addrController.text;
    debugPrint('Attempting to connect to $addr');
    final success = connectToPeer(addr);
    debugPrint('connectToPeer returned $success');
    if (success) {
      debugPrint('Connection successful, refreshing peers');
      _refreshPeers();
    } else {
      debugPrint('Connection failed');
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Failed to connect to $addr')),
      );
    }
  }

  void _stopNode() {
    final stopped = nodeStop();
    if (!stopped) {
      debugPrint('Failed to stop Mycelium node');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Mycelium Peers'),
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: _discoverPeers,
            tooltip: 'Discover via mDNS',
          ),
          IconButton(
            icon: const Icon(Icons.power_settings_new),
            onPressed: _stopNode,
            tooltip: 'Stop Node',
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            TextField(
              controller: _addrController,
              decoration: const InputDecoration(
                labelText: 'Peer Multiaddr',
                hintText: '/ip4/…/tcp/…',
              ),
            ),
            const SizedBox(height: 8),
            ElevatedButton(
              onPressed: _connectPeer,
              child: const Text('Connect'),
            ),
            const SizedBox(height: 16),
            Expanded(
              child: ListView.builder(
                itemCount: _peers.length,
                itemBuilder: (context, index) => ListTile(
                  title: Text(_peers[index]),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
