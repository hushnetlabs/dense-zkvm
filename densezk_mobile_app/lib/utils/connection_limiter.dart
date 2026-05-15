import 'dart:async';

class ConnectionLimiter {
  final int maxCount;
  int _currentCount = 0;
  final _waitQueue = <Completer<void>>[];

  ConnectionLimiter(this.maxCount);

  Future<void> acquire() async {
    if(_currentCount < maxCount) {
      _currentCount++;
      return;
    }
    final completer = Completer<void>();
    _waitQueue.add(completer);
    await completer.future;
  }

  void release() {
    if(_waitQueue.isNotEmpty) {
      _waitQueue.removeAt(0).complete();
    } else {
      _currentCount--;
    }
  }
}