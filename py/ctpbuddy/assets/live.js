(() => {
  const byId = id => document.getElementById(id);
  const columns = {
    Accounts: [['broker','Broker'],['investor','账户'],['balance','权益'],['available','可用'],['used_margin','保证金'],['position_profit','持仓盈亏'],['commission','手续费']],
    Positions: [['broker','Broker'],['investor','账户'],['instrument','合约'],['side','方向'],['open_date','开仓日'],['trade_id','成交号'],['volume','数量'],['open_price','开仓价'],['settlement_price','结算价']],
    Orders: [['broker','Broker'],['investor','账户'],['instrument','合约'],['order_ref','报单引用'],['order_sys_id','报单号'],['direction','买卖'],['offset','开平'],['status','状态'],['price','价格'],['volume','委托量'],['volume_traded','成交量'],['volume_total','剩余量']],
    Trades: [['broker','Broker'],['investor','账户'],['instrument','合约'],['trade_id','成交号'],['order_sys_id','报单号'],['direction','买卖'],['offset','开平'],['price','价格'],['volume','数量'],['time','时间']]
  };
  const labels = {
    status: {'0':'全部成交','1':'部分成交','2':'部分成交已结束','3':'未成交','4':'未成交已结束','5':'已撤单','a':'已提交','b':'未触发','c':'已触发'},
    direction: {'0':'买','1':'卖'}, offset: {'0':'开','1':'平','3':'平今','4':'平昨'}, side: {long:'多',short:'空'}
  };
  let offset = 0, busy = false, timer, generation = 0;
  const limit = 50;
  function table(name, rows) {
    const target = byId('live' + name);
    target.replaceChildren();
    if (!rows.length) { target.textContent = '暂无记录'; return; }
    const table = document.createElement('table'), head = document.createElement('thead'), row = document.createElement('tr');
    columns[name].forEach(([,label]) => { const cell = document.createElement('th'); cell.textContent = label; row.append(cell); });
    head.append(row); table.append(head);
    const body = document.createElement('tbody');
    rows.forEach(item => {
      const row = document.createElement('tr');
      columns[name].forEach(([key]) => { const cell = document.createElement('td'), value = item[key]; cell.textContent = labels[key]?.[value] ?? (value == null ? '—' : String(value)); row.append(cell); });
      body.append(row);
    });
    table.append(body); target.append(table);
  }
  function schedule() {
    clearTimeout(timer);
    if (byId('liveAuto').checked && !document.hidden) timer = setTimeout(refresh, 2000);
  }
  async function refresh() {
    if (busy) return;
    clearTimeout(timer); busy = true;
    const request = generation, controller = new AbortController(), timeout = setTimeout(() => controller.abort(), 5000);
    byId('liveRefresh').disabled = true;
    try {
      const params = new URLSearchParams({limit:String(limit),offset:String(offset)});
      ['Broker','Investor'].forEach(name => { const value = byId('live'+name).value.trim(); if (value) params.set(name.toLowerCase(), value); });
      const response = await fetch('/api/live?' + params, {signal:controller.signal, headers:{Accept:'application/json'}}), live = await response.json();
      if (!response.ok) throw new Error(live.error || '读取失败');
      if (request !== generation) return;
      Object.keys(columns).forEach(name => table(name, live[name.toLowerCase()] || []));
      byId('accountStatus').textContent = '实时核心 · 交易日 '+live.trading_day+' · journal '+live.metadata.journal_seq+' · 更新于 '+new Date().toLocaleTimeString();
      byId('accountStatus').className = 'ok';
      byId('livePage').textContent = '第 '+(offset/limit+1)+' 页 · 账户 '+live.account_count+' / 持仓明细 '+live.position_count+' / 订单 '+live.order_count+' / 成交 '+live.trade_count;
      byId('livePrev').disabled = offset === 0;
      byId('liveNext').disabled = Math.max(live.account_count,live.position_count,live.order_count,live.trade_count) <= offset+limit;
    } catch (error) {
      if (request !== generation) return;
      byId('accountStatus').textContent = '实时核心不可用；下表保留上次读数：'+error.message;
      byId('accountStatus').className = 'err';
    } finally {
      clearTimeout(timeout); busy = false; byId('liveRefresh').disabled = false;
      if (request !== generation && !document.hidden) refresh(); else schedule();
    }
  }
  ['liveBroker','liveInvestor'].forEach(id => { byId(id).onchange = () => { offset=0; generation++; refresh(); }; });
  byId('liveRefresh').onclick = refresh;
  byId('livePrev').onclick = () => { offset=Math.max(0,offset-limit); generation++; refresh(); };
  byId('liveNext').onclick = () => { offset+=limit; generation++; refresh(); };
  byId('liveAuto').onchange = schedule;
  document.addEventListener('visibilitychange', () => { if (document.hidden) clearTimeout(timer); else refresh(); });
  refresh();
})();
