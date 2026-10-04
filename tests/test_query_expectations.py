# -*- coding: utf-8 -*-
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[1] / "tools"))

from audit_three_way import (  # noqa: E402
    BadData,
    expected_positions,
    match_trades_orders,
    settlement_positions,
)


class QueryExpectationTests(unittest.TestCase):
    def settlement(self, qty=3, instrument="rb2601", exchange="SHFE"):
        detail = [{
            "交易所": exchange, "合约": instrument, "投资单元": "", "投保": "投机",
            "买卖": "买", "持仓量": str(qty),
        }]
        summary = [{
            "合约": instrument, "投资单元": "", "投保": "投机",
            "买持": str(qty), "卖持": "0",
        }]
        return {"positions_detail": detail, "positions_summary": summary}

    def trade(self, instrument="rb2601", exchange="SHFE", direction="0", offset="0", volume=1, hedge="投机"):
        return {
            "BrokerID": "B", "InvestorID": "I", "TradingDay": "20260105",
            "ExchangeID": exchange, "InstrumentID": instrument, "Direction": direction,
            "OffsetFlag": offset, "Volume": str(volume), "HedgeFlag": hedge,
        }

    def test_shfe_initial_open_and_close_today_yesterday(self):
        prior = self.settlement(2)
        trades = [self.trade(offset="0", volume=2), self.trade(direction="1", offset="4", volume=1), self.trade(direction="1", offset="3", volume=1)]
        got, ambiguity = expected_positions(prior, trades, "20260105")
        self.assertEqual(got[(("SHFE", "rb2601", "", "投机"), "long")], 2)
        self.assertEqual(ambiguity["shfe_ine_close_age_not_inferred"], 0)

    def test_option_quantity_change_does_not_create_premium(self):
        prior = self.settlement(1, "MO2601-C-6000", "CFFEX")
        got, _ = expected_positions(prior, [self.trade("MO2601-C-6000", "CFFEX", volume=2)], "20260105")
        self.assertEqual(got[(("CFFEX", "MO2601-C-6000", "", "投机"), "long")], 3)
        self.assertNotIn("premium", str(got).lower())

    def test_duplicate_order_sysid_is_ambiguous(self):
        base = {"BrokerID": "B", "InvestorID": "I", "TradingDay": "20260105", "ExchangeID": "SHFE", "OrderSysID": "7", "OrderRef": "9", "FrontID": "1", "SessionID": "2", "InstrumentID": "rb2601", "Direction": "0"}
        orders = [dict(base), dict(base, OrderRef="10")]
        trade = dict(base, OrderRef="9", OffsetFlag="0", Volume="1")
        matched, reasons = match_trades_orders([trade], orders)
        self.assertEqual(matched, 0)
        self.assertEqual(reasons["ambiguous_order_match"], 1)

    def test_negative_position_fails(self):
        with self.assertRaises(BadData):
            settlement_positions({"positions_detail": [{"交易所": "SHFE", "合约": "rb", "买卖": "买", "持仓量": "-1"}]}, "synthetic")

    def test_detail_summary_total_mismatch_fails(self):
        doc = self.settlement(2)
        doc["positions_summary"][0]["买持"] = "1"
        with self.assertRaises(BadData):
            settlement_positions(doc, "synthetic")

    def test_missing_prior_is_explicitly_representable(self):
        got, _ = expected_positions(None, [self.trade(volume=1)], "20260105")
        self.assertEqual(sum(got.values()), 1)


if __name__ == "__main__":
    unittest.main()
