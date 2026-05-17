import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../providers/densezk_provider.dart';
import '../widgets/result_card.dart';

class StressTestScreen extends StatefulWidget {
  const StressTestScreen({super.key});

  @override
  State<StressTestScreen> createState() => _StressTestScreenState();
}

class _StressTestScreenState extends State<StressTestScreen> {
  final _formKey = GlobalKey<FormState>();
  final _iterationsController = TextEditingController(text: '10');
  final _senderController = TextEditingController(text: '456');
  final _receiverController = TextEditingController(text: '789');
  final _weightController = TextEditingController(text: '1');
  final _graphRootController = TextEditingController(text: '0xabc123');
  final _thresholdController = TextEditingController(text: '1');

  bool _stressLoading = false;
  bool _benchmarkLoading = false;


  @override
  void dispose() {
    _iterationsController.dispose();
    _senderController.dispose();
    _receiverController.dispose();
    _weightController.dispose();
    _graphRootController.dispose();
    _thresholdController.dispose();
    super.dispose();
  }

  Future<void> _runStressTest() async {
    if (!_formKey.currentState!.validate()) return;
    setState(() => _stressLoading = true);
    await context.read<DenseZkProvider>().runStressTest(
          iterations: int.parse(_iterationsController.text),
          senderId: int.parse(_senderController.text),
          receiverId: int.parse(_receiverController.text),
          weight: int.parse(_weightController.text),
          graphRoot: _graphRootController.text,
          threshold: int.parse(_thresholdController.text),
        );
    setState(() => _stressLoading = false);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Stress Test'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Consumer<DenseZkProvider>(
        builder: (context, provider, child) {
          return SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Form(
              key: _formKey,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  const Card(
                    child: Padding(
                      padding: EdgeInsets.all(16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            'Performance Benchmark',
                            style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
                          ),
                          SizedBox(height: 8),
                          Text(
                            'Run multiple proof generations and measure performance metrics.',
                            style: TextStyle(fontSize: 14, color: Colors.grey),
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 16),
                  _buildTextField(
                    controller: _iterationsController,
                    label: 'Iterations',
                    hint: '10',
                  ),
                  const SizedBox(height: 12),
                  _buildTextField(
                    controller: _senderController,
                    label: 'Sender ID',
                    hint: '456',
                  ),
                  const SizedBox(height: 12),
                  _buildTextField(
                    controller: _receiverController,
                    label: 'Receiver ID',
                    hint: '789',
                  ),
                  const SizedBox(height: 12),
                  _buildTextField(
                    controller: _weightController,
                    label: 'Edge Weight',
                    hint: '1',
                  ),
                  const SizedBox(height: 12),
                  _buildTextField(
                    controller: _graphRootController,
                    label: 'Graph Root',
                    hint: '0xabc123',
                  ),
                  const SizedBox(height: 12),
                  _buildTextField(
                    controller: _thresholdController,
                    label: 'Threshold',
                    hint: '1',
                  ),
                  const SizedBox(height: 24),
                  ElevatedButton.icon(
                    onPressed: (_stressLoading || _benchmarkLoading) ? null : _runStressTest,
                    icon: _stressLoading
                        ? const SizedBox(
                            width: 20,
                            height: 20,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.speed),
                    label: Text(_stressLoading  ? 'Running...' : 'Run Stress Test'),
                  ),
                  const SizedBox(height: 12),
                  ElevatedButton.icon(
                    onPressed: (_stressLoading || _benchmarkLoading) ? null : () async {
                      setState(() => _benchmarkLoading = true);
                      await context.read<DenseZkProvider>().runConcurrencyBenchmark(
                        iterations: int.parse(_iterationsController.text),
                      );
                      setState(() => _benchmarkLoading = false);
                    },
                    icon: _benchmarkLoading
                        ? const SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                        : const Icon(Icons.bar_chart),
                    label: Text(_benchmarkLoading ? 'Running...' : 'Run Concurrency Benchmark'),
                  ),
                  if (provider.error.isNotEmpty) ...[
                    const SizedBox(height: 16),
                    ResultCard(
                      title: 'Error',
                      content: provider.error,
                      isError: true,
                    ),
                  ],
                  if (provider.stressResults != null) ...[
                    const SizedBox(height: 16),
                    _buildStressResults(provider.stressResults!),
                  ],
                  if (provider.benchmarkResults != null) ...[
                    const SizedBox(height: 16),
                    _buildBenchmarkResults(provider.benchmarkResults!),
                  ],
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildStressResults(Map<String, dynamic> results) {
    final successCount = results['success_count'] as int? ?? 0;
    final failCount = results['fail_count'] as int? ?? 0;
    final successRate = results['success_rate'] as String? ?? "0%";

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Stress Test Results',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 16),
            _buildResultRow('Iterations', results['iterations'].toString()),
            _buildResultRow('Success', '$successCount'),
            _buildResultRow('Failed', '$failCount'),
            _buildResultRow('Success Rate', successRate),
            if (results['total_time_ms'] != null) ...[
              const Divider(),
              _buildResultRow('Parallel Time', '${results['parallel_time_ms']} ms'),
              _buildResultRow('Total Time', '${results['total_time_ms']} ms'),
              _buildResultRow('Avg Time', '${results['avg_time_ms']} ms'),
              _buildResultRow('Min Time', '${results['min_time_ms']} ms'),
              _buildResultRow('Max Time', '${results['max_time_ms']} ms'),
            ],
            if ((results['proof_sizes'] as List).isNotEmpty) ...[
              const Divider(),
              _buildResultRow(
                'Proof Size',
                '${(results['proof_sizes'] as List<int>).first} bytes',
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildResultRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: TextStyle(color: Colors.grey[600])),
          Text(value, style: const TextStyle(fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }

  Widget _buildTextField({
    required TextEditingController controller,
    required String label,
    required String hint,
  }) {
    return TextFormField(
      controller: controller,
      decoration: InputDecoration(labelText: label, hintText: hint),
      keyboardType: TextInputType.number,
      validator: (v) => v == null || v.isEmpty ? 'Required' : null,
    );
  }

  Widget _buildBenchmarkResults(Map<String, dynamic> results) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Concurrency Benchmark',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            const Text(
              'Same test with different concurrency levels',
              style: TextStyle(fontSize: 12, color: Colors.grey),
            ),
            const SizedBox(height: 16),
            _buildResultRow('Sequential (1x)', '${results['concurrency_1']} ms'),
            _buildResultRow('2 Concurrent',   '${results['concurrency_2']} ms'),
            _buildResultRow('4 Concurrent',   '${results['concurrency_4']} ms'),
            _buildResultRow('8 Concurrent',   '${results['concurrency_8']} ms'),
            const Divider(),
            _buildResultRow('2x Speedup', results['speedup_2x'] as String),
            _buildResultRow('4x Speedup', results['speedup_4x'] as String),
            _buildResultRow('8x Speedup', results['speedup_8x'] as String),
          ],
        ),
      ),
    );
  }
}
