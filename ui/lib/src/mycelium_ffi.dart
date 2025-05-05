import 'dart:ffi';
import 'dart:io' show Platform;
import 'package:ffi/ffi.dart';
import 'dart:convert';

/// Load the native Mycelium library for the current platform
DynamicLibrary _openMyceliumLib() {
  if (Platform.isLinux) return DynamicLibrary.open('/home/s8sato/git/works/mycelium/ui/lib/generated/libmycelium_core.so');
  if (Platform.isMacOS) return DynamicLibrary.open('libmycelium.dylib');
  return DynamicLibrary.open('mycelium.dll');
}

final DynamicLibrary _lib = _openMyceliumLib();

// ----- FFI type definitions and bindings -----

// node_start: initialize and start the P2P node
// Signature: Pointer<Utf8> -> Uint8
typedef _node_start_native = Uint8 Function(Pointer<Utf8>);
typedef _NodeStart = int Function(Pointer<Utf8>);
final _NodeStart _nodeStart = _lib
  .lookup<NativeFunction<_node_start_native>>('node_start')
  .asFunction();

/// Starts the node listening on the given multiaddr. Returns true on success.
bool nodeStart(String addr) {
  final ptr = addr.toNativeUtf8();
  final result = _nodeStart(ptr);
  calloc.free(ptr);
  return result != 0;
}

// node_stop: stop the P2P node
typedef _node_stop_native = Uint8 Function();
typedef _NodeStop = int Function();
final _NodeStop _nodeStop = _lib
  .lookup<NativeFunction<_node_stop_native>>('node_stop')
  .asFunction();

/// Stops the node. Returns true on success.
bool nodeStop() => _nodeStop() != 0;

// connected_peers: get JSON-encoded list of peers
typedef _connected_peers_native = Pointer<Utf8> Function();
typedef _ConnectedPeers = Pointer<Utf8> Function();
final _ConnectedPeers _connectedPeers = _lib
  .lookup<NativeFunction<_connected_peers_native>>('connected_peers')
  .asFunction();

/// Retrieves the list of connected peers.
List<String> connectedPeers() {
  final ptr = _connectedPeers();
  final jsonStr = ptr.toDartString();
  calloc.free(ptr);
  return List<String>.from(json.decode(jsonStr));
}

// discovered_nodes: mDNS-based peer discovery
typedef _discovered_nodes_native = Pointer<Utf8> Function();
typedef _DiscoveredNodes = Pointer<Utf8> Function();
final _DiscoveredNodes _discoveredNodes = _lib
  .lookup<NativeFunction<_discovered_nodes_native>>('discovered_nodes')
  .asFunction();

/// Discovers local peers via mDNS.
List<String> discoveredNodes() {
  final ptr = _discoveredNodes();
  final jsonStr = ptr.toDartString();
  calloc.free(ptr);
  return List<String>.from(json.decode(jsonStr));
}

// connect_to_peer: dial a peer address
typedef _connect_to_peer_native = Uint8 Function(Pointer<Utf8>);
typedef _ConnectToPeer = int Function(Pointer<Utf8>);
final _ConnectToPeer _connectToPeer = _lib
  .lookup<NativeFunction<_connect_to_peer_native>>('connect_to_peer')
  .asFunction();

/// Connects to the given multiaddr. Returns true on success.
bool connectToPeer(String addr) {
  final ptr = addr.toNativeUtf8();
  final result = _connectToPeer(ptr);
  calloc.free(ptr);
  return result != 0;
}

// publish_post: publish a SignedPost message
typedef _publish_post_native = Uint8 Function(
  Pointer<Utf8>, Pointer<Uint8>, Uint64
);
typedef _PublishPost = int Function(
  Pointer<Utf8>, Pointer<Uint8>, int
);
final _PublishPost _publishPost = _lib
  .lookup<NativeFunction<_publish_post_native>>('publish_post')
  .asFunction();

/// Publishes a SignedPost to the given topic.
bool publishPost(String topic, List<int> signedPostBytes) {
  final tPtr = topic.toNativeUtf8();
  final pPtr = calloc<Uint8>(signedPostBytes.length);
  final buffer = pPtr.asTypedList(signedPostBytes.length);
  buffer.setAll(0, signedPostBytes);
  final result = _publishPost(tPtr, pPtr, signedPostBytes.length);
  calloc.free(tPtr);
  calloc.free(pPtr);
  return result != 0;
}

// subscribe_topic: subscribe to a topic
typedef _subscribe_topic_native = Uint8 Function(Pointer<Utf8>);
typedef _SubscribeTopic = int Function(Pointer<Utf8>);
final _SubscribeTopic _subscribeTopic = _lib
  .lookup<NativeFunction<_subscribe_topic_native>>('subscribe_topic')
  .asFunction();

/// Subscribes to the specified topic. Returns true on success.
bool subscribeTopic(String topic) {
  final ptr = topic.toNativeUtf8();
  final result = _subscribeTopic(ptr);
  calloc.free(ptr);
  return result != 0;
}

// send_reaction: send a SignedReaction message
typedef _send_reaction_native = Uint8 Function(
  Pointer<Utf8>, Pointer<Uint8>, Uint64
);
typedef _SendReaction = int Function(
  Pointer<Utf8>, Pointer<Uint8>, int
);
final _SendReaction _sendReaction = _lib
  .lookup<NativeFunction<_send_reaction_native>>('send_reaction')
  .asFunction();

/// Sends a SignedReaction to a peer. Returns true on success.
bool sendReaction(String peer, List<int> signedReactionBytes) {
  final pPtr = peer.toNativeUtf8();
  final bPtr = calloc<Uint8>(signedReactionBytes.length);
  final buf = bPtr.asTypedList(signedReactionBytes.length);
  buf.setAll(0, signedReactionBytes);
  final result = _sendReaction(pPtr, bPtr, signedReactionBytes.length);
  calloc.free(pPtr);
  calloc.free(bPtr);
  return result != 0;
}
