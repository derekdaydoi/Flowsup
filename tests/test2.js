<script>
(() => {
const out = []; const ok = (name, c, info='') => out.push((c ? 'PASS ' : 'FAIL ') + name + ' ' + info);
const base = { id:'t', label:'t', principal: 100e6, start:'2026-11-14', first:'2026-12-14', end:'2027-02-25', mode:'equal', timing:'periodic', rate:1.5, unit:'month' };
const t1 = D.schedule({ ...base, basis:'total', fixed: 6e6 });
ok('total periodic sum', t1.total === 6e6 && t1.rows.reduce((a,r)=>a+r.interest,0) === 6e6, t1.rows.map(r=>r.days+':'+r.interest).join(' '));
ok('total by days', t1.rows[1].interest > t1.rows[0].interest && t1.rows[3].interest < t1.rows[0].interest);
const t2 = D.schedule({ ...base, basis:'total', fixed: 6e6, timing:'upfront' });
ok('total upfront', t2.upfront === 6e6 && t2.rows[0].interest === 6e6 && t2.rows.slice(1).every(r => r.interest === 0));
const u = { ...base, basis:'flat', timing:'upfront' }; const ev11 = D.loanEvents(u,'2026-11'), ev12 = D.loanEvents(u,'2026-12');
ok('upfront on first date', ev11.length === 0 && ev12.some(e => e.cat==='interest' && e.day===14 && e.value === -D.schedule(u).total), JSON.stringify(ev12));
const t3 = D.schedule({ ...base, basis:'total', fixed: 6e6, timing:'end' });
ok('total end', t3.rows[3].interest === 6e6);
const f = D.schedule({ ...base, basis:'flat' });
ok('flat still', f.rows.map(r=>r.interest).join() === '1479452,1528767,1528767,542466');
const pre = document.createElement('pre'); pre.id='testout'; pre.textContent = 'TESTS\n' + out.join('\n') + '\nENDTESTS'; document.body.append(pre);
})();
</script>
<script>(() => { const out=[]; const ok=(n,c,i='')=>out.push((c?'PASS ':'FAIL ')+n+' '+i);
const s = { openDate:'2026-11-01', horizon:3, opening:0, carry:true, incomes:[], loans:[],
  living:{ items:[{id:'a',amount:5e6},{id:'b',amount:8e6,from:'2026-12',until:'2026-12'},{id:'c',amount:1e6,from:'2027-01',until:null}] } };
const P = D.project(s);
ok('expense monthly + once + range', P[0].living === 5e6 && P[1].living === 13e6 && P[2].living === 6e6, P.map(m=>m.living).join());
const pre=document.createElement('pre'); pre.id='t4'; pre.textContent='TESTS\n'+out.join('\n')+'\nENDTESTS'; document.body.append(pre); })();</script>
<script>(() => { const out=[]; const ok=(n,c,i='')=>out.push((c?'PASS ':'FAIL ')+n+' '+i);
const dao = { id:'d', dir:'borrow', label:'Đáo', principal:20e6, start:'2026-10-20', first:'2026-11-20', end:'2029-10-20', mode:'bullet', basis:'fixed', fixed:320000, timing:'periodic', open:true };
const ev = D.loanEvents(dao, '2027-03');
ok('open fixed: monthly interest', ev.length === 1 && ev[0].value === -320000 && ev[0].day === 20 && ev[0].cat === 'interest', JSON.stringify(ev));
const tc = { id:'t', dir:'borrow', label:'TC', principal:90e6, start:'2026-10-01', first:'2026-11-30', end:'2027-10-30', mode:'bullet', basis:'reducing', rate:19.4, unit:'year', timing:'periodic', open:true };
const sc = D.schedule(tc);
ok('open reducing: daily interest on full principal', sc.rows[1].interest === Math.round(90e6*0.194/365*30) && sc.rows[2].interest === Math.round(90e6*0.194/365*31) && sc.rows.every(r => r.principal === 0), sc.rows.slice(0,3).map(r=>r.days+':'+r.interest).join(' '));
ok('open: balance unchanged', D.loanAt(tc, '2027-06').balance === 90e6);
const pre=document.createElement('pre'); pre.id='t5'; pre.textContent='TESTS\n'+out.join('\n')+'\nENDTESTS'; document.body.append(pre); })();</script>
