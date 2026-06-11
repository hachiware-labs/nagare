pub(crate) fn serve_stylesheet() -> &'static str {
    r#":root{color-scheme:light;--bg:#f8fafc;--surface:#fff;--surface2:#f1f5f9;--text:#020617;--muted:#475569;--line:#e2e8f0;--blue:#4338ca;--green:#047857;--amber:#b45309;--red:#b91c1c}*{box-sizing:border-box}[hidden]{display:none!important}body{margin:0;background:var(--bg);color:var(--text);font:14px/1.45 Inter,"Yu Gothic UI",Meiryo,Arial,sans-serif}.app{display:grid;grid-template-columns:200px minmax(0,1fr);min-height:100vh}.sidebar{background:var(--surface);border-right:1px solid var(--line);padding:24px 18px}.brand{display:block;margin:0 0 24px}.brand-logo{display:block;width:132px;height:auto}.brand-text{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}nav a{display:block;padding:9px 14px;border-radius:7px;color:var(--muted);text-decoration:none;font-weight:700}nav a.active{background:#eef2ff;color:var(--blue)}.content{min-width:0;padding:26px 32px}.breadcrumbs{display:flex;gap:8px;align-items:center;color:var(--muted);font-size:12px;font-weight:800;margin:0 0 12px}.breadcrumbs a{padding:0;border-radius:0}.breadcrumbs span{color:var(--muted)}.topbar{display:flex;justify-content:space-between;gap:24px;align-items:flex-start;margin-bottom:22px}.actions{display:flex;gap:6px;flex-wrap:wrap;justify-content:flex-end}h1{font-size:24px;margin:0 0 4px}h2{font-size:17px;margin:0 0 12px}h3{font-size:15px;margin:0 0 10px}.muted{color:var(--muted);font-size:12px}.panel,.composer{background:var(--surface);border:1px solid var(--line);border-radius:8px;padding:20px;margin-bottom:18px}.settings-tabs{display:flex;gap:6px;flex-wrap:wrap;margin:-4px 0 18px;border-bottom:1px solid var(--line)}.settings-tab{appearance:none;border:0;border-bottom:3px solid transparent;border-radius:7px 7px 0 0;background:transparent;color:var(--muted);padding:10px 14px;font-size:12px;font-weight:800;cursor:pointer}.settings-tab.active{background:#eef2ff;border-bottom-color:var(--blue);color:var(--blue)}.settings-panel[hidden]{display:none}.primary-action{border-color:#94a3b8;box-shadow:0 1px 2px rgba(15,23,42,.06)}.queue-layout{display:block}.queue-panel{min-width:0;overflow-x:auto}.quick-composer textarea{resize:vertical}.form-grid{display:grid;grid-template-columns:1fr 1fr;gap:12px}.advanced-form{border:1px solid var(--line);border-radius:7px;padding:10px;background:#fbfdff}.advanced-form[open]{display:grid;gap:12px}.filter-panel{display:grid;grid-template-columns:1fr 1fr auto;gap:12px;align-items:start;border:1px solid var(--line);border-radius:7px;background:#fbfdff;padding:12px;margin-bottom:12px}.filter-panel h3{font-size:12px;margin:0 0 8px;color:var(--muted)}.checkbox-grid{display:flex;gap:6px;flex-wrap:wrap}.check-option{display:inline-flex;grid-auto-flow:column;align-items:center;gap:6px;width:auto;min-height:30px;border:1px solid var(--line);border-radius:7px;background:#fff;padding:5px 8px;color:var(--text);font-size:12px}.filter-actions{display:grid;gap:8px;justify-items:start}.status-strip{display:flex;gap:8px;flex-wrap:wrap;margin:0 0 12px}.queue-chip{display:inline-flex;align-items:center;gap:8px;min-height:30px;border:1px solid var(--line);border-radius:7px;background:#fff;padding:6px 10px;color:var(--muted);font-size:12px;font-weight:800;cursor:pointer}.queue-chip b{color:var(--text)}.queue-chip.active{outline:2px solid #a5b4fc;color:var(--blue)}.queue-chip.attention{border-color:#fde68a;background:#fffbeb}.queue-chip.failed{border-color:#fecaca;background:#fff7f7}.queue-chip.approval{border-color:#bfdbfe;background:#eff6ff}.queue-chip.running{border-color:#a5b4fc;background:#eef2ff}.panel-head,.event-head{display:flex;justify-content:space-between;align-items:center;gap:12px;margin-bottom:12px}.event-head{justify-content:flex-start;align-items:flex-start;margin-bottom:8px}.badge{display:inline-flex;align-items:center;min-height:24px;border-radius:12px;padding:4px 9px;font-size:11px;font-weight:800;white-space:nowrap}.blue{background:#eef2ff;color:var(--blue)}.green{background:#ecfdf5;color:var(--green)}.amber{background:#fffbeb;color:var(--amber)}.red{background:#fef2f2;color:var(--red)}.gray{background:#f1f5f9;color:var(--muted)}.button-link{display:inline-flex;align-items:center;min-height:32px;border-radius:7px;background:var(--blue);color:#fff;padding:7px 12px;font-size:12px;font-weight:800}.button-link.secondary{background:var(--surface2);border:1px solid var(--line);color:var(--blue)}form{display:grid;gap:12px;margin-top:12px}.delete-work-form,.delete-domain-form,.delete-domain-group-form{display:block;margin:0}.row-actions{display:flex;gap:6px;align-items:center;flex-wrap:wrap}.row-actions .button-link,.row-actions button{min-height:30px}label{display:grid;gap:6px;color:var(--muted);font-size:12px;font-weight:800}input,textarea,select{width:100%;border:1px solid #cbd5e1;border-radius:7px;background:#fff;color:var(--text);padding:9px;font:inherit}input[type=radio],input[type=checkbox]{width:18px;height:18px;accent-color:var(--blue);cursor:pointer}button:not(.queue-chip):not(.settings-tab){border:0;border-radius:7px;background:var(--blue);color:#fff;padding:10px 12px;font-weight:800;cursor:pointer}button.secondary-button{background:var(--surface2);border:1px solid var(--line);color:var(--blue);padding:7px 10px}button:not(.queue-chip):not(.settings-tab).danger,button.danger{background:var(--surface);border:1px solid #fecaca;color:var(--red);padding:7px 10px}table{width:100%;border-collapse:collapse}th{text-align:left;color:var(--muted);font-size:11px;padding:10px;border-bottom:1px solid var(--line)}td{padding:12px 10px;border-bottom:1px solid var(--line);vertical-align:top}tr.state-running{background:#f8faff}tr.state-needs-input,tr.state-needs-approval{background:#fffdf5}tr.state-failed{background:#fff7f7}a{color:var(--blue);font-weight:800;text-decoration:none}code{display:inline-block;max-width:100%;overflow-wrap:anywhere;background:var(--surface2);border:1px solid var(--line);border-radius:6px;padding:5px 7px;font-family:Consolas,Menlo,monospace;font-size:12px}.grid{display:grid;gap:12px}.grid.four{grid-template-columns:repeat(4,minmax(0,1fr))}.grid div{background:var(--surface2);border:1px solid var(--line);border-radius:7px;padding:12px;min-width:0}.grid b{display:block}.grid span{display:block;margin-top:6px;overflow-wrap:anywhere}.detail-layout{display:block;max-width:980px}.summary{position:static}.action-stack{min-width:0}.answer-preview{display:grid;gap:6px;min-width:220px}.answer-body{white-space:pre-wrap;font-size:15px;margin:0 0 14px}.answer-panel .detail-section{margin-top:10px}dl{display:grid;grid-template-columns:140px 1fr;gap:8px 12px;margin:0}dt{color:var(--muted);font-size:12px;font-weight:800}dd{margin:0;min-width:0;overflow-wrap:anywhere}.history-list{display:grid;gap:12px}.history-event{border:1px solid var(--line);border-radius:7px;padding:14px;background:var(--surface)}.history-event.running{border-color:#a5b4fc;background:#eef2ff}.history-event p{margin:0 0 10px}.history-step{display:inline-flex;align-items:center;min-height:24px;border-radius:12px;background:#f8fafc;border:1px solid var(--line);color:var(--muted);padding:4px 9px;font-size:11px;font-weight:800;white-space:nowrap}.history-title{display:grid;gap:2px;min-width:0;flex:1}.history-title b{font-size:15px;overflow-wrap:anywhere}.history-time{margin-left:auto;white-space:nowrap}.event-summary{color:var(--text);font-weight:700}.history-facts{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px;margin:10px 0 12px}.history-facts div{background:#f8fafc;border:1px solid var(--line);border-radius:7px;padding:9px;min-width:0}.history-facts span{display:block;color:var(--muted);font-size:11px;font-weight:800;margin-bottom:4px}.history-facts b{display:block;font-size:13px;font-weight:700;overflow-wrap:anywhere}.history-details{border-top:1px solid var(--line);padding-top:10px}.history-details[open]{display:grid;gap:12px}summary{cursor:pointer;color:var(--blue);font-weight:800}.detail-section{border:1px solid var(--line);border-radius:7px;background:#fbfdff;padding:12px}.detail-section+ .detail-section{margin-top:10px}.detail-section h3{font-size:13px;color:var(--text);margin-bottom:10px}@media(max-width:1100px){.grid.four,.form-grid,.filter-panel{grid-template-columns:1fr 1fr}.filter-actions{grid-column:1/-1}.history-facts{grid-template-columns:repeat(2,minmax(0,1fr))}.queue-layout{display:block}}@media(max-width:760px){.app{grid-template-columns:1fr}.sidebar{display:none}.content{padding:18px}.topbar{display:block}.actions{justify-content:flex-start;margin-top:12px}.settings-tabs{overflow-x:auto;flex-wrap:nowrap}.settings-tab{white-space:nowrap}.grid.four,.form-grid,.filter-panel{grid-template-columns:1fr}dl{grid-template-columns:96px 1fr}table{display:block;overflow-x:auto;white-space:nowrap}.event-head{display:grid;grid-template-columns:auto 1fr}.event-head .badge{justify-self:start}.history-time{margin-left:0}.history-facts{grid-template-columns:1fr}}"#
}

pub(crate) fn serve_responsive_stylesheet() -> &'static str {
    r#"
.panel-head .button-link{flex-shrink:0}
a:hover,.button-link:hover,nav a:hover,summary:hover{filter:brightness(.96);text-decoration:none}
button:hover:not(:disabled),.settings-tab:hover,.queue-chip:hover,.check-option:hover{filter:brightness(.97)}
a:focus-visible,button:focus-visible,input:focus-visible,textarea:focus-visible,select:focus-visible,summary:focus-visible{outline:3px solid #a5b4fc;outline-offset:2px}
input:focus-visible,textarea:focus-visible,select:focus-visible{border-color:var(--blue)}
button:disabled{cursor:not-allowed;opacity:.65}
.form-section{border-top:1px solid var(--line);padding-top:14px;display:grid;gap:12px}
.form-section-head{display:flex;justify-content:space-between;gap:12px;align-items:flex-start}
.skill-picker{display:grid;gap:10px}
.skill-search{max-width:420px}
.skill-selected{display:flex;gap:6px;flex-wrap:wrap;min-height:30px;align-items:center}
.skill-chip{display:inline-flex;align-items:center;min-height:24px;border-radius:12px;background:#eef2ff;color:var(--blue);border:1px solid #c7d2fe;padding:4px 9px;font-size:11px;font-weight:800;overflow-wrap:anywhere}
.skill-picker-list{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:8px}
.skill-option{display:grid;grid-template-columns:20px minmax(0,1fr);gap:10px;align-items:start;border:1px solid var(--line);border-radius:7px;background:#fff;padding:10px;color:var(--text);font-weight:700}
.skill-option:hover,.skill-option:focus-within{border-color:#a5b4fc;background:#f8faff}
.skill-option-body{display:grid;gap:4px;min-width:0}
.skill-option-title{display:flex;gap:8px;align-items:center;justify-content:space-between;min-width:0}
.skill-option-title span:first-child{overflow-wrap:anywhere}
.skill-option-details{color:var(--muted);font-size:12px;font-weight:500;overflow-wrap:anywhere}
.agent-table td{min-width:140px}
.agent-meta{display:flex;gap:6px;flex-wrap:wrap;margin-top:6px}
.agent-meta span{display:inline-flex;max-width:100%;border:1px solid var(--line);border-radius:12px;background:#f8fafc;color:var(--muted);padding:3px 8px;font-size:11px;font-weight:800;overflow-wrap:anywhere}
.agent-model{margin-top:7px;overflow-wrap:anywhere}
.toast-region{position:fixed;top:18px;right:18px;z-index:1000;display:grid;gap:10px;width:min(420px,calc(100vw - 32px))}
.toast{border:1px solid var(--line);border-left-width:4px;border-radius:8px;background:#fff;color:var(--text);box-shadow:0 18px 42px rgba(15,23,42,.16);padding:12px 14px;font-size:13px;line-height:1.5;white-space:pre-wrap;overflow-wrap:anywhere}
.toast.info{border-left-color:var(--blue)}
.toast.success{border-left-color:var(--green)}
.toast.error{border-left-color:var(--red);background:#fff7f7}
@media(max-width:760px){
  .sidebar{display:flex;border-right:0;border-bottom:1px solid var(--line);padding:14px 18px;align-items:center;justify-content:space-between;gap:12px}
  .brand{margin:0}
  .brand-logo{width:112px}
  nav{display:flex;gap:6px;flex-wrap:wrap;justify-content:flex-end}
  nav a{padding:8px 10px}
  .panel-head,.form-section-head{display:grid;grid-template-columns:1fr;align-items:start}
  .panel-head .button-link{justify-self:start}
  .skill-picker-list{grid-template-columns:1fr}
  .domain-table{display:table;overflow:visible;white-space:normal}
  .domain-table thead,.agent-table thead{display:none}
  .domain-table tbody,.domain-table tr,.domain-table td,.agent-table tbody,.agent-table tr,.agent-table td{display:block;width:100%}
  .domain-table tr,.agent-table tr{border:1px solid var(--line);border-radius:7px;margin:0 0 10px;padding:10px;background:#fff}
  .domain-table td,.agent-table td{display:grid;grid-template-columns:116px minmax(0,1fr);gap:10px;align-items:start;border:0;padding:7px 0;overflow-wrap:anywhere;white-space:normal}
  .domain-table td::before,.agent-table td::before{content:attr(data-label);color:var(--muted);font-size:11px;font-weight:800}
  .domain-table td:first-child,.agent-table td:first-child{padding-top:0}
  .domain-table td:last-child,.agent-table td:last-child{padding-bottom:0}
  .domain-table .row-actions,.agent-table .row-actions{justify-content:flex-start}
}
"#
}

pub(crate) fn serve_item_detail_stylesheet() -> &'static str {
    r#"
.summary .panel-head{align-items:flex-start}
.summary .panel-head h2{margin-bottom:4px}
.summary .panel-head p{margin:0}
.status-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px;margin:14px 0}
.status-card{display:grid;gap:6px;min-width:0;border:1px solid var(--line);border-radius:8px;background:#fbfdff;padding:12px}
.status-card.primary{border-color:#a5b4fc;background:#eef2ff}
.status-card.conclusion-card{grid-column:1/-1}
.status-card.conclusion-card b{font-size:18px}
.status-card span{color:var(--muted);font-size:11px;font-weight:800}
.status-card b{display:block;font-size:16px;line-height:1.35;overflow-wrap:anywhere}
.status-card small{display:block;color:var(--muted);font-size:12px;line-height:1.45;overflow-wrap:anywhere}
.summary-meta{margin-top:12px}
.detail-disclosure>summary{display:flex;justify-content:space-between;gap:12px;align-items:flex-start;color:var(--text);font-weight:800;line-height:1.35;list-style-position:inside}
.detail-disclosure>summary small{color:var(--muted);font-size:12px;font-weight:600;line-height:1.45;text-align:right;overflow-wrap:anywhere}
.detail-disclosure[open]>summary{margin-bottom:12px}
.answer-panel .answer-details{border-top:1px solid var(--line);padding-top:12px;margin-top:12px}
.answer-attention{border-color:#fde68a;background:#fffbeb}
.details-stack{display:grid;gap:12px}
.technical-details .workflow-panel{border:0;background:transparent;padding:0;margin:0}
.technical-details .workflow-panel+.workflow-panel{border-top:1px solid var(--line);padding-top:14px}
.progress-panel .panel-head h2{margin-bottom:4px}
.progress-panel .panel-head p{margin:0}
.flow-list{display:grid;grid-template-columns:repeat(auto-fit,minmax(210px,1fr));gap:12px;list-style:none;margin:14px 0 0;padding:0;counter-reset:none}
.flow-node{position:relative;display:grid;grid-template-columns:34px minmax(0,1fr);gap:10px;min-width:0;border:1px solid var(--line);border-radius:8px;background:#fbfdff;padding:12px}
.flow-node:not(:last-child)::after{content:"";position:absolute;right:-12px;top:50%;width:12px;border-top:2px solid var(--line)}
.flow-node.done{border-color:#bbf7d0;background:#f0fdf4}
.flow-node.active{border-color:#a5b4fc;background:#eef2ff}
.flow-node.blocked{border-color:#fecaca;background:#fff7f7}
.flow-node.omitted{border-style:dashed;background:#f8fafc}
.flow-marker{display:inline-flex;align-items:center;justify-content:center;width:28px;height:28px;border-radius:999px;background:#fff;border:1px solid var(--line);color:var(--muted);font-size:12px;font-weight:800}
.flow-node.done .flow-marker{border-color:#86efac;color:var(--green)}
.flow-node.active .flow-marker{border-color:#a5b4fc;color:var(--blue)}
.flow-node.blocked .flow-marker{border-color:#fecaca;color:var(--red)}
.flow-node span:not(.flow-marker){display:block;color:var(--muted);font-size:11px;font-weight:800}
.flow-node b{display:block;margin-top:4px;font-size:15px;line-height:1.35;overflow-wrap:anywhere}
.flow-node small{display:block;margin-top:6px;color:var(--muted);font-size:12px;line-height:1.45;overflow-wrap:anywhere}
.candidate-panel .panel-head h2{margin-bottom:4px}
.candidate-panel .panel-head p{margin:0}
.candidate-panel>summary span{font-size:17px}
.candidate-panel>summary small{max-width:560px}
.candidate-list{display:grid;gap:10px;margin-top:14px}
.candidate-row{border:1px solid var(--line);border-radius:8px;background:#fbfdff;padding:12px}
.candidate-head{display:flex;justify-content:space-between;gap:10px;align-items:flex-start}
.candidate-head b{overflow-wrap:anywhere}
.candidate-row p{margin:8px 0 10px;line-height:1.5;overflow-wrap:anywhere}
.candidate-row dl{grid-template-columns:88px 1fr}
.step-detail-panel .panel-head h2{margin-bottom:4px}
.step-detail-panel .panel-head p{margin:0}
.step-result{border:1px solid var(--line);border-radius:8px;background:#fbfdff;padding:10px 12px;margin:10px 0}
.step-result span{display:block;color:var(--muted);font-size:11px;font-weight:800;margin-bottom:4px}
.step-result p{margin:0;line-height:1.55;overflow-wrap:anywhere}
.step-candidate-detail{margin-top:0;margin-bottom:12px}
.step-candidate-detail h3{margin-bottom:6px}
.step-candidate-detail p{margin:0 0 8px}
@media(max-width:1100px){.status-grid{grid-template-columns:1fr 1fr}.flow-list{grid-template-columns:1fr}.flow-node:not(:last-child)::after{left:28px;right:auto;top:auto;bottom:-13px;height:13px;width:0;border-top:0;border-left:2px solid var(--line)}}
@media(max-width:760px){.status-grid{grid-template-columns:1fr}}
"#
}

pub(crate) fn serve_script() -> &'static str {
    r#"function notificationRegion(){
  let region=document.getElementById('app-notifications');
  if(region){return region;}
  region=document.createElement('div');
  region.id='app-notifications';
  region.className='toast-region';
  region.setAttribute('role','status');
  region.setAttribute('aria-live','polite');
  document.body.appendChild(region);
  return region;
}
function notify(message,kind='info'){
  const text=(message || '').trim();
  if(!text){return;}
  const toast=document.createElement('div');
  toast.className=`toast ${kind}`;
  toast.textContent=text;
  notificationRegion().appendChild(toast);
  setTimeout(()=>toast.remove(),kind==='error' ? 9000 : 4500);
}
async function responseMessage(response){
  const text=await response.text();
  if(!text){return response.statusText || 'Request failed';}
  try{
    const data=JSON.parse(text);
    return data.error || data.message || text;
  }catch(_){
    return text;
  }
}
async function notifyResponseError(response,statusEl){
  const message=await responseMessage(response);
  if(statusEl){statusEl.textContent='';}
  notify(message,'error');
}
const form=document.getElementById('create-work-form');
const statusEl=document.getElementById('form-status');
if(form){
  form.addEventListener('submit',async(event)=>{
    event.preventDefault();
    statusEl.textContent='Work Itemを追加しています…';
    const response=await fetch('/api/items',{method:'POST',body:new URLSearchParams(new FormData(form))});
    if(!response.ok){await notifyResponseError(response,statusEl);return;}
    const item=await response.json();
    statusEl.textContent='Work Itemを追加しました。バックグラウンド実行を開始しました。';
    notify('Work Itemを追加しました。','success');
    window.location.href=item.id ? `/items/${encodeURIComponent(item.id)}` : '/';
  });
}
const detailStateKey=`nagare:open-history:${window.location.pathname}`;
function openHistoryKeys(){
  try{return new Set(JSON.parse(sessionStorage.getItem(detailStateKey)||'[]'));}catch(_){return new Set();}
}
function saveOpenHistoryKeys(){
  const keys=[...document.querySelectorAll('.history-details[open]')].map((detail)=>detail.dataset.historyKey).filter(Boolean);
  sessionStorage.setItem(detailStateKey,JSON.stringify(keys));
}
const openKeys=openHistoryKeys();
document.querySelectorAll('.history-details').forEach((detail)=>{
  if(openKeys.has(detail.dataset.historyKey)){detail.open=true;}
  detail.addEventListener('toggle',saveOpenHistoryKeys);
});
const autoNextActions=new Set(['dispatch','accept_dispatch','run_agent','review','synthesize','recover','apply_recovery']);
if((document.body.dataset.nextAction && autoNextActions.has(document.body.dataset.nextAction)) || document.body.dataset.running){
  setTimeout(()=>{
    if(document.querySelector('.history-details[open]')){return;}
    window.location.reload();
  },1000);
}
const queueFilters=[...document.querySelectorAll('[data-filter-state]')];
if(queueFilters.length){
  queueFilters.forEach((filterButton)=>{
    filterButton.addEventListener('click',()=>{
      const state=filterButton.dataset.filterState;
      queueFilters.forEach((button)=>button.classList.toggle('active',button===filterButton));
      document.querySelectorAll('#work-items tr[data-queue-state]').forEach((row)=>{
        const states=(row.dataset.queueState||'').split(/\s+/);
        row.hidden=state!=='all' && !states.includes(state);
      });
    });
  });
}
const settingsTabs=[...document.querySelectorAll('[data-settings-tab]')];
const settingsPanels=[...document.querySelectorAll('[data-settings-panel]')];
if(settingsTabs.length && settingsPanels.length){
  function showSettingsTab(tab){
    const known=settingsTabs.some((button)=>button.dataset.settingsTab===tab);
    const active=known ? tab : 'workflow';
    settingsTabs.forEach((button)=>{
      const selected=button.dataset.settingsTab===active;
      button.classList.toggle('active',selected);
      button.setAttribute('aria-selected',selected ? 'true' : 'false');
      button.tabIndex=selected ? 0 : -1;
    });
    settingsPanels.forEach((panel)=>{
      const selected=panel.dataset.settingsPanel===active;
      panel.hidden=!selected;
      panel.tabIndex=selected ? 0 : -1;
    });
  }
  settingsTabs.forEach((button)=>{
    button.addEventListener('click',()=>{
      const tab=button.dataset.settingsTab;
      showSettingsTab(tab);
      history.replaceState(null,'',`#${tab}`);
    });
  });
  showSettingsTab((window.location.hash||'').replace(/^#/,''));
}
const agentFilterGroups=[...document.querySelectorAll('[data-agent-filter-group]')];
const agentFilterDomains=[...document.querySelectorAll('[data-agent-filter-domain]')];
const agentFilterDomainOptions=[...document.querySelectorAll('[data-agent-filter-domain-option]')];
const agentDomainFilterEmpty=document.querySelector('[data-agent-domain-filter-empty]');
const agentRows=[...document.querySelectorAll('[data-agent-row]')];
const agentFilterCount=document.querySelector('[data-agent-filter-count]');
const clearAgentFilters=document.querySelector('[data-clear-agent-filters]');
function selectedValues(inputs){
  return inputs.filter((input)=>input.checked).map((input)=>input.value);
}
function rowHasAny(row,attr,values){
  if(!values.length){return true;}
  const rowValues=(row.dataset[attr]||'').split(/\s+/).filter(Boolean);
  return values.some((value)=>rowValues.includes(value));
}
function syncDomainFilterOptions(groups){
  const hasGroupFilter=groups.length>0;
  let visibleOptions=0;
  agentFilterDomainOptions.forEach((option)=>{
    const optionGroup=option.dataset.domainGroup || '';
    const show=hasGroupFilter && groups.includes(optionGroup);
    option.hidden=!show;
    const input=option.querySelector('input[type="checkbox"]');
    if(input){
      input.disabled=!show;
      if(!show){input.checked=false;}
    }
    if(show){visibleOptions+=1;}
  });
  if(agentDomainFilterEmpty){
    agentDomainFilterEmpty.hidden=visibleOptions>0;
  }
}
function applyAgentFilters(){
  const groups=selectedValues(agentFilterGroups);
  syncDomainFilterOptions(groups);
  const domains=selectedValues(agentFilterDomains);
  let visible=0;
  agentRows.forEach((row)=>{
    const show=rowHasAny(row,'agentDomainGroups',groups) && rowHasAny(row,'agentDomains',domains);
    row.hidden=!show;
    if(show){visible+=1;}
  });
  if(agentFilterCount){
    const active=groups.length+domains.length;
    agentFilterCount.textContent=active ? `${visible}件のエージェントを表示中` : '';
  }
}
if(agentRows.length){
  [...agentFilterGroups,...agentFilterDomains].forEach((input)=>input.addEventListener('change',applyAgentFilters));
  if(clearAgentFilters){
    clearAgentFilters.addEventListener('click',()=>{
      [...agentFilterGroups,...agentFilterDomains].forEach((input)=>{input.checked=false;});
      applyAgentFilters();
    });
  }
  applyAgentFilters();
}
document.querySelectorAll('.delete-work-form').forEach((deleteForm)=>{
  deleteForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    const workId=deleteForm.dataset.workId;
    const title=deleteForm.dataset.workTitle || workId;
    if(!confirm(`Work Item「${title}」を削除しますか？`)){return;}
    const button=deleteForm.querySelector('button');
    button.disabled=true;
    button.textContent='削除中…';
    const response=await fetch(`/api/items/${workId}/delete`,{method:'POST'});
    if(!response.ok){button.disabled=false;button.textContent='削除';await notifyResponseError(response);return;}
    deleteForm.closest('tr').remove();
  });
});
document.querySelectorAll('.delete-domain-group-form').forEach((deleteForm)=>{
  deleteForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    const groupId=deleteForm.dataset.domainGroupId;
    const name=deleteForm.dataset.domainGroupName || groupId;
    if(!confirm(`ドメイングループ「${name}」を削除しますか？`)){return;}
    const button=deleteForm.querySelector('button');
    button.disabled=true;
    button.textContent='削除中…';
    const response=await fetch(`/api/domain-groups/${groupId}/delete`,{method:'POST'});
    if(!response.ok){button.disabled=false;button.textContent='削除';await notifyResponseError(response);return;}
    deleteForm.closest('tr').remove();
  });
});
document.querySelectorAll('.delete-domain-form').forEach((deleteForm)=>{
  deleteForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    const domainId=deleteForm.dataset.domainId;
    const name=deleteForm.dataset.domainName || domainId;
    if(!confirm(`ドメイン「${name}」を削除しますか？`)){return;}
    const button=deleteForm.querySelector('button');
    button.disabled=true;
    button.textContent='削除中…';
    const response=await fetch(`/api/domains/${domainId}/delete`,{method:'POST'});
    if(!response.ok){button.disabled=false;button.textContent='削除';await notifyResponseError(response);return;}
    deleteForm.closest('tr').remove();
  });
});
const agentProfileForm=document.getElementById('agent-profile-form');
if(agentProfileForm){
  const agentProfileStatus=document.getElementById('agent-profile-status');
  const kindSelect=document.getElementById('agent-kind');
  const runtimeInput=agentProfileForm.querySelector('input[name="runtime"]');
  const adapterInput=agentProfileForm.querySelector('input[name="adapter"]');
  const idInput=agentProfileForm.querySelector('input[name="id"]');
  const externalProviderInput=agentProfileForm.querySelector('input[name="external_provider"]');
  const externalAgentIdInput=document.getElementById('external-agent-id');
  const modelSection=agentProfileForm.querySelector('[data-model-section="model"]');
  const providerField=agentProfileForm.querySelector('[data-model-field="provider"]');
  const baseUrlField=agentProfileForm.querySelector('[data-model-field="base-url"]');
  const providerInput=document.getElementById('openclaw-model-provider');
  const modelInput=agentProfileForm.querySelector('input[name="model_id"]');
  const baseUrlInput=agentProfileForm.querySelector('input[name="base_url"]');
  const apiKeyEnvInput=agentProfileForm.querySelector('input[name="api_key_env"]');
  const agentDomainGroupSelect=document.getElementById('agent-domain-group');
  const agentDomainSelect=document.getElementById('agent-domain');
  function isOpenClawAgent(){
    return kindSelect.value==='openclaw';
  }
  function setHidden(element, hidden){
    if(element){element.hidden=hidden;}
  }
  function syncExternalAgentId(){
    if(externalAgentIdInput && idInput){externalAgentIdInput.value=idInput.value.trim();}
  }
  function syncModelFields(){
    const isOpenClaw=isOpenClawAgent();
    setHidden(modelSection,false);
    setHidden(providerField,!isOpenClaw);
    setHidden(baseUrlField,!isOpenClaw || providerInput.value==='openai-codex' || providerInput.value==='openai');
    modelInput.required=false;
    baseUrlInput.required=false;
    if(!isOpenClaw){
      providerInput.value='';
      baseUrlInput.value='';
      apiKeyEnvInput.value='';
      modelInput.removeAttribute('list');
      modelInput.placeholder='gpt-5.3-codex';
      return;
    }
    if(!providerInput.value){providerInput.value='openai-codex';}
    modelInput.required=true;
    if(providerInput.value==='openai-codex' || providerInput.value==='openai'){
      modelInput.setAttribute('list','openai-model-options');
      modelInput.placeholder='gpt-5.3-codex';
      if(!modelInput.value){modelInput.value='gpt-5.3-codex';}
      baseUrlInput.value='';
      apiKeyEnvInput.value='';
    }else{
      modelInput.removeAttribute('list');
      modelInput.placeholder=providerInput.value==='ollama' ? 'llama3.2' : 'loaded-model-name';
      if(providerInput.value==='ollama' && !baseUrlInput.value){baseUrlInput.value='http://127.0.0.1:11434/v1';}
      if(providerInput.value==='lmstudio' && !baseUrlInput.value){baseUrlInput.value='http://127.0.0.1:1234/v1';}
      baseUrlInput.required=true;
      apiKeyEnvInput.value='';
    }
  }
  function scrubModelFieldsForSubmit(){
    syncExternalAgentId();
    if(!isOpenClawAgent()){
      providerInput.value='';
      baseUrlInput.value='';
      apiKeyEnvInput.value='';
      return;
    }
    if(providerInput.value==='openai-codex' || providerInput.value==='openai'){
      baseUrlInput.value='';
      apiKeyEnvInput.value='';
    }
  }
  function syncAgentKind(){
    if(kindSelect.value==='codex_app_server'){
      runtimeInput.value='codex-app-local';
      adapterInput.value='stdio.codex-app-server';
      externalProviderInput.value='codex';
    }else if(kindSelect.value==='openclaw'){
      runtimeInput.value='openclaw-local';
      adapterInput.value='process.openclaw-agent';
      externalProviderInput.value='openclaw';
    }else{
      runtimeInput.value='codex-local';
      adapterInput.value='process.codex-cli';
      externalProviderInput.value='codex-cli';
    }
    syncExternalAgentId();
    syncModelFields();
  }
  function syncAgentDomainOptions(){
    if(!agentDomainGroupSelect || !agentDomainSelect){return;}
    const group=agentDomainGroupSelect.value;
    let selectedStillVisible=false;
    [...agentDomainSelect.options].forEach((option)=>{
      if(!option.value){
        option.hidden=false;
        option.disabled=false;
        return;
      }
      const show=Boolean(group) && option.dataset.domainGroup===group;
      option.hidden=!show;
      option.disabled=!show;
      if(show && option.selected){selectedStillVisible=true;}
    });
    agentDomainSelect.disabled=!group;
    if(!selectedStillVisible){agentDomainSelect.value='';}
  }
  kindSelect.addEventListener('change',syncAgentKind);
  if(idInput){idInput.addEventListener('input',syncExternalAgentId);}
  if(providerInput){providerInput.addEventListener('change',syncModelFields);}
  if(agentDomainGroupSelect){agentDomainGroupSelect.addEventListener('change',syncAgentDomainOptions);}
  syncAgentKind();
  syncAgentDomainOptions();
  function escapeHtml(value){
    return value.replace(/[&<>"']/g,(char)=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
  }
  document.querySelectorAll('[data-skill-picker]').forEach((picker)=>{
    const searchInput=picker.querySelector('[data-skill-search]');
    const selectedEl=picker.querySelector('[data-skill-selected]');
    const options=[...picker.querySelectorAll('[data-skill-option]')];
    function renderSelectedSkills(){
      const selected=options
        .filter((option)=>option.querySelector('input[type="checkbox"]').checked)
        .map((option)=>option.querySelector('.skill-option-title span').textContent.trim());
      selectedEl.innerHTML=selected.length
        ? selected.map((name)=>`<span class="skill-chip" translate="no">${escapeHtml(name)}</span>`).join('')
        : `<span class="muted">${escapeHtml(picker.dataset.emptyLabel || 'No skills selected')}</span>`;
    }
    function filterSkills(){
      const query=(searchInput.value || '').trim().toLowerCase();
      options.forEach((option)=>{
        option.hidden=query && !(option.dataset.skillSearchText || '').includes(query);
      });
    }
    options.forEach((option)=>option.querySelector('input[type="checkbox"]').addEventListener('change',renderSelectedSkills));
    if(searchInput){searchInput.addEventListener('input',filterSkills);}
    renderSelectedSkills();
  });
  agentProfileForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    agentProfileStatus.textContent='エージェントを保存しています…';
    syncAgentKind();
    scrubModelFieldsForSubmit();
    const response=await fetch(agentProfileForm.dataset.action,{method:'POST',body:new URLSearchParams(new FormData(agentProfileForm))});
    if(!response.ok){await notifyResponseError(response,agentProfileStatus);return;}
    agentProfileStatus.textContent='エージェントを保存しました。';
    window.location.href=agentProfileForm.dataset.redirect || '/settings';
  });
  const deleteAgentButton=document.getElementById('delete-agent-button');
  if(deleteAgentButton){
    deleteAgentButton.addEventListener('click',async()=>{
      const name=deleteAgentButton.dataset.agentName || 'このエージェント';
      if(!confirm(`エージェント「${name}」を削除しますか？`)){return;}
      deleteAgentButton.disabled=true;
      deleteAgentButton.textContent='削除中…';
      const response=await fetch(deleteAgentButton.dataset.action,{method:'POST'});
      if(!response.ok){deleteAgentButton.disabled=false;deleteAgentButton.textContent='エージェントを削除';await notifyResponseError(response,agentProfileStatus);return;}
      window.location.href=agentProfileForm.dataset.redirect || '/settings';
    });
  }
}
const domainProfileForm=document.getElementById('domain-profile-form');
if(domainProfileForm){
  const domainProfileStatus=document.getElementById('domain-profile-status');
  domainProfileForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    domainProfileStatus.textContent='ドメインを保存しています…';
    const response=await fetch(domainProfileForm.dataset.action,{method:'POST',body:new URLSearchParams(new FormData(domainProfileForm))});
    if(!response.ok){await notifyResponseError(response,domainProfileStatus);return;}
    domainProfileStatus.textContent='ドメインを保存しました。';
    window.location.href=domainProfileForm.dataset.redirect || '/settings';
  });
}
const domainGroupForm=document.getElementById('domain-group-form');
if(domainGroupForm){
  const domainGroupStatus=document.getElementById('domain-group-status');
  domainGroupForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    domainGroupStatus.textContent='ドメイングループを保存しています…';
    const response=await fetch(domainGroupForm.dataset.action,{method:'POST',body:new URLSearchParams(new FormData(domainGroupForm))});
    if(!response.ok){await notifyResponseError(response,domainGroupStatus);return;}
    domainGroupStatus.textContent='ドメイングループを保存しました。';
    window.location.href=domainGroupForm.dataset.redirect || '/settings';
  });
}
const workflowSettingsForm=document.getElementById('workflow-settings-form');
if(workflowSettingsForm){
  const workflowSettingsStatus=document.getElementById('workflow-settings-status');
  workflowSettingsForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    workflowSettingsStatus.textContent='ワークフロー設定を保存しています…';
    const response=await fetch(workflowSettingsForm.dataset.action,{method:'POST',body:new URLSearchParams(new FormData(workflowSettingsForm))});
    if(!response.ok){await notifyResponseError(response,workflowSettingsStatus);return;}
    workflowSettingsStatus.textContent='ワークフロー設定を保存しました。';
  });
}
const skillPackageForm=document.getElementById('skill-package-form');
if(skillPackageForm){
  const skillPackageStatus=document.getElementById('skill-package-status');
  const skillSourceKind=document.getElementById('skill-source-kind');
  const skillSourceFields=[...skillPackageForm.querySelectorAll('[data-skill-source-field]')];
  function setFieldVisible(field, visible){
    field.hidden=!visible;
    field.querySelectorAll('input,textarea,select').forEach((input)=>{
      input.disabled=!visible;
      input.required=false;
    });
  }
  function syncSkillSourceFields(){
    const kind=skillSourceKind ? skillSourceKind.value : 'skill-creator';
    const visibleByKind={
      'skill-creator':['path'],
      'local':['path'],
      'clawhub':['source','reference','checksum'],
      'vercel':['source','reference','checksum'],
      'git':['source','path','reference','checksum'],
    };
    const visible=new Set(visibleByKind[kind] || ['source']);
    skillSourceFields.forEach((field)=>setFieldVisible(field,visible.has(field.dataset.skillSourceField)));
    const sourceInput=skillPackageForm.querySelector('input[name="source"]');
    const pathInput=skillPackageForm.querySelector('input[name="path"]');
    if(sourceInput){sourceInput.required=visible.has('source');}
    if(pathInput){pathInput.required=visible.has('path') && (kind==='skill-creator' || kind==='local');}
  }
  if(skillSourceKind){skillSourceKind.addEventListener('change',syncSkillSourceFields);}
  syncSkillSourceFields();
  skillPackageForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    skillPackageStatus.textContent='スキルを登録しています…';
    const response=await fetch(skillPackageForm.dataset.action,{method:'POST',body:new URLSearchParams(new FormData(skillPackageForm))});
    if(!response.ok){await notifyResponseError(response,skillPackageStatus);return;}
    skillPackageStatus.textContent='スキルを登録しました。';
    window.location.href=skillPackageForm.dataset.redirect || '/settings#agents';
  });
}
const answerForm=document.getElementById('answer-form');
if(answerForm){
  const answerStatus=document.getElementById('answer-status');
  answerForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    answerStatus.textContent='回答を保存しています…';
    const workId=answerForm.dataset.workId;
    const response=await fetch(`/api/items/${workId}/answer`,{method:'POST',body:new URLSearchParams(new FormData(answerForm))});
    if(!response.ok){await notifyResponseError(response,answerStatus);return;}
    answerStatus.textContent='回答を保存しました。バックグラウンド実行を開始しました。';
    window.location.reload();
  });
}
const runForm=document.getElementById('run-form');
if(runForm){
  const runStatus=document.getElementById('run-status');
  runForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    runStatus.textContent='実行中…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/run`,{method:'POST',body:new URLSearchParams(new FormData(runForm))});
    if(!response.ok){await notifyResponseError(response,runStatus);return;}
    runStatus.textContent='実行が完了しました。';
    window.location.reload();
  });
}
const dispatchForm=document.getElementById('dispatch-form');
if(dispatchForm){
  const dispatchStatus=document.getElementById('dispatch-status');
  dispatchForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    dispatchStatus.textContent='Dispatch中…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/preview`,{method:'POST',body:new URLSearchParams(new FormData(dispatchForm))});
    if(!response.ok){await notifyResponseError(response,dispatchStatus);return;}
    dispatchStatus.textContent='Dispatchが完了しました。';
    window.location.reload();
  });
}
const dispatchAcceptForm=document.getElementById('dispatch-accept-form');
if(dispatchAcceptForm){
  const dispatchAcceptStatus=document.getElementById('dispatch-accept-status');
  dispatchAcceptForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    dispatchAcceptStatus.textContent='Dispatch planを承認しています…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/dispatch/accept`,{method:'POST',body:new URLSearchParams(new FormData(dispatchAcceptForm))});
    if(!response.ok){await notifyResponseError(response,dispatchAcceptStatus);return;}
    dispatchAcceptStatus.textContent='Dispatch planを承認しました。';
    window.location.reload();
  });
}
const reviewForm=document.getElementById('review-form');
if(reviewForm){
  const reviewStatus=document.getElementById('review-status');
  reviewForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    reviewStatus.textContent='レビュー中…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/review`,{method:'POST',body:new URLSearchParams(new FormData(reviewForm))});
    if(!response.ok){await notifyResponseError(response,reviewStatus);return;}
    reviewStatus.textContent='レビューが完了しました。';
    window.location.reload();
  });
}
const synthesisForm=document.getElementById('synthesis-form');
if(synthesisForm){
  const synthesisStatus=document.getElementById('synthesis-status');
  synthesisForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    synthesisStatus.textContent='統合サマリーを作成中…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/advance`,{method:'POST',body:new URLSearchParams(new FormData(synthesisForm))});
    if(!response.ok){await notifyResponseError(response,synthesisStatus);return;}
    synthesisStatus.textContent='統合サマリーを作成しました。';
    window.location.reload();
  });
}
const approveForm=document.getElementById('approve-form');
if(approveForm){
  const approveStatus=document.getElementById('approve-status');
  approveForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    approveStatus.textContent='承認中…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/approve`,{method:'POST',body:new URLSearchParams(new FormData(approveForm))});
    if(!response.ok){await notifyResponseError(response,approveStatus);return;}
    approveStatus.textContent='承認しました。';
    window.location.reload();
  });
}
const rejectForm=document.getElementById('reject-form');
if(rejectForm){
  const rejectStatus=document.getElementById('reject-status');
  rejectForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    rejectStatus.textContent='差し戻し中…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/reject`,{method:'POST',body:new URLSearchParams(new FormData(rejectForm))});
    if(!response.ok){await notifyResponseError(response,rejectStatus);return;}
    rejectStatus.textContent='差し戻しました。次はDispatchです。';
    window.location.reload();
  });
}
const recoverForm=document.getElementById('recover-form');
if(recoverForm){
  const recoverStatus=document.getElementById('recover-status');
  recoverForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    recoverStatus.textContent='Recovery planを作成しています…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/recover`,{method:'POST',body:new URLSearchParams(new FormData(recoverForm))});
    if(!response.ok){await notifyResponseError(response,recoverStatus);return;}
    recoverStatus.textContent='Recovery planを作成しました。';
    window.location.reload();
  });
}
const recoverAcceptForm=document.getElementById('recover-accept-form');
if(recoverAcceptForm){
  const recoverAcceptStatus=document.getElementById('recover-accept-status');
  recoverAcceptForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    recoverAcceptStatus.textContent='Recovery planを承認しています…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/recover/accept`,{method:'POST',body:new URLSearchParams(new FormData(recoverAcceptForm))});
    if(!response.ok){await notifyResponseError(response,recoverAcceptStatus);return;}
    recoverAcceptStatus.textContent='Recovery planを承認しました。';
    window.location.reload();
  });
}
const recoverApplyForm=document.getElementById('recover-apply-form');
if(recoverApplyForm){
  const recoverApplyStatus=document.getElementById('recover-apply-status');
  recoverApplyForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    recoverApplyStatus.textContent='Recovery planを適用しています…';
    const workId=window.location.pathname.split('/').pop();
    const response=await fetch(`/api/items/${workId}/recover/apply`,{method:'POST',body:new URLSearchParams(new FormData(recoverApplyForm))});
    if(!response.ok){await notifyResponseError(response,recoverApplyStatus);return;}
    recoverApplyStatus.textContent='Recovery planを適用しました。';
    window.location.reload();
  });
}"#
}
