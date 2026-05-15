import 'dart:convert';
import 'package:http/http.dart' as http;
import '../models/densezk_models.dart';
import '../utils/connection_limiter.dart';

class DenseZkApiException implements Exception {
  final String method;
  final String message;
  final int? statusCode;

  const DenseZkApiException(this.method, this.message, {this.statusCode});

  @override
  String toString() => 'DenseZkApiException($method): $message';
}

class DenseZkApiService {
  late String _baseUrl;
  final Duration _timeout = const Duration(seconds: 30);
  final int maxConcurrentRequests;

  DenseZkApiService({String? baseUrl, this.maxConcurrentRequests = 4}) {
    _baseUrl = baseUrl ?? 'http://10.0.2.2:8080';
  }

  void setBaseUrl(String url) {
    _baseUrl = url;
  }

  String get baseUrl => _baseUrl;

  Future<ServerHealth> checkHealth() async {
    try {
      final response = await http.get(Uri.parse('$_baseUrl/health')).timeout(_timeout);

      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        return ServerHealth.fromJson(json);
      }
      throw DenseZkApiException('checkHealth', 'Server returned ${response.statusCode}', statusCode: response.statusCode);
    } catch (e) {
      if (e is DenseZkApiException) rethrow;
      throw DenseZkApiException('checkHealth', e.toString());
    }
  }

  Future<ClientWitness> createWitness({required int senderId, required int receiverId, required int weight}) async {
    try {
      final response = await http
          .post(Uri.parse('$_baseUrl/witness'), headers: {'Content-Type': 'application/json'}, body: jsonEncode({'sender_id': senderId, 'receiver_id': receiverId, 'weight': weight}))
          .timeout(_timeout);

      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        return ClientWitness.fromJson(json);
      }
      final error = _parseError(response.body);
      throw DenseZkApiException('createWitness', error, statusCode: response.statusCode);
    } catch (e) {
      if (e is DenseZkApiException) rethrow;
      throw DenseZkApiException('createWitness', e.toString());
    }
  }

  Future<ZKProof> prove({required int senderId, required int receiverId, required int weight, required String graphRoot, required int threshold}) async {
    try {
      final response = await http
          .post(
            Uri.parse('$_baseUrl/prove'),
            headers: {'Content-Type': 'application/json'},
            body: jsonEncode({'sender_id': senderId, 'receiver_id': receiverId, 'weight': weight, 'graph_root': graphRoot, 'threshold': threshold}),
          )
          .timeout(_timeout);

      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        return ZKProof.fromJson(json);
      }
      final error = _parseError(response.body);
      throw DenseZkApiException('prove', error, statusCode: response.statusCode);
    } catch (e) {
      if (e is DenseZkApiException) rethrow;
      throw DenseZkApiException('prove', e.toString());
    }
  }

  Future<ZKProof> executeFlow({required int senderId, required int receiverId, required int weight, required String graphRoot, required int threshold}) async {
    try {
      final response = await http
          .post(
            Uri.parse('$_baseUrl/execute_flow'),
            headers: {'Content-Type': 'application/json'},
            body: jsonEncode({'sender_id': senderId, 'receiver_id': receiverId, 'weight': weight, 'graph_root': graphRoot, 'threshold': threshold}),
          )
          .timeout(_timeout);

      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        return ZKProof.fromJson(json);
      }
      final error = _parseError(response.body);
      throw DenseZkApiException('executeFlow', error, statusCode: response.statusCode);
    } catch (e) {
      if (e is DenseZkApiException) rethrow;
      throw DenseZkApiException('executeFlow', e.toString());
    }
  }

  Future<String> poseidonHash({required int senderId, required int receiverId, required int weight}) async {
    try {
      final response = await http
          .post(Uri.parse('$_baseUrl/poseidon_hash'), headers: {'Content-Type': 'application/json'}, body: jsonEncode({'sender_id': senderId, 'receiver_id': receiverId, 'weight': weight}))
          .timeout(_timeout);

      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        return json['hash'] as String;
      }
      final error = _parseError(response.body);
      throw DenseZkApiException('poseidonHash', error, statusCode: response.statusCode);
    } catch (e) {
      if (e is DenseZkApiException) rethrow;
      throw DenseZkApiException('poseidonHash', e.toString());
    }
  }

  Future<Map<String, dynamic>> runStressTest({required int iterations, int senderId = 456, int receiverId = 789, int weight = 1, String graphRoot = '0xabc123', int threshold = 1}) async {
    final results = <String, dynamic>{'iterations': iterations, 'concurrency': maxConcurrentRequests, 'success_count': 0, 'fail_count': 0, 'proof_sizes': <int>[], 'times_ms': <int>[]};

    final overallStart = DateTime.now().millisecondsSinceEpoch;
    final limiter = ConnectionLimiter(maxConcurrentRequests);

    final futures = List.generate(iterations, (_) async {
      await limiter.acquire();
      final startTime = DateTime.now().millisecondsSinceEpoch;
      try {
        final proof = await _withRetry(() => executeFlow(senderId: senderId, receiverId: receiverId, weight: weight, graphRoot: graphRoot, threshold: threshold));
        final elapsed = DateTime.now().millisecondsSinceEpoch - startTime;
        return {'success': true, 'proof': proof, 'elapsed': elapsed};
      } catch (e) {
        return {'success': false, 'elapsed': 0};
      } finally {
        limiter.release();
      }
    });

    final allResults = await Future.wait(futures);
    results['parallel_time_ms'] = DateTime.now().millisecondsSinceEpoch - overallStart;

    for (final r in allResults) {
      if (r['success'] == true) {
        results['success_count'] = (results['success_count'] as int) + 1;
        final proof = r['proof'] as ZKProof;
        (results['proof_sizes'] as List<int>).add(proof.proofBytes.length);
        (results['times_ms'] as List<int>).add(r['elapsed'] as int);
      } else {
        results['fail_count'] = (results['fail_count'] as int) + 1;
      }
    }

    final successCount = results['success_count'] as int;
    results['success_rate'] = iterations > 0 ? '${(successCount / iterations * 100).toStringAsFixed(1)}%' : '0%';
    if ((results['times_ms'] as List<int>).isNotEmpty) {
      final times = results['times_ms'] as List<int>;
      results['total_time_ms'] = times.reduce((a, b) => a + b);
      results['avg_time_ms'] = (results['total_time_ms'] as int) ~/ successCount;
      results['min_time_ms'] = times.reduce((a, b) => a < b ? a : b);
      results['max_time_ms'] = times.reduce((a, b) => a > b ? a : b);
    }

    return results;
  }

  Future<List<ClientWitness>> createWitnessBatch(List<Map<String, dynamic>> inputs) async {
    final limiter = ConnectionLimiter(maxConcurrentRequests);

    final futures = inputs.map((input) async {
      try {
        await limiter.acquire();
        return await _withRetry(() => createWitness(senderId: input['sender_id'] as int, receiverId: input['receiver_id'] as int, weight: input['weight'] as int));
      } finally {
        limiter.release();
      }
    });

    return Future.wait(futures);
  }

  Future<T> _withRetry<T>(Future<T> Function() fn, {int maxRetries = 3}) async {
    int attempt = 0;
    while (true) {
      try {
        return await fn();
      } catch (e) {
        attempt++;
        if (attempt >= maxRetries) rethrow;
        await Future.delayed(Duration(milliseconds: 200 * attempt));
      }
    }
  }

  Future<Map<String, dynamic>> runConcurrencyBenchmark({required int iterations, int senderId = 456, int receiverId = 789, int weight = 1, String graphRoot = '0xabc123', int threshold = 1}) async {
    final benchmarks = <String, dynamic>{};

    for (final concurrency in [1, 2, 4, 8]) {
      final limiter = ConnectionLimiter(concurrency);
      final start = DateTime.now().millisecondsSinceEpoch;

      final futures = List.generate(iterations, (_) async {
        await limiter.acquire();
        try {
          return await _withRetry(() => executeFlow(senderId: senderId, receiverId: receiverId, weight: weight, graphRoot: graphRoot, threshold: threshold));
        } finally {
          limiter.release();
        }
      });

      await Future.wait(futures);
      benchmarks['concurrency_$concurrency'] = DateTime.now().millisecondsSinceEpoch - start;
    }

    // calculate speedup vs sequential (concurrency=1)
    final baseline = benchmarks['concurrency_1'] as int;
    benchmarks['speedup_2x'] = '${(baseline / (benchmarks['concurrency_2'] as int)).toStringAsFixed(1)}x';
    benchmarks['speedup_4x'] = '${(baseline / (benchmarks['concurrency_4'] as int)).toStringAsFixed(1)}x';
    benchmarks['speedup_8x'] = '${(baseline / (benchmarks['concurrency_8'] as int)).toStringAsFixed(1)}x';

    return benchmarks;
  }

  String _parseError(String body) {
    try {
      final json = jsonDecode(body) as Map<String, dynamic>;
      return json['error'] ?? body;
    } catch (_) {
      return body;
    }
  }
}
