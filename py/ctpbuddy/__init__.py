"""ctpbuddy - local/private CTP-compatible simulated trading environment.

Layout:
- wire      framing + message ids shared with the C++ shim
- sdk       sync client (front session) + admin control-plane client
- sources   market-data source plugins (canonical scenario CSV in M1)
- refdata   reference-data provider plugins (contracts / margin / commission)
- cli       `ctpbuddy` command line entry
"""
__version__ = "0.1.0"

from .calendar import CalendarDay, CalendarError, TradingCalendar, load_calendar

__all__ = ["CalendarDay", "CalendarError", "TradingCalendar", "load_calendar"]
