//! Operational scheduler; settlement always uses the explicit ADMIN contract.
use crate::{
    json::{self, Value},
    World,
};
use std::io::Read;
use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct AutoSettlement {
    last_check: Option<Instant>,
    completed_day: Option<String>,
    pub last_error: Option<String>,
}

impl AutoSettlement {
    fn due(&mut self, now: Instant, date: &str, time: &str, trading_day: &str, at: &str) -> bool {
        if trading_day != date || time < at || self.completed_day.as_deref() == Some(trading_day) {
            return false;
        }
        if self
            .last_check
            .map(|last| now.saturating_duration_since(last) < Duration::from_secs(1))
            .unwrap_or(false)
        {
            return false;
        }
        self.last_check = Some(now);
        true
    }
}

impl World {
    pub(crate) fn poll_auto_settlement(&mut self, now: Instant) {
        let Some(at) = self.cfg.auto_settle_at.as_deref() else {
            return;
        };
        let time = crate::dtime::now_cst_time();
        let date = crate::dtime::today_natural_date();
        if !self
            .auto_settlement
            .due(now, &date, &time[..5], &self.vt_trading_day, at)
        {
            return;
        }
        let current = self.vt_trading_day.clone();
        let result = (|| {
            let path = self
                .cfg
                .auto_settle_prices
                .as_ref()
                .ok_or("自动日结等待结算价文件")?;
            let file = std::fs::File::open(path).map_err(|_| "自动日结结算价文件不可用")?;
            let mut text = String::new();
            file.take(1_048_577)
                .read_to_string(&mut text)
                .map_err(|_| "自动日结结算价文件不可用")?;
            if text.len() > 1_048_576 {
                return Err("自动日结结算价文件超过 1 MiB".into());
            }
            let value = json::parse(&text).map_err(|_| "自动日结结算价文件 JSON 无效")?;
            let fields = value.as_obj().ok_or("自动日结结算价文件必须是对象")?;
            if json::unknown_or_duplicate(
                fields,
                &["trading_day", "next_trading_day", "settlement_prices"],
            )
            .is_some()
            {
                return Err("自动日结结算价文件含未知或重复字段".to_string());
            }
            if value.get_str("trading_day").as_deref() != Some(current.as_str()) {
                return Err("自动日结结算价日期与当前 TradingDay 不匹配".to_string());
            }
            let mut request = vec![("cmd".into(), json::s("settle_day"))];
            request.push((
                "settlement_prices".into(),
                value
                    .get("settlement_prices")
                    .ok_or("自动日结缺少 settlement_prices")?
                    .clone(),
            ));
            if let Some(next) = value.get("next_trading_day") {
                if next.as_str().is_none() {
                    return Err("next_trading_day 必须是字符串".into());
                }
                request.push(("next_trading_day".into(), next.clone()));
            }
            self.settle_day(&Value::Obj(request))
        })();
        match result {
            Ok(_) => {
                self.auto_settlement.completed_day = Some(current);
                self.auto_settlement.last_error = None;
            }
            Err(error) => {
                if self.auto_settlement.last_error.as_deref() != Some(&error) {
                    eprintln!("[ctpbuddy] auto settlement waiting: {error}");
                }
                self.auto_settlement.last_error = Some(error);
            }
        }
    }

    pub(crate) fn auto_settlement_status(&self) -> Value {
        json::obj_sorted(vec![
            ("enabled".into(), json::b(self.cfg.auto_settle_at.is_some())),
            ("timezone".into(), json::s("Asia/Shanghai")),
            (
                "at".into(),
                self.cfg
                    .auto_settle_at
                    .as_deref()
                    .map(json::s)
                    .unwrap_or(Value::Null),
            ),
            (
                "completed_day".into(),
                self.auto_settlement
                    .completed_day
                    .as_deref()
                    .map(json::s)
                    .unwrap_or(Value::Null),
            ),
            (
                "last_error".into(),
                self.auto_settlement
                    .last_error
                    .as_deref()
                    .map(json::s)
                    .unwrap_or(Value::Null),
            ),
        ])
    }
}
