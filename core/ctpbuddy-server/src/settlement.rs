//! 结算报告输入、持久化与查询；正文为原始 GBK 字节。
use crate::json::{self, Value};
use crate::World;
use ctpbuddy_ledger::Ledger;

#[derive(Clone)]
pub(crate) struct Report {
    pub broker: String,
    pub investor: String,
    pub day: String,
    pub settlement_id: i32,
    pub account: String,
    pub currency: String,
    pub content: Vec<u8>,
    pub source: String,
}

impl Report {
    pub fn parse(v: &Value, broker: &str) -> Result<Self, String> {
        let fields = v.as_obj().ok_or("报告必须为对象")?;
        let mut seen = std::collections::HashSet::new();
        for (k, _) in fields {
            if !matches!(
                k.as_str(),
                "broker"
                    | "investor"
                    | "trading_day"
                    | "settlement_id"
                    | "account_id"
                    | "currency_id"
                    | "content_bytes"
                    | "source"
            ) || !seen.insert(k)
            {
                return Err("报告含未知或重复字段".into());
            }
        }
        let b = v.get_str("broker").ok_or("报告缺少 broker")?;
        let investor = v.get_str("investor").ok_or("报告缺少 investor")?;
        let day = v.get_str("trading_day").ok_or("报告缺少 trading_day")?;
        let id = v.get_num("settlement_id").ok_or("报告缺少 settlement_id")?;
        let account = v.get_str("account_id").unwrap_or_default();
        let currency = v.get_str("currency_id").unwrap_or_default();
        if b != broker
            || investor.is_empty()
            || investor.len() > 12
            || !investor.is_ascii()
            || investor.contains('\0')
            || b.len() > 10
            || !b.is_ascii()
            || b.contains('\0')
        {
            return Err("报告身份无效".into());
        }
        if !((day.len() == 8 && valid_day(&day))
            || (day.len() == 6
                && day.bytes().all(|c| c.is_ascii_digit())
                && day[4..]
                    .parse::<u32>()
                    .map(|m| (1..=12).contains(&m))
                    .unwrap_or(false)))
        {
            return Err("报告 trading_day 必须为有效 YYYYMMDD 或 YYYYMM".into());
        }
        if id < 1.0
            || id > i32::MAX as f64
            || id.fract() != 0.0
            || !id.is_finite()
            || account.len() > 12
            || currency.len() > 3
            || !account.is_ascii()
            || !currency.is_ascii()
            || account.contains('\0')
            || currency.contains('\0')
        {
            return Err("报告元数据无效".into());
        }
        let Some(Value::Arr(bytes)) = v.get("content_bytes") else {
            return Err("报告需要 content_bytes 字节数组".into());
        };
        if bytes.is_empty() || bytes.len() > 512 * 1024 {
            return Err("报告正文长度必须为 1..524288 字节".into());
        }
        let mut content = Vec::with_capacity(bytes.len());
        for byte in bytes {
            let n = byte.as_num().ok_or("正文必须为字节数组")?;
            if !(1.0..=255.0).contains(&n) || n.fract() != 0.0 {
                return Err("正文字节必须为 1..255，不能含 NUL".into());
            }
            content.push(n as u8);
        }
        Ok(Self {
            broker: b,
            investor,
            day,
            settlement_id: id as i32,
            account,
            currency,
            content,
            source: v
                .get_str("source")
                .unwrap_or_else(|| "user_supplied".into()),
        })
    }

    pub fn value(&self) -> Value {
        json::obj_sorted(vec![
            ("broker".into(), json::s(&self.broker)),
            ("investor".into(), json::s(&self.investor)),
            ("trading_day".into(), json::s(&self.day)),
            ("settlement_id".into(), json::n(self.settlement_id as f64)),
            ("account_id".into(), json::s(&self.account)),
            ("currency_id".into(), json::s(&self.currency)),
            ("source".into(), json::s(&self.source)),
            (
                "content_bytes".into(),
                Value::Arr(self.content.iter().map(|b| json::n(*b as f64)).collect()),
            ),
        ])
    }
}

impl World {
    pub(crate) fn validate_reports(&self, reports: &[Report]) -> Result<(), String> {
        let mut keys = std::collections::HashSet::new();
        for r in reports {
            if !keys.insert((&r.broker, &r.investor, &r.day))
                || self.settlement_reports.iter().any(|old| {
                    old.broker == r.broker && old.investor == r.investor && old.day == r.day
                })
            {
                return Err("结算报告已存在或输入重复，不允许覆盖".into());
            }
        }
        Ok(())
    }

    pub(crate) fn save_reports(&mut self, reports: Vec<Report>) -> Result<(), String> {
        self.validate_reports(&reports)?;
        if !self.cfg.data_dir.is_empty() && !reports.is_empty() {
            use std::io::Write;
            let dir = std::path::Path::new(&self.cfg.data_dir).join("settlement_reports");
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos();
            let path = dir.join(format!("{stamp}.json"));
            let pending = dir.join(format!("{stamp}.pending"));
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&pending)
                .map_err(|e| e.to_string())?;
            file.write_all(
                Value::Arr(reports.iter().map(Report::value).collect())
                    .to_json()
                    .as_bytes(),
            )
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
            drop(file);
            std::fs::rename(pending, path).map_err(|e| e.to_string())?;
        }
        for r in reports {
            self.journal_record_json("settlement_report", &r.broker, &r.investor, r.value());
            self.settlement_reports.push(r);
        }
        Ok(())
    }

    pub(crate) fn minimal_reports(ledger: &Ledger, day: &str) -> Vec<Report> {
        let mut accounts: Vec<_> = ledger.accounts().collect();
        accounts.sort_by(|a, b| {
            a.broker_id
                .cmp(&b.broker_id)
                .then(a.investor_id.cmp(&b.investor_id))
        });
        accounts.into_iter().map(|a| {
            let content = format!("CTPBuddy simulated ledger minimal settlement report\nSource=modeled_ledger_only; not an official broker statement\nTradingDay={day}\nBrokerID={}\nInvestorID={}\nCurrencyID={}\nSettledEquity={}\nCarriedMargin={}\nUnmodeled fields omitted; amounts from explicit settle_day\n", a.broker_id, a.investor_id, a.currency_id, a.pre_balance, a.used_margin).into_bytes();
            Report { broker: a.broker_id.clone(), investor: a.investor_id.clone(), day: day.into(), settlement_id: 1, account: a.investor_id.clone(), currency: a.currency_id.clone(), content, source: "modeled_ledger_minimal".into() }
        }).collect()
    }
}

fn valid_day(day: &str) -> bool {
    if day.len() != 8 || !day.bytes().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let y = day[0..4].parse::<i64>().ok();
    let m = day[4..6].parse::<u32>().ok();
    let d = day[6..8].parse::<u32>().ok();
    match (y, m, d) {
        (Some(y), Some(m), Some(d)) if (1..=12).contains(&m) && (1..=31).contains(&d) => {
            let next = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
            crate::dtime::days_from_civil(next.0, next.1, 1)
                - crate::dtime::days_from_civil(y, m, 1)
                >= d as i64
        }
        _ => false,
    }
}

pub(crate) fn load_reports(dir: &str, broker: &str) -> Result<Vec<Report>, String> {
    let path = std::path::Path::new(dir).join("settlement_reports");
    if dir.is_empty() || !path.exists() {
        return Ok(Vec::new());
    }
    let mut paths = std::fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    paths.sort_by_key(|e| e.file_name());
    let mut out = Vec::new();
    let mut keys = std::collections::HashSet::new();
    for entry in paths {
        if entry.path().extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).map_err(|e| e.to_string())?;
        let Value::Arr(rows) = json::parse(&text)? else {
            return Err("报告存储格式错误".into());
        };
        for row in rows {
            let report = Report::parse(&row, broker)?;
            if !keys.insert((
                report.broker.clone(),
                report.investor.clone(),
                report.day.clone(),
            )) {
                return Err("存储中报告重复".into());
            }
            out.push(report);
        }
    }
    Ok(out)
}
