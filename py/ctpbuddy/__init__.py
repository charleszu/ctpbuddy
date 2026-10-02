"""ctpbuddy - local/private CTP-compatible simulated trading environment.

Layout:
- wire      framing + message ids shared with the C++ shim
- sdk       sync client (front session) + admin control-plane client
- sources   market-data source plugins (canonical scenario CSV in M1)
- cli       `ctpbuddy` command line entry
"""
__version__ = "0.1.0"
