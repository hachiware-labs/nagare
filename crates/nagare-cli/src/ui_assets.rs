pub(crate) fn serve_stylesheet() -> &'static str {
    r#":root{color-scheme:light;--bg:#f8fafc;--surface:#fff;--surface2:#f1f5f9;--text:#020617;--muted:#475569;--line:#e2e8f0;--blue:#4338ca;--green:#047857;--amber:#b45309;--red:#b91c1c}*{box-sizing:border-box}[hidden]{display:none!important}body{margin:0;background:var(--bg);color:var(--text);font:14px/1.45 Inter,"Yu Gothic UI",Meiryo,Arial,sans-serif}.app{display:grid;grid-template-columns:200px minmax(0,1fr);min-height:100vh}.sidebar{background:var(--surface);border-right:1px solid var(--line);padding:24px 18px}.brand{display:block;margin:0 0 24px}.brand-logo{display:block;width:132px;height:auto}.brand-text{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}nav a{display:block;padding:9px 14px;border-radius:7px;color:var(--muted);text-decoration:none;font-weight:700}nav a.active{background:#eef2ff;color:var(--blue)}.content{min-width:0;padding:26px 32px}.breadcrumbs{display:flex;gap:8px;align-items:center;color:var(--muted);font-size:12px;font-weight:800;margin:0 0 12px}.breadcrumbs a{padding:0;border-radius:0}.breadcrumbs span{color:var(--muted)}.topbar{display:flex;justify-content:space-between;gap:24px;align-items:flex-start;margin-bottom:22px}.actions{display:flex;gap:6px;flex-wrap:wrap;justify-content:flex-end}h1{font-size:24px;margin:0 0 4px}h2{font-size:17px;margin:0 0 12px}h3{font-size:15px;margin:0 0 10px}.muted{color:var(--muted);font-size:12px}.panel,.composer{background:var(--surface);border:1px solid var(--line);border-radius:8px;padding:20px;margin-bottom:18px}.settings-tabs{display:flex;gap:6px;flex-wrap:wrap;margin:-4px 0 18px;border-bottom:1px solid var(--line)}.settings-tab{appearance:none;border:0;border-bottom:3px solid transparent;border-radius:7px 7px 0 0;background:transparent;color:var(--muted);padding:10px 14px;font-size:12px;font-weight:800;cursor:pointer}.settings-tab.active{background:#eef2ff;border-bottom-color:var(--blue);color:var(--blue)}.settings-panel[hidden]{display:none}.primary-action{border-color:#94a3b8;box-shadow:0 1px 2px rgba(15,23,42,.06)}.queue-layout{display:block}.queue-panel{min-width:0;overflow-x:auto}.quick-composer textarea{resize:vertical}.form-grid{display:grid;grid-template-columns:1fr 1fr;gap:12px}.advanced-form{border:1px solid var(--line);border-radius:7px;padding:10px;background:#fbfdff}.advanced-form[open]{display:grid;gap:12px}.filter-panel{display:grid;grid-template-columns:1fr 1fr auto;gap:12px;align-items:start;border:1px solid var(--line);border-radius:7px;background:#fbfdff;padding:12px;margin-bottom:12px}.filter-panel h3{font-size:12px;margin:0 0 8px;color:var(--muted)}.checkbox-grid{display:flex;gap:6px;flex-wrap:wrap}.check-option{display:inline-flex;grid-auto-flow:column;align-items:center;gap:6px;width:auto;min-height:30px;border:1px solid var(--line);border-radius:7px;background:#fff;padding:5px 8px;color:var(--text);font-size:12px}.filter-actions{display:grid;gap:8px;justify-items:start}.status-strip{display:flex;gap:8px;flex-wrap:wrap;margin:0 0 12px}.queue-chip{display:inline-flex;align-items:center;gap:8px;min-height:30px;border:1px solid var(--line);border-radius:7px;background:#fff;padding:6px 10px;color:var(--muted);font-size:12px;font-weight:800;cursor:pointer}.queue-chip b{color:var(--text)}.queue-chip.active{outline:2px solid #a5b4fc;color:var(--blue)}.queue-chip.attention{border-color:#fde68a;background:#fffbeb}.queue-chip.failed{border-color:#fecaca;background:#fff7f7}.queue-chip.approval{border-color:#bfdbfe;background:#eff6ff}.queue-chip.running{border-color:#a5b4fc;background:#eef2ff}.panel-head,.event-head{display:flex;justify-content:space-between;align-items:center;gap:12px;margin-bottom:12px}.event-head{justify-content:flex-start;align-items:flex-start;margin-bottom:8px}.badge{display:inline-flex;align-items:center;min-height:24px;border-radius:12px;padding:4px 9px;font-size:11px;font-weight:800;white-space:nowrap}.blue{background:#eef2ff;color:var(--blue)}.green{background:#ecfdf5;color:var(--green)}.amber{background:#fffbeb;color:var(--amber)}.red{background:#fef2f2;color:var(--red)}.gray{background:#f1f5f9;color:var(--muted)}.button-link{display:inline-flex;align-items:center;min-height:32px;border-radius:7px;background:var(--blue);color:#fff;padding:7px 12px;font-size:12px;font-weight:800}.button-link.secondary{background:var(--surface2);border:1px solid var(--line);color:var(--blue)}form{display:grid;gap:12px;margin-top:12px}.delete-work-form,.delete-domain-form,.delete-domain-group-form{display:block;margin:0}.row-actions{display:flex;gap:6px;align-items:center;flex-wrap:wrap}.row-actions .button-link,.row-actions button{min-height:30px}label{display:grid;gap:6px;color:var(--muted);font-size:12px;font-weight:800}input,textarea,select{width:100%;border:1px solid #cbd5e1;border-radius:7px;background:#fff;color:var(--text);padding:9px;font:inherit}input[type=radio],input[type=checkbox]{width:18px;height:18px;accent-color:var(--blue);cursor:pointer}button:not(.queue-chip):not(.settings-tab){border:0;border-radius:7px;background:var(--blue);color:#fff;padding:10px 12px;font-weight:800;cursor:pointer}button.secondary-button{background:var(--surface2);border:1px solid var(--line);color:var(--blue);padding:7px 10px}button:not(.queue-chip):not(.settings-tab).danger,button.danger{background:var(--surface);border:1px solid #fecaca;color:var(--red);padding:7px 10px}table{width:100%;border-collapse:collapse}th{text-align:left;color:var(--muted);font-size:11px;padding:10px;border-bottom:1px solid var(--line)}td{padding:12px 10px;border-bottom:1px solid var(--line);vertical-align:top}tr.state-running{background:#f8faff}tr.state-needs-input,tr.state-needs-approval{background:#fffdf5}tr.state-failed{background:#fff7f7}a{color:var(--blue);font-weight:800;text-decoration:none}code{display:inline-block;max-width:100%;overflow-wrap:anywhere;background:var(--surface2);border:1px solid var(--line);border-radius:6px;padding:5px 7px;font-family:Consolas,Menlo,monospace;font-size:12px}.grid{display:grid;gap:12px}.grid.four{grid-template-columns:repeat(4,minmax(0,1fr))}.grid div{background:var(--surface2);border:1px solid var(--line);border-radius:7px;padding:12px;min-width:0}.grid b{display:block}.grid span{display:block;margin-top:6px;overflow-wrap:anywhere}.detail-layout{display:block;max-width:980px}.summary{position:static}.action-stack{min-width:0}.answer-preview{display:grid;gap:6px;min-width:220px}.answer-body{white-space:pre-wrap;font-size:15px;margin:0 0 14px}.answer-panel .detail-section{margin-top:10px}dl{display:grid;grid-template-columns:140px 1fr;gap:8px 12px;margin:0}dt{color:var(--muted);font-size:12px;font-weight:800}dd{margin:0;min-width:0;overflow-wrap:anywhere}.history-list{display:grid;gap:12px}.history-event{border:1px solid var(--line);border-radius:7px;padding:14px;background:var(--surface)}.history-event.running{border-color:#a5b4fc;background:#eef2ff}.history-event p{margin:0 0 10px}.history-step{display:inline-flex;align-items:center;min-height:24px;border-radius:12px;background:#f8fafc;border:1px solid var(--line);color:var(--muted);padding:4px 9px;font-size:11px;font-weight:800;white-space:nowrap}.history-title{display:grid;gap:2px;min-width:0;flex:1}.history-title b{font-size:15px;overflow-wrap:anywhere}.history-time{margin-left:auto;white-space:nowrap}.event-summary{color:var(--text);font-weight:700}.history-facts{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px;margin:10px 0 12px}.history-facts div{background:#f8fafc;border:1px solid var(--line);border-radius:7px;padding:9px;min-width:0}.history-facts span{display:block;color:var(--muted);font-size:11px;font-weight:800;margin-bottom:4px}.history-facts b{display:block;font-size:13px;font-weight:700;overflow-wrap:anywhere}.history-details{border-top:1px solid var(--line);padding-top:10px}.history-details[open]{display:grid;gap:12px}summary{cursor:pointer;color:var(--blue);font-weight:800}.detail-section{border:1px solid var(--line);border-radius:7px;background:#fbfdff;padding:12px}.detail-section+ .detail-section{margin-top:10px}.detail-section h3{font-size:13px;color:var(--text);margin-bottom:10px}@media(max-width:1100px){.grid.four,.form-grid,.filter-panel{grid-template-columns:1fr 1fr}.filter-actions{grid-column:1/-1}.history-facts{grid-template-columns:repeat(2,minmax(0,1fr))}.queue-layout{display:block}}@media(max-width:760px){.app{grid-template-columns:1fr}.sidebar{display:none}.content{padding:18px}.topbar{display:block}.actions{justify-content:flex-start;margin-top:12px}.settings-tabs{overflow-x:auto;flex-wrap:nowrap}.settings-tab{white-space:nowrap}.grid.four,.form-grid,.filter-panel{grid-template-columns:1fr}dl{grid-template-columns:96px 1fr}table{display:block;overflow-x:auto;white-space:nowrap}.event-head{display:grid;grid-template-columns:auto 1fr}.event-head .badge{justify-self:start}.history-time{margin-left:0}.history-facts{grid-template-columns:1fr}}"#
}

pub(crate) fn serve_responsive_stylesheet() -> &'static str {
    r#"
.panel-head .button-link{flex-shrink:0}
button{font:inherit}
a:hover,.button-link:hover,nav a:hover,summary:hover{filter:brightness(.96);text-decoration:none}
button:hover:not(:disabled),.settings-tab:hover,.queue-chip:hover,.check-option:hover{filter:brightness(.97)}
a:focus-visible,button:focus-visible,input:focus-visible,textarea:focus-visible,select:focus-visible,summary:focus-visible{outline:3px solid #a5b4fc;outline-offset:2px}
input:focus-visible,textarea:focus-visible,select:focus-visible{border-color:var(--blue)}
button:disabled{cursor:not-allowed;opacity:.65}
.visually-hidden{position:absolute!important;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}
.field-group-head{display:grid;gap:4px;margin:2px 0 8px}
.field-group-head h2{font-size:15px;margin:0}
.field-group-head p{margin:0}
.routing-preview{display:grid;gap:8px;border:1px solid #c7d2fe;border-radius:8px;background:#eef2ff;padding:12px;color:var(--text)}
.routing-preview.compact{background:#fbfdff;border-color:var(--line)}
.routing-preview div{display:grid;gap:3px}
.routing-preview span{color:var(--muted);font-size:11px;font-weight:800}
.routing-preview b{font-size:15px;line-height:1.35;overflow-wrap:anywhere}
.routing-preview p{margin:0;line-height:1.5;overflow-wrap:anywhere}
.routing-preview small{color:var(--muted);line-height:1.45;overflow-wrap:anywhere}
.queue-card-list{display:none}
.queue-card{border:1px solid var(--line);border-radius:8px;background:#fff;padding:12px}
.queue-card+.queue-card{margin-top:10px}
.queue-card.empty{padding:16px}
.queue-card-head{display:flex;justify-content:space-between;gap:10px;align-items:flex-start}
.queue-card-head p{margin:4px 0 0}
.queue-card h3{font-size:15px;margin:10px 0;line-height:1.35;overflow-wrap:anywhere}
.queue-card-answer{margin:0 0 10px}
.queue-card-meta{display:grid;grid-template-columns:1fr 1fr;gap:8px;margin:10px 0}
.queue-card-meta div{border:1px solid var(--line);border-radius:7px;background:#f8fafc;padding:8px;min-width:0}
.queue-card-meta dt{font-size:11px}
.queue-card-actions{display:flex;justify-content:flex-end;margin-top:8px}
.source-choice-section{display:grid;gap:10px}
.source-choice-grid{display:grid;grid-template-columns:repeat(5,minmax(0,1fr));gap:8px}
.source-choice{display:grid!important;gap:5px;align-content:start;min-height:86px;border:1px solid var(--line)!important;border-radius:8px!important;background:#fff!important;color:var(--text)!important;text-align:left;padding:10px!important}
.source-choice b{font-size:13px;line-height:1.3}
.source-choice span{color:var(--muted);font-size:12px;line-height:1.4;font-weight:600}
.source-choice.active,.source-choice[aria-pressed="true"]{border-color:#a5b4fc!important;background:#eef2ff!important;color:var(--blue)!important}
.source-choice.active span,.source-choice[aria-pressed="true"] span{color:var(--text)}
.form-section{border-top:1px solid var(--line);padding-top:14px;display:grid;gap:12px}
.form-section-head{display:flex;justify-content:space-between;gap:12px;align-items:flex-start}
.skill-picker{display:grid;gap:10px}
.skill-search{max-width:420px}
.skill-selected{display:flex;gap:6px;flex-wrap:wrap;min-height:30px;align-items:center}
.skill-chip-group{display:inline-flex;align-items:center;gap:4px;flex-wrap:wrap}
.skill-chip{display:inline-flex;align-items:center;min-height:24px;border-radius:12px;background:#eef2ff;color:var(--blue);border:1px solid #c7d2fe;padding:4px 9px;font-size:11px;font-weight:800;overflow-wrap:anywhere}
.skill-selected button.skill-chip{gap:6px;background:#eef2ff!important;color:var(--blue)!important;border:1px solid #c7d2fe!important;padding:4px 8px!important;min-height:24px;border-radius:12px!important;font-size:11px!important}
.skill-selected button.skill-chip span{display:inline-flex;align-items:center;justify-content:center;width:16px;height:16px;border-radius:50%;background:#dbeafe;color:var(--blue);font-size:11px;line-height:1}
.skill-selected button.skill-chip-uninstall{background:#fff7f7!important;color:var(--red)!important;border-color:#fecaca!important}
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
body:has(.flow-app){background:#f8fafc}
.flow-app{font-family:"Segoe UI","Yu Gothic UI",Meiryo,Arial,sans-serif;font-synthesis-weight:none;text-rendering:geometricPrecision}
.flow-app button,.flow-app input,.flow-app textarea,.flow-app select{font-family:inherit;font-synthesis-weight:none;text-shadow:none}
.flow-app label{font-weight:700}
.flow-app form>label,.runtime-settings-card label,.work-filter-panel label.filter-project,.work-filter-panel label.filter-keyword{gap:8px;align-content:start;min-width:0;border:1px solid rgba(148,163,184,.3);border-radius:8px;background:linear-gradient(90deg,#fff,#fbfdff);padding:12px;box-shadow:inset 0 1px 0 rgba(255,255,255,.9);transition:border-color .18s ease,box-shadow .22s ease,transform .18s cubic-bezier(.2,.8,.2,1)}
.flow-app form>label:focus-within,.runtime-settings-card label:focus-within,.work-filter-panel label.filter-project:focus-within,.work-filter-panel label.filter-keyword:focus-within{border-color:rgba(148,163,184,.3);box-shadow:inset 0 1px 0 rgba(255,255,255,.9)}
.flow-app input:not([type=checkbox]):not([type=radio]):not([type=hidden]),.flow-app textarea,.flow-app select{border-color:rgba(148,163,184,.38);background:linear-gradient(90deg,#fff,#f8fbff);box-shadow:inset 0 1px 2px rgba(15,23,42,.035);transition:border-color .18s ease,box-shadow .22s ease,background .18s ease}
.flow-app input:not([type=checkbox]):not([type=radio]):not([type=hidden]):focus-visible,.flow-app textarea:focus-visible,.flow-app select:focus-visible{border-color:rgba(67,56,202,.58);box-shadow:0 0 0 3px rgba(165,180,252,.28),inset 0 1px 2px rgba(15,23,42,.03);outline:0}
.flow-app textarea{font-weight:500;line-height:1.65}
.flow-app select{font-weight:600}
.flow-app button:not(.queue-chip):not(.settings-tab):not(.secondary-button):not(.danger):not(.source-choice):not(.skill-chip),.flow-app .button-link.setup-primary,.flow-app .setup-primary{position:relative;display:inline-flex;align-items:center;justify-content:center;gap:9px;overflow:hidden;isolation:isolate;min-height:40px;background:linear-gradient(90deg,#2563eb 0%,#4338ca 56%,#2f2a86 100%)!important;color:#fff!important;font-weight:700!important;text-shadow:none;border:0;box-shadow:inset 0 1px 0 rgba(255,255,255,.28),inset 0 -1px 0 rgba(15,23,42,.24),0 12px 22px rgba(67,56,202,.18);transition:transform .18s cubic-bezier(.2,.8,.2,1),box-shadow .22s cubic-bezier(.2,.8,.2,1),filter .18s ease}
.flow-app button:not(.queue-chip):not(.settings-tab):not(.secondary-button):not(.danger):not(.source-choice):not(.skill-chip):hover,.flow-app .button-link.setup-primary:hover,.flow-app .setup-primary:hover{transform:translateY(-1px);filter:saturate(1.03) brightness(1.02);box-shadow:inset 0 1px 0 rgba(255,255,255,.24),inset 0 -1px 0 rgba(15,23,42,.2),0 14px 24px rgba(67,56,202,.18)}
.flow-app button:not(.queue-chip):not(.settings-tab):not(.secondary-button):not(.danger):not(.source-choice):not(.skill-chip)::after,.flow-app .button-link.setup-primary::after,.flow-app .setup-primary::after{content:"";position:absolute;inset:0;z-index:-1;background:linear-gradient(110deg,rgba(255,255,255,.24),rgba(255,255,255,0) 38%),linear-gradient(90deg,rgba(255,255,255,0),rgba(255,255,255,.08) 64%,rgba(255,255,255,0));opacity:.76;pointer-events:none}
.flow-app .secondary-button,.flow-app .button-link.secondary{background:linear-gradient(90deg,#fff 0%,#f8fbff 55%,#eef2ff 100%)!important;color:var(--blue)!important;border:1px solid rgba(148,163,184,.38)!important;box-shadow:inset 0 1px 0 rgba(255,255,255,.9),0 8px 16px rgba(15,23,42,.04)!important;font-weight:700!important;text-shadow:none;transition:transform .18s cubic-bezier(.2,.8,.2,1),border-color .18s ease,box-shadow .22s ease}
.flow-app .secondary-button:hover,.flow-app .button-link.secondary:hover{transform:translateY(-1px);border-color:rgba(67,56,202,.45)!important;box-shadow:inset 0 1px 0 rgba(255,255,255,.9),0 12px 22px rgba(67,56,202,.08)!important;filter:none}
.flow-app button.danger,.flow-app button:not(.queue-chip):not(.settings-tab).danger{background:linear-gradient(90deg,#fff 0%,#fff7f7 58%,#fee2e2 100%)!important;border:1px solid rgba(185,28,28,.28)!important;color:var(--red)!important;box-shadow:inset 0 1px 0 rgba(255,255,255,.9),0 8px 16px rgba(185,28,28,.06)!important;font-weight:700;text-shadow:none}
.flow-app .panel,.flow-app .composer,.runtime-settings-card,.runtime-test-card,.runtime-tool-panel,.runtime-test-box{border-color:rgba(148,163,184,.34);background:linear-gradient(90deg,#fff 0%,#fbfdff 100%);box-shadow:inset 0 1px 0 rgba(255,255,255,.94),0 12px 28px rgba(15,23,42,.045)}
.flow-app .sidebar{background:linear-gradient(90deg,#fff,#f8fbff);backdrop-filter:blur(12px)}
.flow-app .brand{display:grid;grid-template-columns:34px minmax(0,1fr);grid-template-rows:auto auto;column-gap:10px;align-items:center;margin:0 0 38px}
.flow-app .brand::before{display:none}
.flow-app .brand-logo{display:block;grid-row:1/3;width:34px;height:34px;border-radius:50%;object-fit:contain}
.flow-app .brand-text{position:static;width:auto;height:auto;overflow:visible;clip:auto;white-space:normal;font-size:24px;line-height:1.05;font-weight:800;color:var(--text)}
.flow-app .brand-text::after{content:"hachiware-labs";display:block;margin-top:4px;color:var(--muted);font-size:11px;line-height:1.2;font-weight:600}
.flow-app nav a{min-height:36px;display:flex;align-items:center;gap:10px;margin-bottom:7px}
.nav-icon{position:relative;display:inline-flex;align-items:center;justify-content:center;width:18px;height:18px;flex:0 0 18px;color:currentColor}
.nav-icon::before,.nav-icon::after{content:"";position:absolute;box-sizing:border-box}
.nav-icon-work::before{left:2px;right:2px;top:5px;bottom:3px;border:1.8px solid currentColor;border-radius:2px}
.nav-icon-work::after{left:5px;right:5px;top:3px;height:5px;border:1.8px solid currentColor;border-bottom:0;border-radius:2px 2px 0 0}
.nav-icon-project::before{left:2px;right:2px;top:6px;bottom:3px;border:1.8px solid currentColor;border-radius:2px}
.nav-icon-project::after{left:3px;top:4px;width:7px;height:4px;border:1.8px solid currentColor;border-bottom:0;border-radius:2px 2px 0 0}
.nav-icon-knowledge::before{left:4px;top:2px;width:10px;height:14px;border:1.8px solid currentColor;border-radius:2px}
.nav-icon-knowledge::after{left:6px;top:6px;width:6px;height:1.8px;background:currentColor;box-shadow:0 4px 0 currentColor}
.nav-icon-agent::before{left:6px;top:2px;width:6px;height:6px;border:1.8px solid currentColor;border-radius:50%}
.nav-icon-agent::after{left:3px;right:3px;bottom:3px;height:7px;border:1.8px solid currentColor;border-radius:3px}
.nav-icon-settings::before{left:4px;top:4px;width:10px;height:10px;border:1.8px solid currentColor;border-radius:50%}
.nav-icon-settings::after{left:8px;top:1px;width:1.8px;height:16px;background:currentColor;box-shadow:-5px 5px 0 -1px currentColor,5px 5px 0 -1px currentColor;transform:rotate(45deg)}
.sidebar-project{border-top:1px solid var(--line);margin-top:28px;padding-top:18px;display:grid;gap:5px}
.sidebar-project span{color:var(--muted);font-size:11px;font-weight:800}
.sidebar-project b{font-size:13px}
.sidebar-project small{color:var(--muted);line-height:1.4;overflow-wrap:anywhere}
.flow-content{background:linear-gradient(90deg,#f8fafc 0%,#f8fbff 58%,#eef6ff 100%);min-height:100vh}
.flow-content>.topbar{position:relative;margin-bottom:16px}
.flow-content>.topbar::after{content:"";position:absolute;left:0;right:0;bottom:-8px;height:3px;border-radius:999px;background:linear-gradient(90deg,#99f6e4,#2563eb 48%,#312e81)}
.home-composer-panel,.work-history-panel{box-shadow:0 18px 36px rgba(15,23,42,.08)}
.home-composer-panel{margin-top:12px;border-color:#d7dee8;padding:24px 28px}
.home-work-form{grid-template-columns:minmax(180px,280px) minmax(320px,1fr) 164px;align-items:end;gap:24px}
.home-work-form textarea{min-height:76px;resize:vertical}
.home-work-form #home-form-status:empty{display:none}
.home-work-actions{display:grid;gap:8px}
.home-work-actions button:not(.queue-chip):not(.settings-tab){background:linear-gradient(90deg,#2563eb 0%,#4338ca 58%,#312e81 100%);min-height:40px}
.home-work-actions button[type="submit"]{display:inline-flex;align-items:center;justify-content:center;gap:8px}
.home-work-actions button[type="submit"]::before{content:"+";display:inline-flex;align-items:center;justify-content:center;width:18px;height:18px;border-radius:999px;border:1.8px solid currentColor;font-size:16px;line-height:1;font-weight:700}
.home-work-actions button.secondary-button:not(.queue-chip):not(.settings-tab){min-height:34px;background:linear-gradient(90deg,#ffffff 0%,#eef2ff 56%,#e0e7ff 100%);color:var(--blue);border:1px solid #cbd5e1;box-shadow:none}
.work-history-panel{border-color:#d7dee8}
.history-counts{display:flex;gap:8px;flex-wrap:wrap;justify-content:flex-end}
.work-filter-panel{display:grid;grid-template-columns:minmax(180px,240px) minmax(240px,320px) minmax(220px,1fr);gap:16px;margin:14px 0 10px;align-items:center}
.work-filter-panel label{margin:0}
.filter-keyword input{min-height:40px}
.work-history-table th{padding-left:20px}
.work-history-table td{padding:14px 20px}
.work-history-row{position:relative;border-radius:8px}
.work-history-row td{background:#fff;border-top:1px solid #e2e8f0;border-bottom:1px solid #e2e8f0}
.work-history-row td:first-child{border-left:1px solid #e2e8f0;border-radius:8px 0 0 8px}
.work-history-row td:last-child{border-right:1px solid #e2e8f0;border-radius:0 8px 8px 0}
.work-history-row.state-needs-input td,.work-history-row.state-needs-approval td{background:linear-gradient(90deg,#fffdf5,#fff8d8)}
.work-history-row.state-running td,.work-history-row.state-in-review td{background:linear-gradient(90deg,#f8fbff,#eaf2ff)}
.work-history-row.state-failed td,.work-history-row.state-changes-requested td{background:linear-gradient(90deg,#fff7f7,#ffecec)}
.work-history-row.state-done td{background:linear-gradient(90deg,#f7fffb,#dcfce7)}
.work-history-row .badge{gap:6px;border:1px solid currentColor}
.work-history-row .badge::before{content:"";display:inline-flex;width:12px;height:12px;border:1.8px solid currentColor;border-radius:999px;box-sizing:border-box}
.work-history-row .badge.amber::after{content:"";position:relative;display:inline-flex;width:3px;height:3px;margin-left:-14px;margin-right:5px;border-radius:50%;background:currentColor;box-shadow:0 -4px 0 currentColor}
.work-history-row .badge.red::before{border-radius:2px;transform:rotate(45deg)}
.work-history-row .badge.green::after{content:"";position:relative;display:inline-flex;width:6px;height:3px;margin-left:-14px;margin-right:3px;border-left:1.8px solid currentColor;border-bottom:1.8px solid currentColor;transform:rotate(-45deg)}
.work-history-row .badge.blue::after{content:"";position:relative;display:inline-flex;width:4px;height:4px;margin-left:-14px;margin-right:5px;border-radius:50%;background:currentColor}
.work-title-line{display:flex;align-items:center;gap:10px;font-weight:900}
.work-state-dot{display:inline-flex;width:18px;height:18px;border-radius:999px;border:2px solid currentColor;color:#2563eb;flex:0 0 auto}
.work-state-dot.amber{color:#b45309}
.work-state-dot.red{color:#b91c1c}
.work-state-dot.green{color:#047857}
.work-state-dot.blue{color:#2563eb}
.work-state-dot.gray{color:#64748b}
.work-answer-inline{margin-top:6px;max-width:640px}
.work-answer-inline .answer-preview{min-width:0}
.work-answer-inline .answer-body{font-size:12px;margin:0;color:var(--muted)}
.work-history-row .row-actions{justify-content:flex-end}
.setup-page{background:linear-gradient(90deg,#f7fbff,#edf4ff 52%,#f9fbff)}
.setup-entry-content{min-height:100vh;background:linear-gradient(90deg,rgba(37,99,235,.05),rgba(20,184,166,.04))}
.setup-entry-frame{position:relative;margin-top:74px;min-height:334px;border:1px solid #d7dee8;border-radius:8px;background:linear-gradient(90deg,#fff,#f8fbff);box-shadow:0 18px 36px rgba(15,23,42,.08);overflow:hidden}
.setup-entry-frame::before{content:"";position:absolute;left:0;right:0;top:0;height:3px;background:linear-gradient(90deg,#99f6e4,#2563eb 48%,#312e81)}
.setup-entry-dialog{position:absolute;left:120px;top:88px;width:306px;border:1px solid #d7dee8;border-radius:8px;background:#fff;padding:24px 22px;box-shadow:0 18px 42px rgba(15,23,42,.12)}
.setup-entry-dialog h2{font-size:17px;margin:0 0 12px}
.setup-entry-dialog p{margin:0 0 22px;line-height:1.55;color:var(--muted)}
.setup-primary,.button-link.setup-primary{justify-content:center;min-height:38px;background:linear-gradient(90deg,#2563eb,#4338ca 58%,#312e81);color:#fff;box-shadow:0 10px 22px rgba(67,56,202,.22);border:0}
.runtime-setup-content{background:linear-gradient(90deg,rgba(37,99,235,.05),rgba(20,184,166,.04));min-height:100vh}
.runtime-page-header{display:flex;align-items:center;justify-content:space-between;min-height:72px;border:1px solid #d7dee8;border-radius:8px;background:linear-gradient(90deg,#fff,#f8fbff);padding:14px 24px;margin-bottom:20px;box-shadow:0 12px 24px rgba(15,23,42,.06)}
.runtime-page-header h1{font-size:24px;margin:0 0 4px}
.runtime-stepper{display:grid;grid-template-columns:150px 104px 140px 104px 140px;align-items:center;gap:12px;min-height:72px;border:1px solid #d7dee8;border-radius:8px;background:#fff;padding:14px 22px;margin-bottom:24px}
.runtime-step{display:grid;grid-template-columns:24px minmax(0,1fr);grid-template-areas:"num title" "num sub";column-gap:10px;align-items:center}
.runtime-step span{grid-area:num;display:inline-flex;align-items:center;justify-content:center;width:24px;height:24px;border-radius:50%;border:1px solid #c7d2fe;background:#fff;color:var(--blue);font-weight:800;font-size:12px}
.runtime-step.done span,.runtime-step.current span{background:linear-gradient(90deg,#2563eb,#4338ca);color:#fff;border-color:transparent}
.runtime-step b{grid-area:title;font-size:12px}
.runtime-step small{grid-area:sub;color:var(--muted);font-size:12px}
.runtime-step-line{height:1px;background:#d7dee8}
.runtime-step-line.active{background:linear-gradient(90deg,#99f6e4,#2563eb 48%,#312e81);height:2px}
.runtime-main-panel{position:relative;display:grid;grid-template-columns:minmax(520px,686px) minmax(320px,406px);gap:32px;min-height:644px;border:1px solid #d7dee8;border-radius:8px;background:linear-gradient(90deg,#fff,#f8fbff);padding:84px 24px 88px;margin:0;box-shadow:0 18px 36px rgba(15,23,42,.08);overflow:hidden}
.runtime-main-panel::before{content:"";position:absolute;left:0;right:0;top:0;height:3px;background:linear-gradient(90deg,#99f6e4,#2563eb 48%,#312e81)}
.runtime-settings-card,.runtime-test-card{background:#fff;border:1px solid #d7dee8;border-radius:8px;padding:20px 24px}
.runtime-settings-card h2,.runtime-test-card h2{font-size:17px;margin:0 0 30px}
.runtime-settings-card label{margin-bottom:22px}
.runtime-tool-panel{border:1px solid var(--line);border-radius:8px;background:#fbfdff;padding:18px 20px;margin-top:12px}
.runtime-tool-panel h3{font-size:16px;margin:0 0 18px}
.runtime-model-grid{grid-template-columns:260px 196px;gap:22px}
.runtime-test-box{border:1px solid var(--line);border-radius:8px;background:#fbfdff;padding:20px 22px;display:grid;gap:8px}
.runtime-test-box>span{color:var(--muted);font-size:12px}
.runtime-test-result{display:grid;grid-template-columns:22px minmax(0,1fr);grid-template-areas:"check title" "check body";border-top:1px solid var(--line);margin-top:14px;padding-top:20px}
.runtime-test-result span{grid-area:check;display:inline-flex;align-items:center;justify-content:center;width:16px;height:16px;border-radius:50%;background:#047857;color:#fff;font-size:11px}
.runtime-test-result b{grid-area:title;font-size:12px}
.runtime-test-result small{grid-area:body;color:var(--muted);font-size:12px}
.runtime-test-actions{display:flex;gap:16px;margin-top:38px;align-items:center}
.runtime-ok{display:inline-flex;align-items:center;justify-content:center;min-width:116px;min-height:32px;border-radius:16px;background:#ecfdf5;color:#047857;border:1px solid #bbf7d0;font-size:12px;font-weight:800}
.runtime-footer-actions{position:absolute;left:24px;right:24px;bottom:22px;display:flex;justify-content:space-between}
.runtime-footer-actions .button-link,.runtime-footer-actions button{min-width:116px;min-height:38px}
.runtime-main-panel>#setup-status{position:absolute;left:24px;bottom:64px}
.design-catalog{display:grid;gap:18px;font-family:"Segoe UI","Yu Gothic UI",Meiryo,Arial,sans-serif;font-synthesis-weight:none;text-rendering:geometricPrecision}
.design-catalog>.topbar{margin-bottom:0}
.catalog-hero{display:grid;grid-template-columns:minmax(0,1fr) 280px;gap:24px;align-items:center;margin-top:12px;background:linear-gradient(90deg,#fff 0%,#f8fbff 58%,#eef6ff 100%);border-color:rgba(148,163,184,.36);box-shadow:inset 0 1px 0 rgba(255,255,255,.9),0 18px 36px rgba(15,23,42,.06)}
.catalog-hero h2{font-size:22px;line-height:1.35;margin:4px 0 10px;max-width:720px}
.catalog-hero p{margin:0;color:var(--muted);line-height:1.65;max-width:760px}
.catalog-kicker,.catalog-section-head span{display:block;color:var(--blue);font-size:11px;font-weight:700;line-height:1.4}
.catalog-flow-card{position:relative;display:grid;gap:8px;overflow:hidden;border:1px solid rgba(165,180,252,.45);border-radius:8px;background:linear-gradient(90deg,#f8ffff 0%,#f8fbff 52%,#eef2ff 100%);padding:18px;box-shadow:inset 0 1px 0 rgba(255,255,255,.9)}
.catalog-flow-card::after{content:"";position:absolute;left:0;right:0;bottom:0;height:2px;background:linear-gradient(90deg,#99f6e4,#2563eb 52%,#312e81);opacity:.72}
.catalog-flow-card span{color:var(--muted);font-size:11px;font-weight:800}
.catalog-flow-card b{font-size:13px;line-height:1.4}
.catalog-grid{display:grid;grid-template-columns:1fr 1fr;gap:18px}
.catalog-panel{position:relative;overflow:hidden;margin:0;background:linear-gradient(90deg,#fff 0%,#fbfdff 100%);border-color:rgba(148,163,184,.34);box-shadow:inset 0 1px 0 rgba(255,255,255,.94),0 12px 28px rgba(15,23,42,.045)}
.catalog-panel::before{content:"";position:absolute;left:12px;right:12px;top:0;height:1px;background:linear-gradient(90deg,rgba(6,182,212,.38),rgba(37,99,235,.34),rgba(49,46,129,.18));opacity:.7}
.catalog-section-head{display:grid;gap:2px;margin-bottom:14px}
.catalog-section-head h2{margin:0}
.color-token-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:10px}
.color-token{display:grid;align-content:end;gap:4px;min-height:84px;border:1px solid rgba(148,163,184,.3);border-radius:8px;padding:10px;background:#fff;color:var(--text);box-shadow:inset 0 1px 0 rgba(255,255,255,.86)}
.color-token span{font-size:11px;font-weight:800;color:var(--muted)}
.color-token b{font-size:12px}
.bg-token{background:#f8fafc}
.surface-token{background:#fff}
.primary-token{background:linear-gradient(90deg,#3b82f6 0%,#4338ca 54%,#312e81 100%);color:#fff}
.primary-token span,.primary-token b,.flow-token span,.flow-token b,.success-token span,.success-token b,.warning-token span,.warning-token b,.danger-token span,.danger-token b{color:#fff}
.flow-token{background:linear-gradient(90deg,#67e8f9 0%,#2563eb 54%,#312e81 100%)}
.success-token{background:#047857}
.warning-token{background:linear-gradient(90deg,#f59e0b,#b45309)}
.danger-token{background:linear-gradient(90deg,#ef4444,#b91c1c)}
.border-token{background:#fff;border-color:#94a3b8}
.type-specimen{display:grid;gap:6px;border:1px solid var(--line);border-radius:8px;background:#fff;padding:16px}
.type-specimen h1{margin:0;font-size:24px;line-height:1.25}
.type-specimen h2{margin:0;font-size:17px;line-height:1.35}
.type-specimen p{margin:0;font-size:14px;line-height:1.55}
.type-specimen small{color:var(--muted);font-size:12px;font-weight:600}
.catalog-action-grid{display:grid;grid-template-columns:minmax(240px,300px) minmax(420px,1fr);grid-template-areas:"actions composer" "project composer";gap:16px;align-items:start}
.catalog-action-stack{display:grid;grid-template-columns:1fr 1fr;gap:10px;align-content:start;border:1px solid rgba(148,163,184,.3);border-radius:8px;background:linear-gradient(90deg,#fff,#f8fbff);padding:12px;box-shadow:inset 0 1px 0 rgba(255,255,255,.9)}
.catalog-action-stack{grid-area:actions}
.catalog-action-stack .catalog-primary-button{grid-column:1/-1}
.catalog-detail-button{justify-content:center;min-height:36px;border-color:rgba(148,163,184,.34)!important;background:linear-gradient(90deg,#fff,#f8fbff)!important;box-shadow:inset 0 1px 0 rgba(255,255,255,.9);transition:transform .18s cubic-bezier(.2,.8,.2,1),border-color .18s ease,box-shadow .22s ease}
.catalog-field{gap:8px;align-content:start;min-width:0;border:1px solid rgba(148,163,184,.3);border-radius:8px;background:linear-gradient(90deg,#fff,#fbfdff);padding:12px;box-shadow:inset 0 1px 0 rgba(255,255,255,.9);transition:border-color .18s ease,box-shadow .22s ease,transform .18s cubic-bezier(.2,.8,.2,1)}
.catalog-field:focus-within{border-color:rgba(148,163,184,.3);box-shadow:inset 0 1px 0 rgba(255,255,255,.9)}
.catalog-project-field{grid-area:project}
.catalog-wide-input{grid-area:composer}
.catalog-label-row{display:flex;align-items:center;justify-content:space-between;gap:10px;color:var(--text);font-size:12px;font-weight:700}
.catalog-label-row small{color:var(--muted);font-size:11px;font-weight:800}
.catalog-select-shell{position:relative;display:block;align-self:start}
.catalog-select-shell::after{content:"";position:absolute;right:13px;top:50%;width:7px;height:7px;border-right:1.8px solid #475569;border-bottom:1.8px solid #475569;transform:translateY(-62%) rotate(45deg);pointer-events:none}
.catalog-select-shell select{appearance:none;min-height:42px;border-color:rgba(148,163,184,.38);background:linear-gradient(90deg,#fff,#f8fbff);padding:10px 34px 10px 12px;font-weight:600;box-shadow:inset 0 1px 2px rgba(15,23,42,.035);transition:border-color .18s ease,box-shadow .22s ease,background .18s ease}
.catalog-select-shell select:hover{border-color:rgba(67,56,202,.42);background:#fff}
.catalog-primary-button{position:relative;display:inline-flex;align-items:center;justify-content:center;gap:9px;overflow:hidden;isolation:isolate;min-height:44px;background:linear-gradient(90deg,#2563eb 0%,#4338ca 56%,#2f2a86 100%)!important;color:#fff;font-weight:700!important;text-shadow:none;box-shadow:inset 0 1px 0 rgba(255,255,255,.28),inset 0 -1px 0 rgba(15,23,42,.24),0 12px 22px rgba(67,56,202,.18);transition:transform .18s cubic-bezier(.2,.8,.2,1),box-shadow .22s cubic-bezier(.2,.8,.2,1),filter .18s ease}
.catalog-primary-button::before{content:"+";position:relative;z-index:1;display:inline-flex;align-items:center;justify-content:center;width:19px;height:19px;border-radius:999px;border:1.7px solid rgba(255,255,255,.9);background:rgba(255,255,255,.1);font-size:16px;line-height:1;font-weight:700;box-shadow:inset 0 1px 0 rgba(255,255,255,.24)}
.catalog-primary-button::after{content:"";position:absolute;inset:0;z-index:-1;background:linear-gradient(110deg,rgba(255,255,255,.26),rgba(255,255,255,0) 38%),linear-gradient(90deg,rgba(255,255,255,0),rgba(255,255,255,.08) 64%,rgba(255,255,255,0));opacity:.78;pointer-events:none}
.catalog-secondary-button{min-height:36px!important;background:linear-gradient(90deg,#fff 0%,#f8fbff 55%,#eef2ff 100%)!important;color:var(--blue)!important;border:1px solid rgba(148,163,184,.38)!important;box-shadow:inset 0 1px 0 rgba(255,255,255,.9),0 8px 16px rgba(15,23,42,.04)!important;transition:transform .18s cubic-bezier(.2,.8,.2,1),border-color .18s ease,box-shadow .22s ease}
.catalog-secondary-button:hover,.catalog-detail-button:hover{transform:translateY(-1px);border-color:rgba(67,56,202,.45)!important;box-shadow:inset 0 1px 0 rgba(255,255,255,.9),0 12px 22px rgba(67,56,202,.08)!important}
.catalog-composer-shell{display:grid;border:1px solid rgba(148,163,184,.38);border-radius:8px;background:linear-gradient(90deg,#fff,#f8fbff);box-shadow:inset 0 1px 2px rgba(15,23,42,.035);overflow:hidden;transition:border-color .18s ease,box-shadow .22s ease}
.catalog-composer-shell:focus-within{border-color:rgba(148,163,184,.38);box-shadow:inset 0 1px 2px rgba(15,23,42,.035)}
.catalog-composer-shell textarea{min-height:136px;border:0;background:transparent;resize:vertical;padding:13px 14px;font-weight:500;line-height:1.65;box-shadow:none;outline:0}
.catalog-composer-footer{display:flex;align-items:center;justify-content:space-between;gap:12px;border-top:1px solid rgba(226,232,240,.92);background:linear-gradient(90deg,#fbfdff,#f8fbff);padding:8px 12px;color:var(--muted);font-size:11px;font-weight:800}
.catalog-composer-footer b{color:var(--blue);font-size:11px}
.micro-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px;margin-bottom:16px}
.micro-card{position:relative;display:grid;gap:10px;align-content:start;min-width:0;overflow:hidden;border:1px solid rgba(148,163,184,.28);border-radius:8px;background:linear-gradient(90deg,#fff,#fbfdff);padding:12px;box-shadow:inset 0 1px 0 rgba(255,255,255,.9);transition:transform .18s cubic-bezier(.2,.8,.2,1),border-color .18s ease,box-shadow .22s ease}
.micro-card:hover{transform:translateY(-1px);border-color:rgba(165,180,252,.58);box-shadow:inset 0 1px 0 rgba(255,255,255,.9),0 12px 22px rgba(15,23,42,.06)}
.micro-card span{color:var(--muted);font-size:11px;font-weight:800}
.micro-card small{color:var(--muted);font-size:12px;line-height:1.45}
.micro-card .catalog-primary-button,.micro-card .secondary-button{width:100%;min-height:38px}
.catalog-primary-button:hover,.micro-hover{transform:translateY(-1px);filter:saturate(1.03) brightness(1.02);box-shadow:inset 0 1px 0 rgba(255,255,255,.24),0 14px 24px rgba(67,56,202,.18)}
.micro-focus{outline:2px solid rgba(165,180,252,.82)!important;outline-offset:3px}
.micro-selected{animation:nagare-selected-sheen 1.8s cubic-bezier(.2,.8,.2,1) infinite}
.micro-active{transform:translateY(1px) scale(.995);filter:brightness(.96);box-shadow:inset 0 2px 8px rgba(15,23,42,.18)}
.micro-loading{position:relative;pointer-events:none;color:rgba(255,255,255,.86)}
.micro-loading::before{content:"";width:14px;height:14px;border:2px solid rgba(255,255,255,.52);border-top-color:#fff;border-radius:999px;animation:nagare-spin .9s linear infinite}
.micro-loading::after{display:none}
.micro-success{background:linear-gradient(90deg,#10b981,#047857)!important;box-shadow:inset 0 1px 0 rgba(255,255,255,.18),0 10px 18px rgba(4,120,87,.14)}
.micro-success::before{content:"";width:14px;height:8px;border-left:2px solid currentColor;border-bottom:2px solid currentColor;border-radius:0;transform:rotate(-45deg)}
.micro-error{background:linear-gradient(90deg,#dc2626,#991b1b)!important;box-shadow:inset 0 1px 0 rgba(255,255,255,.16),0 10px 18px rgba(185,28,28,.12)}
.micro-error::before{content:"!";display:inline-flex;align-items:center;justify-content:center;width:18px;height:18px;border-radius:999px;border:1.8px solid currentColor;font-size:12px;font-weight:800}
.micro-disabled{opacity:.48;filter:saturate(.6);cursor:not-allowed!important}
.micro-wide{grid-column:span 2}
.micro-undo-bar{display:flex;align-items:center;justify-content:space-between;gap:12px;border:1px solid rgba(165,180,252,.45);border-radius:8px;background:linear-gradient(90deg,#fff,#f8fbff);padding:8px 10px;box-shadow:inset 0 1px 0 rgba(255,255,255,.9)}
.micro-undo-bar b{font-size:13px}
.micro-undo-bar button{min-height:30px!important;width:auto!important}
.micro-field-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px;margin-top:8px}
.micro-field-grid label{border:1px solid rgba(148,163,184,.3);border-radius:8px;background:linear-gradient(90deg,#fff,#fbfdff);padding:11px;box-shadow:inset 0 1px 0 rgba(255,255,255,.9);transition:transform .18s cubic-bezier(.2,.8,.2,1),border-color .18s ease,box-shadow .22s ease}
.micro-field-grid label:hover{transform:translateY(-1px);border-color:rgba(165,180,252,.52);box-shadow:inset 0 1px 0 rgba(255,255,255,.9),0 10px 20px rgba(15,23,42,.055)}
.micro-field-grid input{min-height:38px;border-color:rgba(148,163,184,.36);background:linear-gradient(90deg,#fff,#f8fbff);box-shadow:inset 0 1px 2px rgba(15,23,42,.035);transition:border-color .18s ease,box-shadow .22s ease,background .18s ease}
.micro-field-focus input{border-color:rgba(67,56,202,.64);outline:0;box-shadow:0 0 0 3px rgba(165,180,252,.34),inset 0 1px 2px rgba(15,23,42,.03)}
.micro-field-valid input{border-color:rgba(4,120,87,.45);background:linear-gradient(90deg,#fff,#f0fdf4);box-shadow:0 0 0 3px rgba(16,185,129,.12),inset 0 1px 2px rgba(15,23,42,.025)}
.micro-field-error input{border-color:rgba(185,28,28,.42);background:linear-gradient(90deg,#fff,#fff7f7);box-shadow:0 0 0 3px rgba(239,68,68,.12),inset 0 1px 2px rgba(15,23,42,.025)}
.micro-field-error small{color:var(--red);font-size:11px;font-weight:800}
.micro-row-strip{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px;margin-top:14px}
.micro-row-state{position:relative;display:grid;grid-template-columns:22px minmax(0,1fr);grid-template-areas:"icon title" "icon body";gap:2px 8px;overflow:hidden;border:1px solid rgba(148,163,184,.3);border-radius:8px;background:linear-gradient(90deg,#fff,#fbfdff);padding:12px;box-shadow:inset 0 1px 0 rgba(255,255,255,.9);transition:transform .18s cubic-bezier(.2,.8,.2,1),border-color .18s ease,box-shadow .22s ease}
.micro-row-state .work-state-dot,.micro-progress-dot{grid-area:icon;margin-top:2px}
.micro-row-state b{grid-area:title;font-size:13px}
.micro-row-state small{grid-area:body;color:var(--muted);font-size:12px}
.micro-row-hover{border-color:rgba(165,180,252,.58);box-shadow:inset 0 1px 0 rgba(255,255,255,.9),0 10px 22px rgba(15,23,42,.07);transform:translateY(-1px)}
.micro-row-selected{border-color:rgba(165,180,252,.78);background:linear-gradient(90deg,#fff,#f8fbff 52%,#eef2ff);box-shadow:inset 3px 0 0 #4338ca,0 10px 20px rgba(67,56,202,.08)}
.micro-row-progress::after{content:"";position:absolute;left:0;right:0;bottom:0;height:2px;background:linear-gradient(90deg,#99f6e4,#2563eb 48%,#312e81);animation:nagare-flow 1.8s ease-in-out infinite}
.micro-progress-dot{display:inline-flex;width:18px;height:18px;border-radius:999px;border:2px solid #2563eb}
.micro-progress-dot::after{content:"";width:6px;height:6px;margin:auto;border-radius:999px;background:#2563eb;animation:nagare-pulse 1.2s ease-in-out infinite}
@keyframes nagare-spin{to{transform:rotate(360deg)}}
@keyframes nagare-flow{0%{transform:translateX(-35%)}50%{transform:translateX(0)}100%{transform:translateX(35%)}}
@keyframes nagare-pulse{0%,100%{opacity:.35;transform:scale(.72)}50%{opacity:1;transform:scale(1)}}
@keyframes nagare-selected-sheen{0%,100%{box-shadow:inset 0 1px 0 rgba(255,255,255,.24),0 10px 18px rgba(67,56,202,.14)}50%{box-shadow:inset 0 1px 0 rgba(255,255,255,.3),0 16px 28px rgba(67,56,202,.2)}}
.catalog-chip-row{display:flex;gap:10px;flex-wrap:wrap}
.catalog-chip-row .badge{gap:6px;border:1px solid currentColor}
.catalog-chip-row .badge::before{content:"";display:inline-flex;width:12px;height:12px;border:1.8px solid currentColor;border-radius:999px;box-sizing:border-box}
.catalog-work-table{border-collapse:separate;border-spacing:0 10px}
.catalog-stepper{grid-template-columns:150px minmax(54px,1fr) 140px minmax(54px,1fr) 140px;margin:0}
.catalog-toast-stack{display:grid;gap:12px}
.catalog-toast-stack .toast{position:static;width:auto;box-shadow:0 14px 28px rgba(15,23,42,.12)}
@media(max-width:1200px){
  .catalog-hero,.catalog-grid{grid-template-columns:1fr}
  .catalog-action-grid{grid-template-columns:1fr}
  .catalog-wide-input{grid-column:1/-1}
  .color-token-grid{grid-template-columns:repeat(2,minmax(0,1fr))}
  .micro-grid,.micro-field-grid,.micro-row-strip{grid-template-columns:1fr 1fr}
  .micro-wide{grid-column:1/-1}
  .catalog-stepper{grid-template-columns:1fr}
  .catalog-stepper .runtime-step-line{height:2px;width:100%}
}
@media(prefers-reduced-motion:reduce){
  .micro-loading::before,.micro-selected,.micro-row-progress::after,.micro-progress-dot::after{animation:none}
}
@media(max-width:760px){
  .sidebar{display:flex;border-right:0;border-bottom:1px solid var(--line);padding:14px 18px;align-items:center;justify-content:space-between;gap:12px}
  .brand{margin:0}
  .brand-logo{width:88px}
  nav{display:flex;gap:6px;flex-wrap:wrap;justify-content:flex-end}
  nav a{padding:8px 10px}
  .panel-head,.form-section-head{display:grid;grid-template-columns:1fr;align-items:start}
  .panel-head .button-link{justify-self:start}
  .panel-head .badge{justify-self:start}
  .queue-panel{overflow-x:visible}
  .queue-table{display:none}
  .queue-card-list{display:block}
  .queue-card-meta{grid-template-columns:1fr}
  .source-choice-grid{grid-template-columns:1fr}
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
  .home-work-form,.work-filter-panel{grid-template-columns:1fr}
  .history-counts{justify-content:flex-start}
  .setup-content::before{left:18px;right:18px;top:132px}
  .setup-stage,.runtime-stage{grid-template-columns:1fr;margin-top:28px}
  .setup-work-preview,.setup-runtime-flow{display:none}
  .setup-dialog{padding:22px}
}
"#
}

pub(crate) fn serve_item_detail_stylesheet() -> &'static str {
    r#"
.summary .panel-head{align-items:flex-start}
.summary .panel-head h2{margin-bottom:4px}
.summary .panel-head p{margin:0}
.request-brief{border:1px solid #d7dee8;border-radius:8px;background:linear-gradient(90deg,#fff,#f8fbff);padding:14px 16px;margin:14px 0}
.request-brief span{display:block;color:var(--muted);font-size:11px;font-weight:900;margin-bottom:6px}
.request-brief p{margin:0;white-space:pre-wrap;line-height:1.65;overflow-wrap:anywhere}
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
  setTimeout(()=>toast.remove(),kind==='error' ? 9000 : kind==='success' ? 7000 : 4500);
}
function readFlashNotification(){
  try{
    const raw=sessionStorage.getItem('nagare:flash');
    if(!raw){return;}
    sessionStorage.removeItem('nagare:flash');
    const data=JSON.parse(raw);
    notify(data.message,data.kind || 'info');
  }catch(_){}
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
const setupCodexForm=document.getElementById('setup-codex-form');
if(setupCodexForm){
  const setupStatus=document.getElementById('setup-status');
  setupCodexForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    if(setupStatus){setupStatus.textContent='Codex設定を保存しています…';}
    const response=await fetch('/api/setup/codex',{method:'POST',body:new URLSearchParams(new FormData(setupCodexForm))});
    if(!response.ok){await notifyResponseError(response,setupStatus);return;}
    if(setupStatus){setupStatus.textContent='セットアップが完了しました。';}
    sessionStorage.setItem('nagare:flash',JSON.stringify({kind:'success',message:'セットアップが完了しました\n最初の依頼を作成できます'}));
    window.location.href='/';
  });
}
readFlashNotification();
const form=document.getElementById('create-work-form');
const statusEl=document.getElementById('form-status');
async function submitWorkCreateForm(targetForm,targetStatus){
  if(targetStatus){targetStatus.textContent='Work Itemを追加しています…';}
  const response=await fetch('/api/items',{method:'POST',body:new URLSearchParams(new FormData(targetForm))});
  if(!response.ok){await notifyResponseError(response,targetStatus);return;}
  const item=await response.json();
  if(targetStatus){targetStatus.textContent='Work Itemを追加しました。バックグラウンド実行を開始しました。';}
  notify('Work Itemを追加しました。','success');
  window.location.href=item.id ? `/items/${encodeURIComponent(item.id)}` : '/';
}
if(form){
  const workDomainselect=document.getElementById('work-domain-group');
  const workDomainSelect=document.getElementById('work-domain');
  const workPolicySelect=form.querySelector('select[name="domain_agent_policy"]');
  const routingDomain=form.querySelector('[data-routing-domain]');
  const routingPolicy=form.querySelector('[data-routing-policy]');
  function selectedText(select){
    if(!select || select.disabled){return '';}
    return select.selectedOptions && select.selectedOptions[0] ? select.selectedOptions[0].textContent.trim() : '';
  }
  function syncWorkDomainOptions(){
    if(!workDomainselect || !workDomainSelect){return;}
    const group=workDomainselect.value;
    let selectedStillVisible=false;
    [...workDomainSelect.options].forEach((option)=>{
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
    workDomainSelect.disabled=!group;
    if(!selectedStillVisible){workDomainSelect.value='';}
  }
  function syncRoutingPreview(){
    syncWorkDomainOptions();
    if(routingDomain){
      const groupText=selectedText(workDomainselect) || 'プロジェクト既定';
      const domainText=selectedText(workDomainSelect) || (workDomainSelect && workDomainSelect.disabled ? '成果物種別はドメイン選択後に指定' : 'プロジェクト既定');
      routingDomain.textContent=`${groupText} / ${domainText}`;
    }
    if(routingPolicy){
      const policy=workPolicySelect ? workPolicySelect.value : 'auto_general_fallback';
      const messages={
        auto_general_fallback:'専門エージェントが見つかれば優先し、見つからない場合は汎用エージェントで進めます。',
        confirm_general_fallback:'専門エージェントが見つからない場合は、汎用エージェントへ進める前に確認します。',
        require_domain_agent:'指定ドメインに対応するエージェントが必要です。見つからない場合は確認が必要になります。'
      };
      routingPolicy.textContent=messages[policy] || '作成後にDispatcherが担当候補を確認します。';
    }
  }
  if(workDomainselect){workDomainselect.addEventListener('change',syncRoutingPreview);}
  if(workDomainSelect){workDomainSelect.addEventListener('change',syncRoutingPreview);}
  if(workPolicySelect){workPolicySelect.addEventListener('change',syncRoutingPreview);}
  syncRoutingPreview();
  form.addEventListener('submit',async(event)=>{
    event.preventDefault();
    await submitWorkCreateForm(form,statusEl);
  });
}
const homeWorkForm=document.getElementById('home-work-form');
if(homeWorkForm){
  const homeStatus=document.getElementById('home-form-status');
  homeWorkForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    await submitWorkCreateForm(homeWorkForm,homeStatus);
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
      document.querySelectorAll('[data-work-record][data-queue-state]').forEach((entry)=>{
        const states=(entry.dataset.queueState||'').split(/\s+/);
        entry.hidden=state!=='all' && !states.includes(state);
      });
    });
  });
}
const projectFilter=document.getElementById('work-project-filter');
const keywordFilter=document.getElementById('work-keyword-filter');
const statusFilters=[...document.querySelectorAll('[data-work-status-filter]')];
function applyWorkHistoryFilters(){
  const project=projectFilter ? projectFilter.value : 'all';
  const keyword=keywordFilter ? keywordFilter.value.trim().toLowerCase() : '';
  const activeStates=statusFilters.filter((input)=>input.checked).map((input)=>input.value);
  document.querySelectorAll('[data-work-record][data-queue-state]').forEach((entry)=>{
    const states=(entry.dataset.queueState||'normal').split(/\s+/);
    const stateMatch=activeStates.length===0 || activeStates.some((state)=>states.includes(state) || (state==='done' && states.includes('normal')));
    const projectMatch=project==='all' || entry.dataset.project===project;
    const text=(entry.dataset.search||'').toLowerCase();
    const keywordMatch=!keyword || text.includes(keyword);
    entry.hidden=!(stateMatch && projectMatch && keywordMatch);
  });
}
if(projectFilter){projectFilter.addEventListener('change',applyWorkHistoryFilters);}
if(keywordFilter){keywordFilter.addEventListener('input',applyWorkHistoryFilters);}
statusFilters.forEach((input)=>input.addEventListener('change',applyWorkHistoryFilters));
if(projectFilter || keywordFilter || statusFilters.length){applyWorkHistoryFilters();}
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
    const show=rowHasAny(row,'agentDomains',groups) && rowHasAny(row,'agentArtifactTypes',domains);
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
    document.querySelectorAll('[data-work-record]').forEach((entry)=>{
      if(entry.dataset.workRecord===workId){entry.remove();}
    });
  });
});
document.querySelectorAll('.delete-domain-group-form').forEach((deleteForm)=>{
  deleteForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    const groupId=deleteForm.dataset.domainGroupId;
    const name=deleteForm.dataset.domainGroupName || groupId;
    if(!confirm(`ドメイン「${name}」を削除しますか？`)){return;}
    const button=deleteForm.querySelector('button');
    button.disabled=true;
    button.textContent='削除中…';
    const response=await fetch(`/api/domains/${groupId}/delete`,{method:'POST'});
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
    const response=await fetch(`/api/artifact-types/${domainId}/delete`,{method:'POST'});
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
  const agentKindTitle=agentProfileForm.querySelector('[data-agent-kind-title]');
  const agentKindCopy=agentProfileForm.querySelector('[data-agent-kind-copy]');
  const modelHelp=agentProfileForm.querySelector('[data-model-help]');
  const providerInput=document.getElementById('openclaw-model-provider');
  const modelInput=agentProfileForm.querySelector('input[name="model_id"]');
  const baseUrlInput=agentProfileForm.querySelector('input[name="base_url"]');
  const apiKeyEnvInput=agentProfileForm.querySelector('input[name="api_key_env"]');
  const agentDomainselect=document.getElementById('agent-domain-group');
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
      if(modelHelp){modelHelp.textContent='Codex系のエージェントではOpenAIモデル名だけを指定します。';}
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
      if(modelHelp){modelHelp.textContent='OpenClawでOpenAI系Providerを使う設定です。Base URLは不要です。';}
    }else{
      modelInput.removeAttribute('list');
      modelInput.placeholder=providerInput.value==='ollama' ? 'llama3.2' : 'loaded-model-name';
      if(providerInput.value==='ollama' && !baseUrlInput.value){baseUrlInput.value='http://127.0.0.1:11434/v1';}
      if(providerInput.value==='lmstudio' && !baseUrlInput.value){baseUrlInput.value='http://127.0.0.1:1234/v1';}
      baseUrlInput.required=true;
      apiKeyEnvInput.value='';
      if(modelHelp){modelHelp.textContent='ローカルProviderを使うため、モデル名とBase URLを指定します。';}
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
      if(agentKindTitle){agentKindTitle.textContent='Codex App Server';}
      if(agentKindCopy){agentKindCopy.textContent='Codex App Server経由で実行します。モデル名はCodex側のOpenAIモデルとして扱います。';}
    }else if(kindSelect.value==='openclaw'){
      runtimeInput.value='openclaw-local';
      adapterInput.value='process.openclaw-agent';
      externalProviderInput.value='openclaw';
      if(agentKindTitle){agentKindTitle.textContent='OpenClaw';}
      if(agentKindCopy){agentKindCopy.textContent='Providerに応じてOpenAI、Ollama、LM Studioのモデル設定を切り替えます。';}
    }else{
      runtimeInput.value='codex-local';
      adapterInput.value='process.codex-cli';
      externalProviderInput.value='codex-cli';
      if(agentKindTitle){agentKindTitle.textContent='Codex CLI';}
      if(agentKindCopy){agentKindCopy.textContent='Codex CLIをローカルプロセスとして実行します。OpenAIモデル名を指定します。';}
    }
    syncExternalAgentId();
    syncModelFields();
  }
  function syncAgentDomainOptions(){
    if(!agentDomainselect || !agentDomainSelect){return;}
    const group=agentDomainselect.value;
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
  if(agentDomainselect){agentDomainselect.addEventListener('change',syncAgentDomainOptions);}
  syncAgentKind();
  syncAgentDomainOptions();
  function escapeHtml(value){
    return value.replace(/[&<>"']/g,(char)=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
  }
  document.querySelectorAll('[data-skill-picker]').forEach((picker)=>{
    const searchInput=picker.querySelector('[data-skill-search]');
    const selectedEl=picker.querySelector('[data-skill-selected]');
    const options=[...picker.querySelectorAll('[data-skill-option]')];
    const agentId=picker.dataset.agentId || '';
    const uninstallLabel=picker.dataset.uninstallLabel || 'Uninstall';
    const uninstallConfirm=picker.dataset.uninstallConfirm || 'Remove this skill package?';
    function renderSelectedSkills(){
      const selected=options
        .map((option)=>({option,input:option.querySelector('input[type="checkbox"]'),name:option.querySelector('.skill-option-title span').textContent.trim()}))
        .filter((item)=>item.input && item.input.checked);
      selectedEl.innerHTML=selected.length
        ? selected.map((item)=>{
          const removeButton=`<button class="skill-chip" type="button" data-remove-skill="${escapeHtml(item.input.value)}" aria-label="${escapeHtml(item.name)} を外す"><span aria-hidden="true">x</span><b translate="no">${escapeHtml(item.name)}</b></button>`;
          const uninstallButton=agentId ? `<button class="skill-chip skill-chip-uninstall" type="button" data-uninstall-skill="${escapeHtml(item.input.value)}">${escapeHtml(uninstallLabel)}</button>` : '';
          return `<span class="skill-chip-group">${removeButton}${uninstallButton}</span>`;
        }).join('')
        : `<span class="muted">${escapeHtml(picker.dataset.emptyLabel || 'No skills selected')}</span>`;
    }
    function filterSkills(){
      const query=(searchInput.value || '').trim().toLowerCase();
      options.forEach((option)=>{
        option.hidden=query && !(option.dataset.skillSearchText || '').includes(query);
      });
    }
    options.forEach((option)=>option.querySelector('input[type="checkbox"]').addEventListener('change',renderSelectedSkills));
    selectedEl.addEventListener('click',(event)=>{
      const uninstallButton=event.target.closest('[data-uninstall-skill]');
      if(uninstallButton){
        const skillId=uninstallButton.dataset.uninstallSkill || '';
        if(!agentId || !skillId){return;}
        if(!confirm(uninstallConfirm)){return;}
        uninstallButton.disabled=true;
        if(agentProfileStatus){agentProfileStatus.textContent='スキル本体を削除しています…';}
        fetch(`/api/agents/${encodeURIComponent(agentId)}/skills/${encodeURIComponent(skillId)}/uninstall`,{method:'POST'})
          .then(async(response)=>{
            if(!response.ok){uninstallButton.disabled=false;await notifyResponseError(response,agentProfileStatus);return;}
            const data=await response.json();
            const warnings=Array.isArray(data.warnings) ? data.warnings.filter(Boolean) : [];
            notify(data.package_removed ? 'スキルをアンインストールしました。' : 'スキルをエージェントから外しました。', warnings.length ? 'info' : 'success');
            warnings.forEach((warning)=>notify(warning,'info'));
            window.location.reload();
          })
          .catch((error)=>{
            uninstallButton.disabled=false;
            if(agentProfileStatus){agentProfileStatus.textContent='';}
            notify(String(error),'error');
          });
        return;
      }
      const button=event.target.closest('[data-remove-skill]');
      if(!button){return;}
      const checkbox=options
        .map((option)=>option.querySelector('input[type="checkbox"]'))
        .find((input)=>input && input.value===button.dataset.removeSkill);
      if(!checkbox){return;}
      checkbox.checked=false;
      checkbox.dispatchEvent(new Event('change',{bubbles:true}));
      if(agentProfileStatus){agentProfileStatus.textContent='スキルを外しました。保存すると反映されます。';}
      checkbox.closest('[data-skill-option]')?.focus?.();
    });
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
const ArtifactTypeForm=document.getElementById('domain-profile-form');
if(ArtifactTypeForm){
  const ArtifactTypestatus=document.getElementById('domain-profile-status');
  const generateDomainRubricButton=ArtifactTypeForm.querySelector('[data-generate-domain-rubric]');
  const domainRubricInput=ArtifactTypeForm.querySelector('textarea[name="rubric"]');
  function domainField(name){
    const field=ArtifactTypeForm.querySelector(`[name="${name}"]`);
    return field ? field.value.trim() : '';
  }
  function compactLines(value,limit=3){
    return value.split(/\r?\n|,/).map((line)=>line.trim()).filter(Boolean).slice(0,limit);
  }
  function selectedSampleNames(){
    return [...ArtifactTypeForm.querySelectorAll('input[type="file"]')]
      .flatMap((input)=>[...input.files].map((file)=>file.name))
      .filter(Boolean);
  }
  function joinedOrDefault(values, fallback){
    return values.length ? values.join('、') : fallback;
  }
  function buildDomainRubricDraft(){
    const domainName=domainField('display_name') || domainField('id') || 'このドメイン';
    const description=domainField('description') || `${domainName}で扱う成果物`;
    const artifactTypes=joinedOrDefault(compactLines(domainField('artifact_types'),4),'成果物タイプ');
    const samples=joinedOrDefault(selectedSampleNames().slice(0,4),'登録サンプル');
    const sampleNote=domainField('sample_note');
    const general=joinedOrDefault(compactLines(domainField('general_points'),4),'一般的な品質、正確性、使いやすさ');
    const project=joinedOrDefault(compactLines(domainField('project_points'),4),'プロジェクト固有の制約と優先順位');
    const ng=joinedOrDefault(compactLines(domainField('ng_examples'),4),'目的不一致、根拠不足、検証不能な成果物');
    const sampleBasis=sampleNote ? `${samples}。メモ: ${sampleNote}` : samples;
    return [
      `20点: 目的適合 - ${description}に対して、利用者の目的、判断場面、期待成果が明確に満たされている`,
      `20点: 成果物品質 - ${artifactTypes}として、正確性、完成度、読みやすさ、扱いやすさが十分である`,
      `20点: サンプル適合 - ${sampleBasis}から読み取れる良い特徴を反映し、悪い特徴を避けている`,
      `15点: 一般評価観点 - ${general}を満たし、同種成果物として標準的に期待される品質に届いている`,
      `15点: プロジェクト固有観点 - ${project}を優先し、Nagare上の作業文脈と制約に合っている`,
      `10点: NG回避と検証可能性 - ${ng}を避け、レビュー時に根拠、差分、確認方法を追える`
    ].join('\n');
  }
  if(generateDomainRubricButton && domainRubricInput){
    generateDomainRubricButton.addEventListener('click',()=>{
      if(domainRubricInput.value.trim() && !confirm('現在のRubricを生成結果で置き換えますか？')){return;}
      domainRubricInput.value=buildDomainRubricDraft();
      domainRubricInput.focus();
      ArtifactTypestatus.textContent='100点満点のRubric案を生成しました。';
    });
  }
  ArtifactTypeForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    ArtifactTypestatus.textContent='成果物種別を保存しています…';
    const response=await fetch(ArtifactTypeForm.dataset.action,{method:'POST',body:new FormData(ArtifactTypeForm)});
    if(!response.ok){await notifyResponseError(response,ArtifactTypestatus);return;}
    ArtifactTypestatus.textContent='成果物種別を保存しました。';
    window.location.href=ArtifactTypeForm.dataset.redirect || '/settings';
  });
}
const DomainForm=document.getElementById('domain-group-form');
if(DomainForm){
  const Domainstatus=document.getElementById('domain-group-status');
  DomainForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    Domainstatus.textContent='ドメインを保存しています…';
    const response=await fetch(DomainForm.dataset.action,{method:'POST',body:new URLSearchParams(new FormData(DomainForm))});
    if(!response.ok){await notifyResponseError(response,Domainstatus);return;}
    Domainstatus.textContent='ドメインを保存しました。';
    window.location.href=DomainForm.dataset.redirect || '/settings';
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
const projectOrganizerForm=document.getElementById('project-organizer-form');
if(projectOrganizerForm){
  const organizerStatus=document.getElementById('project-organizer-status');
  const organizerState=projectOrganizerForm.querySelector('[data-organizer-state]');
  const organizerHandoff=projectOrganizerForm.querySelector('[data-organizer-handoff]');
  const organizerSelect=projectOrganizerForm.querySelector('select[name="organizer_agent"]');
  projectOrganizerForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    organizerStatus.textContent='オーガナイザー設定を保存しています…';
    const response=await fetch(projectOrganizerForm.dataset.action,{method:'POST',body:new URLSearchParams(new FormData(projectOrganizerForm))});
    if(!response.ok){await notifyResponseError(response,organizerStatus);return;}
    const result=await response.json();
    if(result.fallback){
      organizerState.textContent='プロジェクト固有は未設定';
      organizerHandoff.textContent=`ビルトインがワーク化と割り振りを行います: ${result.dispatch_agent}`;
      if(organizerSelect){organizerSelect.value='';}
    }else{
      organizerState.textContent=`プロジェクト固有: ${result.organizer_agent}`;
      organizerHandoff.textContent=`現在の引き継ぎ: ${result.organizer_agent}`;
    }
    organizerStatus.textContent='オーガナイザー設定を保存しました。';
  });
}
const skillPackageForm=document.getElementById('skill-package-form');
if(skillPackageForm){
  const skillPackageStatus=document.getElementById('skill-package-status');
  const skillSourceKind=document.getElementById('skill-source-kind');
  const skillSourceChoices=[...skillPackageForm.querySelectorAll('[data-skill-source-choice]')];
  const skillSourceTitle=skillPackageForm.querySelector('[data-skill-source-title]');
  const skillSourceCopy=skillPackageForm.querySelector('[data-skill-source-copy]');
  const skillSourceFieldsCopy=skillPackageForm.querySelector('[data-skill-source-fields]');
  const skillSourceFields=[...skillPackageForm.querySelectorAll('[data-skill-source-field]')];
  const skillIdInput=skillPackageForm.querySelector('input[name="id"]');
  const skillPrimaryLabel=skillPackageForm.querySelector('[data-skill-primary-label]');
  const skillSourceLabel=skillPackageForm.querySelector('[data-skill-source-label]');
  const skillPathLabel=skillPackageForm.querySelector('[data-skill-path-label]');
  const skillSourceGuidance={
    'skill-creator':{
      title:'Skill Creator',
      copy:'作成済みのスキルフォルダを登録します。SKILL.mdのnameを使えるため、通常はPathだけで足ります。',
      fields:'必要: フォルダPath。スキル名はSKILL.mdにnameがない場合だけ。'
    },
    'clawhub':{
      title:'ClawHub',
      copy:'ClawHubのカタログIDを取り込みます。OpenClawまたはClawHub CLIがなければnpx clawhub@latestを使います。',
      fields:'必要: スキル名。取り込み後、実体Pathを登録します。'
    },
    'vercel':{
      title:'Vercel Skills',
      copy:'Vercel Skillsをnpx skills addで選択した範囲とツールだけへ取り込みます。複数スキルrepoでは owner/repo@skill を指定できます。',
      fields:'必要: package ID、対象ツール。Project範囲が既定です。'
    },
    'local':{
      title:'Local',
      copy:'手元のスキルフォルダを登録します。SKILL.mdのnameを使えるため、通常はPathだけで足ります。',
      fields:'必要: フォルダPath。スキル名はSKILL.mdにnameがない場合だけ。'
    },
    'git':{
      title:'Git',
      copy:'Gitリポジトリ上のスキルを登録します。VersionやサブPathの固定は詳細設定で行います。',
      fields:'必要: Repo URL、スキル名'
    }
  };
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
      'clawhub':[],
      'vercel':['vercel_options'],
      'git':['source'],
    };
    const visible=new Set(visibleByKind[kind] || ['source']);
    skillSourceFields.forEach((field)=>setFieldVisible(field,visible.has(field.dataset.skillSourceField)));
    const sourceInput=skillPackageForm.querySelector('input[name="source"]');
    const pathInput=skillPackageForm.querySelector('input[name="path"]');
    if(skillIdInput){
      const idRequired=kind==='clawhub' || kind==='vercel' || kind==='git';
      skillIdInput.required=idRequired;
      if(kind==='local' || kind==='skill-creator'){
        skillIdInput.placeholder='SKILL.mdから自動取得…';
      }else if(kind==='vercel'){
        skillIdInput.placeholder='hachiware-labs/hachi-search…';
      }else{
        skillIdInput.placeholder='react-review…';
      }
    }
    if(sourceInput){sourceInput.required=kind==='git';}
    if(pathInput){pathInput.required=visible.has('path') && (kind==='skill-creator' || kind==='local');}
    if(skillPrimaryLabel){
      if(kind==='local' || kind==='skill-creator'){
        skillPrimaryLabel.textContent='スキル名（任意）';
      }else if(kind==='vercel'){
        skillPrimaryLabel.textContent='Package ID';
      }else{
        skillPrimaryLabel.textContent='スキル名';
      }
    }
    if(skillSourceLabel){
      skillSourceLabel.textContent=kind==='git' ? 'Repo URL' : 'Source';
    }
    if(skillPathLabel){
      skillPathLabel.textContent=(kind==='local' || kind==='skill-creator') ? 'フォルダPath' : 'Path';
    }
    skillSourceChoices.forEach((button)=>{
      const selected=button.dataset.skillSourceChoice===kind;
      button.classList.toggle('active',selected);
      button.setAttribute('aria-pressed',selected ? 'true' : 'false');
    });
    const guidance=skillSourceGuidance[kind] || skillSourceGuidance['skill-creator'];
    if(skillSourceTitle){skillSourceTitle.textContent=guidance.title;}
    if(skillSourceCopy){skillSourceCopy.textContent=guidance.copy;}
    if(skillSourceFieldsCopy){skillSourceFieldsCopy.textContent=guidance.fields;}
  }
  skillSourceChoices.forEach((button)=>{
    button.addEventListener('click',()=>{
      if(skillSourceKind){skillSourceKind.value=button.dataset.skillSourceChoice;}
      syncSkillSourceFields();
    });
  });
  if(skillSourceKind){skillSourceKind.addEventListener('change',syncSkillSourceFields);}
  syncSkillSourceFields();
  skillPackageForm.addEventListener('submit',async(event)=>{
    event.preventDefault();
    if(skillSourceKind && skillSourceKind.value==='vercel'){
      const checkedTargets=[...skillPackageForm.querySelectorAll('input[name="install_targets"]:checked:not(:disabled)')];
      if(!checkedTargets.length){
        skillPackageStatus.textContent='';
        notify('Vercel Skillsは対象ツールを1つ以上選んでください。','error');
        return;
      }
    }
    skillPackageStatus.textContent='スキルを取り込んで登録しています…';
    const response=await fetch(skillPackageForm.dataset.action,{method:'POST',body:new URLSearchParams(new FormData(skillPackageForm))});
    if(!response.ok){await notifyResponseError(response,skillPackageStatus);return;}
    skillPackageStatus.textContent='スキルを取り込んで登録しました。';
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
    notify('ワークが完了しました。','success');
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
    notify('差し戻しました。','info');
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
