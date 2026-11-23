<script>
(() => {
const out = []; const ok = (name, c, info='') => out.push((c ? 'PASS ' : 'FAIL ') + name + ' ' + info);
// 1. khoản vay lẻ ngày: vay 14/11, kỳ đầu 14/12, đáo hạn 25/2
const l = { id:'t', label:'t', principal: 100e6, start:'2026-11-14', first:'2026-12-14', end:'2027-02-25', mode:'equal', basis:'flat', timing:'periodic', rate:1.5, unit:'month', fixed:0 };
const sc = D.schedule(l);
ok('dates', JSON.stringify(sc.rows.map(r=>r.date)) === JSON.stringify(['2026-12-14','2027-01-14','2027-02-14','2027-02-25']), JSON.stringify(sc.rows.map(r=>[r.date,r.days,r.principal,r.interest])));
ok('days', sc.rows.map(r=>r.days).join() === '30,31,31,11');
const rd = 0.18/365;
ok('interest', sc.rows.every(r => r.interest === Math.round(100e6*rd*r.days)));
ok('principal sum', sc.rows.reduce((a,r)=>a+r.principal,0) === 100e6);
ok('span', sc.span === 103, sc.span);
// tháng 2 năm thường 28 ngày: kỳ 14/1→14/2 = 31, 14/2→14/3 = 28
const l2 = { ...l, end:'2027-04-14' }; const s2 = D.schedule(l2);
ok('feb28', s2.rows[3].days === 28, s2.rows.map(r=>r.days).join());
// reducing year
const l3 = { ...l, basis:'reducing', unit:'year', rate:12, mode:'bullet' }; const s3 = D.schedule(l3);
ok('reducing bullet', s3.rows[0].interest === Math.round(100e6*0.12/365*30) && s3.rows[3].principal === 100e6);
// annuity
const l4 = { ...l, basis:'reducing', mode:'annuity', end:'2027-11-14' }; const s4 = D.schedule(l4);
ok('annuity closes', s4.rows[s4.rows.length-1].after === 0 && s4.rows.length === 12, s4.rows.map(r=>r.pay).join());
// open friend loan
const l5 = { ...l, basis:'none', open:true }; const s5 = D.schedule(l5);
ok('open no rows', s5.rows.length === 0 && D.loanEvents(l5,'2026-12').length === 0);
ok('open balance', D.loanAt(l5,'2026-12').balance === 100e6);
// số dư theo ngày: mở 16/11, thu ngày 5 bị bỏ, thu ngày 20 tính, sinh hoạt 15/30 ngày
const s = { openDate:'2026-11-16', horizon:2, opening:10e6, carry:true, incomes:[{id:'a',amount:30e6,day:5},{id:'b',amount:12e6,day:20}], living:{items:[{id:'l',amount:30e6}]}, loans:[] };
const P = D.project(s);
ok('frame full month, cash from date', P[0].income === 42e6 && P[0].living === 30e6 && P[0].free === 12e6 && Math.abs(P[0].end - (10e6+12e6-15e6)) < 1, JSON.stringify([P[0].income,P[0].living,P[0].end,P[0].from]));
ok('next month full', P[1].k === '2026-12' && P[1].income === 42e6 && Math.abs(P[1].end - (P[0].end + 42e6 - 30e6)) < 1);
ok('old format migrate', true);
const pre = document.createElement('pre'); pre.id='testout'; pre.textContent = 'TESTS\n' + out.join('\n') + '\nENDTESTS'; document.body.append(pre);
})();
</script>
