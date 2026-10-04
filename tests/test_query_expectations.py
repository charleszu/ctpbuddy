# -*- coding: utf-8 -*-
import csv
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parents[1] / "tools"))

from audit_three_way import (  # noqa: E402
    BadData,
    audit,
    compare_position_query,
    expected_positions,
    match_trades_orders,
    read_csv,
    settlement_positions,
)
from e2e.m4_real_replay import controlled_quotes  # noqa: E402


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

    def test_bad_offset_join_is_rejected(self):
        order = {"BrokerID": "B", "InvestorID": "I", "TradingDay": "20260105", "ExchangeID": "SHFE", "OrderSysID": "7", "CombOffsetFlag": "0", "InstrumentID": "rb2601", "Direction": "0"}
        trade = dict(order, OffsetFlag="1", Volume="1")
        matched, reasons = match_trades_orders([trade], [order])
        self.assertEqual(matched, 0)
        self.assertEqual(reasons["order_trade_offset_mismatch"], 1)

    def test_sell_controlled_quote_hits_trade_price(self):
        self.assertEqual(controlled_quotes("1", 3500.0), (3500.0, 3500.01))
        self.assertEqual(controlled_quotes("0", 3500.0), (3499.99, 3500.0))

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

    def test_ctp_position_direction_bytes_50_and_51(self):
        expected = {
            (("SHFE", "rb2601", "", "投机"), "long"): 2,
            (("SHFE", "rb2601", "", "投机"), "short"): 3,
        }
        actual = [
            {"交易所": "SHFE", "合约": "rb2601", "投保": "投机", "PosiDirection": "50", "Position": "2"},
            {"交易所": "SHFE", "合约": "rb2601", "投保": "投机", "PosiDirection": "51", "Position": "3"},
        ]
        self.assertEqual(compare_position_query(actual, expected), [])

    def test_position_rows_are_not_counted_as_both_sides(self):
        expected = {(("SHFE", "rb2601", "", "投机"), "long"): 2}
        actual = [{"交易所": "SHFE", "合约": "rb2601", "投保": "投机", "PosiDirection": "2", "Position": "2"}]
        self.assertEqual(compare_position_query(actual, expected), [])

    def test_over_close_fails_before_later_open_can_hide_it(self):
        with self.assertRaises(BadData):
            expected_positions(self.settlement(1), [
                self.trade(direction="1", offset="1", volume=2),
                self.trade(offset="0", volume=2),
            ], "20260105")

    def test_bom_csv_is_decoded_without_gbk_header_corruption(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / "rows.csv"
            path.write_bytes("\ufeffBrokerID,InvestorID\nB,I\n".encode("utf-8"))
            self.assertEqual(read_csv(path)[0]["BrokerID"], "B")

    def test_diff_is_fail_and_missing_actual_query_is_not_evaluated(self):
        with tempfile.TemporaryDirectory() as root:
            export = Path(root) / "export"
            settlement = Path(root) / "settlement"
            actual = Path(root) / "actual"
            export.mkdir(); settlement.mkdir(); actual.mkdir()
            day, investor = "20260105", "I"
            (export / f"{day}_{investor}_account.csv").write_text("BrokerID,InvestorID,TradingDay\nB,I,20260105\n", encoding="utf-8")
            (export / f"{day}_{investor}_order.csv").write_text("BrokerID,InvestorID,TradingDay,ExchangeID,InstrumentID,OrderSysID,OrderRef,FrontID,SessionID,Direction,CombOffsetFlag,LimitPrice,VolumeTotalOriginal\nB,I,20260105,SHFE,rb2601,7,9,1,2,0,0,3500,1\n", encoding="utf-8")
            (export / f"{day}_{investor}_trade.csv").write_text("BrokerID,InvestorID,TradingDay,TradeDate,ExchangeID,InstrumentID,OrderSysID,OrderRef,TradeID,Direction,OffsetFlag,HedgeFlag,Volume\nB,I,20260105,20260105,SHFE,rb2601,7,9,T1,0,0,投机,1\n", encoding="utf-8")
            report = audit(export, settlement)
            self.assertEqual(report["status"], "not_evaluated")
            self.assertIn("missing_same_day_settlement", report["skips"])
            (settlement / "settlement.json").write_text(json.dumps({"meta_info": {"结算日期": day, "资金账号": investor}, "positions_detail": [{"交易所": "SHFE", "合约": "rb2601", "投保": "投机", "买卖": "买", "持仓量": "1"}], "positions_summary": [{"合约": "rb2601", "投保": "投机", "买持": "1", "卖持": "0"}]}, ensure_ascii=False), encoding="utf-8")
            (settlement / "prior.json").write_text(json.dumps({"meta_info": {"结算日期": "20260104", "资金账号": investor}, "positions_detail": [], "positions_summary": []}, ensure_ascii=False), encoding="utf-8")
            report = audit(export, settlement, actual_dir=str(actual))
            self.assertEqual(report["status"], "not_evaluated")
            actual_path = actual / f"{day}_{investor}_position.csv"
            actual_path.write_text("交易所,合约,投保,PosiDirection,Position\nSHFE,rb2601,投机,2,9\n", encoding="utf-8")
            report = audit(export, settlement, actual_dir=str(actual))
            self.assertEqual(report["status"], "fail")
            self.assertGreater(report["difference_count"], 0)


if __name__ == "__main__":
    unittest.main()
