import 'dart:io';

void main() {
  final protoDir = Directory('../proto');
  final outDir = Directory('lib/generated/proto');
  if (!outDir.existsSync()) outDir.createSync(recursive: true);
  for (var file in protoDir.listSync().whereType<File>()) {
    if (file.path.endsWith('.proto')) {
      Process.runSync('protoc', [
        '--dart_out=lib/generated/proto',
        '--proto_path=../proto',
        file.path,
      ], runInShell: true);
    }
  }
}
