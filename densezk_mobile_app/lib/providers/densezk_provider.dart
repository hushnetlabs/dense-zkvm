import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../models/densezk_models.dart';
import '../services/densezk_api_service.dart';

class DenseZkProvider extends ChangeNotifier {
  final DenseZkApiService _api;

  bool _isConnected = false;
  bool _isLoading = false;
  String _error = '';
  ClientWitness? _lastWitness;
  ZKProof? _lastProof;
  ServerHealth? _health;
  Map<String, dynamic>? _stressResults;

  static const String _serverUrlKey = 'server_url';

  DenseZkProvider({String? serverUrl})
      : _api = DenseZkApiService(baseUrl: serverUrl);

  Future<void> init() async {
    final prefs = await SharedPreferences.getInstance();
    final savedUrl = prefs.getString(_serverUrlKey);
    if (savedUrl != null && savedUrl.isNotEmpty) {
      _api.setBaseUrl(savedUrl);
    }
    notifyListeners();
  }

  bool get isConnected => _isConnected;
  bool get isLoading => _isLoading;
  String get error => _error;
  ClientWitness? get lastWitness => _lastWitness;
  ZKProof? get lastProof => _lastProof;
  ServerHealth? get health => _health;
  Map<String, dynamic>? get stressResults => _stressResults;
  String get serverUrl => _api.baseUrl;

  Map<String, dynamic>? get benchmarkResults => _benchmarkResults;
  Map<String, dynamic>? _benchmarkResults;

  Future<void> runConcurrencyBenchmark({
    required int iterations,
  }) async {
    _isLoading = true;
    _error = '';
    _benchmarkResults = null;
    notifyListeners();

    try {
      _benchmarkResults = await _api.runConcurrencyBenchmark(
        iterations: iterations,
      );
    } catch (e) {
      _error = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }
  Future<void> setServerUrl(String url) async {
    _api.setBaseUrl(url);
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_serverUrlKey, url);
    notifyListeners();
  }

  void _setLoading(bool value) {
    _isLoading = value;
    if (value) _error = '';
    notifyListeners();
  }

  void _setError(String message) {
    _error = message;
    _isLoading = false;
    notifyListeners();
  }

  Future<bool> checkConnection() async {
    try {
      _health = await _api.checkHealth();
      _isConnected = true;
      notifyListeners();
      return true;
    } catch (e) {
      _isConnected = false;
      _error = e.toString();
      notifyListeners();
      return false;
    }
  }

  Future<ClientWitness?> createWitness({
    required int senderId,
    required int receiverId,
    required int weight,
  }) async {
    _setLoading(true);
    try {
      _lastWitness = await _api.createWitness(
        senderId: senderId,
        receiverId: receiverId,
        weight: weight,
      );
      _error = '';
      return _lastWitness;
    } catch (e) {
      _setError(e.toString());
      return null;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<ZKProof?> prove({
    required int senderId,
    required int receiverId,
    required int weight,
    required String graphRoot,
    required int threshold,
  }) async {
    _setLoading(true);
    try {
      _lastProof = await _api.prove(
        senderId: senderId,
        receiverId: receiverId,
        weight: weight,
        graphRoot: graphRoot,
        threshold: threshold,
      );
      _error = '';
      return _lastProof;
    } catch (e) {
      _setError(e.toString());
      return null;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<ZKProof?> executeFlow({
    required int senderId,
    required int receiverId,
    required int weight,
    required String graphRoot,
    required int threshold,
  }) async {
    _setLoading(true);
    try {
      _lastProof = await _api.executeFlow(
        senderId: senderId,
        receiverId: receiverId,
        weight: weight,
        graphRoot: graphRoot,
        threshold: threshold,
      );
      _error = '';
      return _lastProof;
    } catch (e) {
      _setError(e.toString());
      return null;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<String?> poseidonHash({
    required int senderId,
    required int receiverId,
    required int weight,
  }) async {
    _setLoading(true);
    try {
      final hash = await _api.poseidonHash(
        senderId: senderId,
        receiverId: receiverId,
        weight: weight,
      );
      _error = '';
      _isLoading = false;
      notifyListeners();
      return hash;
    } catch (e) {
      _setError(e.toString());
      return null;
    }
  }

  Future<Map<String, dynamic>?> runStressTest({
    required int iterations,
    int senderId = 456,
    int receiverId = 789,
    int weight = 1,
    String graphRoot = '0xabc123',
    int threshold = 1,
  }) async {
    _setLoading(true);
    try {
      _stressResults = await _api.runStressTest(
        iterations: iterations,
        senderId: senderId,
        receiverId: receiverId,
        weight: weight,
        graphRoot: graphRoot,
        threshold: threshold,
      );
      _error = '';
      return _stressResults;
    } catch (e) {
      _setError(e.toString());
      return null;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  void clearResults() {
    _lastWitness = null;
    _lastProof = null;
    _stressResults = null;
    _error = '';
    notifyListeners();
  }
}
