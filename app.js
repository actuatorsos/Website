// =========================================
// I18N — translation helper + language switch
// =========================================

let currentLang = localStorage.getItem('actuators-lang') || 'en';

function t(key) {
    return (TRANSLATIONS[currentLang] && TRANSLATIONS[currentLang][key])
        || (TRANSLATIONS['en'] && TRANSLATIONS['en'][key])
        || key;
}

function setLanguage(lang) {
    currentLang = lang;
    localStorage.setItem('actuators-lang', lang);

    document.documentElement.dir  = lang === 'ar' ? 'rtl' : 'ltr';
    document.documentElement.lang = lang;

    // Plain text nodes
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        el.textContent = t(key);
    });

    // HTML nodes (contain <br>, <span> etc.)
    document.querySelectorAll('[data-i18n-html]').forEach(el => {
        const key = el.getAttribute('data-i18n-html');
        el.innerHTML = t(key);
    });

    // Placeholder attributes
    document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
        el.placeholder = t(el.getAttribute('data-i18n-placeholder'));
    });

    // Active state on toggle buttons
    document.querySelectorAll('.lang-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.lang === lang);
    });
}

document.addEventListener('DOMContentLoaded', () => {

    // Wire up language toggle buttons
    document.querySelectorAll('.lang-btn').forEach(btn => {
        btn.addEventListener('click', () => setLanguage(btn.dataset.lang));
    });

    // Apply stored/default language on load
    setLanguage(currentLang);

    // =========================================
    // HERO CANVAS — drifting nodes → snap grid
    // =========================================

    const heroCanvas = document.getElementById('hero-canvas');
    const heroWrap   = document.getElementById('hero-canvas-wrap');

    if (heroCanvas && heroWrap) {
        const hCtx = heroCanvas.getContext('2d');
        const N = 90;
        let hW, hH, heroNodes = [], snapped = false;

        function resizeHero() {
            hW = heroCanvas.width  = heroWrap.offsetWidth;
            hH = heroCanvas.height = heroWrap.offsetHeight;
            calcTargets();
        }

        for (let i = 0; i < N; i++) {
            heroNodes.push({
                x: Math.random() * (heroWrap.offsetWidth  || 1200),
                y: Math.random() * (heroWrap.offsetHeight || 800),
                tx: 0, ty: 0,
                vx: (Math.random() - 0.5) * 0.4,
                vy: (Math.random() - 0.5) * 0.4,
                pulse: Math.random() * Math.PI * 2,
                big: i % 9 === 0,
                accent: i % 13 === 0
            });
        }

        function calcTargets() {
            if (!hW || !hH) return;
            const cols = Math.min(12, Math.floor(hW / 100));
            const rows = Math.ceil(N / cols);
            const gx   = hW / (cols + 1);
            const gy   = hH / (rows + 1);
            heroNodes.forEach((n, i) => {
                n.tx = (1 + (i % cols)) * gx;
                n.ty = (1 + Math.floor(i / cols)) * gy;
            });
        }

        resizeHero();
        window.addEventListener('resize', resizeHero);

        // Snap after 2 seconds
        setTimeout(() => { snapped = true; }, 2000);

        function drawHero() {
            hCtx.clearRect(0, 0, hW, hH);

            if (snapped) {
                const cols = Math.min(12, Math.floor(hW / 100));
                const rows = Math.ceil(N / cols);

                // Horizontal lines
                for (let r = 0; r < rows; r++) {
                    for (let c = 0; c < cols - 1; c++) {
                        const i = r * cols + c;
                        const j = i + 1;
                        if (i < N && j < N) {
                            hCtx.strokeStyle = 'rgba(0, 183, 194, 0.12)';
                            hCtx.lineWidth = 0.5;
                            hCtx.beginPath();
                            hCtx.moveTo(heroNodes[i].x, heroNodes[i].y);
                            hCtx.lineTo(heroNodes[j].x, heroNodes[j].y);
                            hCtx.stroke();
                        }
                    }
                }

                // Vertical lines
                for (let r = 0; r < rows - 1; r++) {
                    for (let c = 0; c < cols; c++) {
                        const i = r * cols + c;
                        const j = (r + 1) * cols + c;
                        if (i < N && j < N) {
                            hCtx.strokeStyle = 'rgba(0, 183, 194, 0.06)';
                            hCtx.lineWidth = 0.5;
                            hCtx.beginPath();
                            hCtx.moveTo(heroNodes[i].x, heroNodes[i].y);
                            hCtx.lineTo(heroNodes[j].x, heroNodes[j].y);
                            hCtx.stroke();
                        }
                    }
                }
            }

            heroNodes.forEach(n => {
                if (snapped) {
                    n.x += (n.tx - n.x) * 0.04;
                    n.y += (n.ty - n.y) * 0.04;
                } else {
                    n.x += n.vx;
                    n.y += n.vy;
                    if (n.x < 0 || n.x > hW) n.vx *= -1;
                    if (n.y < 0 || n.y > hH) n.vy *= -1;
                }

                n.pulse += 0.015;
                const alpha = 0.25 + Math.sin(n.pulse) * 0.18;
                const size  = n.big ? 5 : 2.5;

                if (n.big) {
                    hCtx.shadowColor = n.accent ? '#FF6A2A' : '#00B7C2';
                    hCtx.shadowBlur  = 10;
                }

                hCtx.fillStyle = n.accent
                    ? `rgba(255, 106, 42, ${alpha})`
                    : `rgba(0, 183, 194, ${alpha})`;
                hCtx.fillRect(n.x - size / 2, n.y - size / 2, size, size);
                hCtx.shadowBlur = 0;
            });

            requestAnimationFrame(drawHero);
        }

        requestAnimationFrame(drawHero);
    }

    // =========================================
    // DASHBOARD DEMO
    // =========================================

    const vizContainer    = document.getElementById('system-visualization');
    const logsOutput      = document.getElementById('logs-output');
    const logTimestamp    = document.getElementById('log-timestamp');
    const nodesTableBody  = document.getElementById('nodes-table-body');
    const fullLogsList    = document.getElementById('full-logs-list');
    const nodeSearch      = document.getElementById('node-search');
    const addNodeBtn      = document.getElementById('add-node-btn');
    const topoSvg         = document.getElementById('topo-svg');
    const logFilter       = document.getElementById('log-filter');
    const purgeBtn        = document.getElementById('purge-btn');

    const state = {
        nodes: [
            { id: '0x3F2A1', geometry: 'QUAD_A', latency: '4ms',  status: 'online'  },
            { id: '0x7B9C4', geometry: 'QUAD_B', latency: '12ms', status: 'online'  },
            { id: '0x1E5D8', geometry: 'CORE_M', latency: '1ms',  status: 'warning' },
            { id: '0xBC4E1', geometry: 'QUAD_A', latency: '6ms',  status: 'online'  }
        ],
        allLogs: []
    };

    // --- Visual node grid ---
    let vizNodes = [];
    const COL_COUNT  = 12;
    const SPACING_X  = 72;
    const SPACING_Y  = 72;
    let gridActive   = false;

    function initVisuals() {
        if (!vizContainer) return;
        vizContainer.innerHTML = '<canvas id="connections-canvas"></canvas>'
            + '<div class="center-content">'
            + `<h2 class="system-title">${t('demo.infra_title')}</h2>`
            + `<p class="system-subtitle">${t('demo.infra_sub')}</p>`
            + '</div>';

        vizNodes = [];
        for (let i = 0; i < 48; i++) spawnVizNode();
        setTimeout(alignGrid, 600);
    }

    function spawnVizNode() {
        const el = document.createElement('div');
        el.className = 'system-node';
        const rx = Math.random() * (vizContainer.offsetWidth  || 800);
        const ry = Math.random() * (vizContainer.offsetHeight || 340);
        el.style.cssText = `
            position:absolute;width:8px;height:8px;
            background:var(--cyan-primary);
            left:${rx}px;top:${ry}px;
            transition:all 1.2s cubic-bezier(1,0,0,1);
            opacity:0.35;border:1px solid rgba(255,255,255,0.25);z-index:2;`;
        vizContainer.appendChild(el);
        vizNodes.push({ el, grid: { x: 0, y: 0 } });
    }

    function alignGrid() {
        gridActive = true;
        const rows   = Math.ceil(vizNodes.length / COL_COUNT);
        const startX = vizContainer.offsetWidth  / 2 - ((COL_COUNT - 1) * SPACING_X / 2);
        const startY = (vizContainer.offsetHeight || 340) / 2 - ((rows - 1) * SPACING_Y / 2);

        vizNodes.forEach((n, i) => {
            n.grid.x = startX + (i % COL_COUNT) * SPACING_X;
            n.grid.y = startY + Math.floor(i / COL_COUNT) * SPACING_Y;
            n.el.style.left    = `${n.grid.x}px`;
            n.el.style.top     = `${n.grid.y}px`;
            n.el.style.opacity = '1';
        });

        drawConnections();
        renderTopology();
    }

    function drawConnections() {
        const canvas = document.getElementById('connections-canvas');
        if (!canvas || !vizContainer) return;
        const ctx = canvas.getContext('2d');
        canvas.width  = vizContainer.offsetWidth;
        canvas.height = vizContainer.offsetHeight || 340;
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        if (!gridActive) return;

        const rows = Math.ceil(vizNodes.length / COL_COUNT);
        ctx.strokeStyle = 'rgba(0, 183, 194, 0.2)';
        ctx.lineWidth   = 0.8;

        for (let r = 0; r < rows; r++) {
            const s = r * COL_COUNT;
            const e = Math.min(s + COL_COUNT - 1, vizNodes.length - 1);
            if (vizNodes[s] && vizNodes[e]) {
                ctx.beginPath();
                ctx.moveTo(vizNodes[s].grid.x + 4, vizNodes[s].grid.y + 4);
                ctx.lineTo(vizNodes[e].grid.x + 4, vizNodes[e].grid.y + 4);
                ctx.stroke();
            }
        }
    }

    function renderTopology() {
        if (!topoSvg) return;
        topoSvg.innerHTML = '';
        const gapX = 110;

        state.nodes.forEach((n, i) => {
            const x = 50 + (i % 6) * gapX;
            const y = 50;

            if (i > 0) {
                const px = 50 + ((i - 1) % 6) * gapX;
                const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
                line.setAttribute('x1', px + 20); line.setAttribute('y1', y + 10);
                line.setAttribute('x2', x + 20);  line.setAttribute('y2', y + 10);
                line.setAttribute('stroke', 'rgba(0, 183, 194, 0.35)');
                line.setAttribute('stroke-width', '1');
                topoSvg.appendChild(line);
            }

            const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
            rect.setAttribute('x', x); rect.setAttribute('y', y);
            rect.setAttribute('width', 40); rect.setAttribute('height', 20);
            rect.setAttribute('fill',   n.status === 'online' ? 'rgba(0, 183, 194, 0.15)' : 'rgba(255, 106, 42, 0.15)');
            rect.setAttribute('stroke', n.status === 'online' ? '#00B7C2' : '#FF6A2A');
            topoSvg.appendChild(rect);
        });
    }

    function addLog(message, priority = 'INFO') {
        const ts  = new Date().toISOString().split('T')[1].split('.')[0];
        const raw = `[${ts}] [${priority}] ${message}`;
        state.allLogs.unshift({ ts, priority, message, raw });

        if (logsOutput) {
            const el = document.createElement('div');
            el.className = 'log-entry';
            el.textContent = raw;
            logsOutput.insertBefore(el, logsOutput.firstChild);
            if (logsOutput.children.length > 8) logsOutput.removeChild(logsOutput.lastChild);
        }

        if (logTimestamp) logTimestamp.textContent = ts;
        renderFullLogs();
    }

    function renderFullLogs(filter = '') {
        if (!fullLogsList) return;
        const filtered = filter
            ? state.allLogs.filter(l => l.raw.toLowerCase().includes(filter.toLowerCase()))
            : state.allLogs;

        fullLogsList.innerHTML = filtered.map(l =>
            `<div class="log-entry"><span style="color:var(--cyan-primary)">${l.ts}</span> | <b>${l.priority}</b> | ${l.message}</div>`
        ).join('');
    }

    function renderNodesTable(filter = '') {
        if (!nodesTableBody) return;
        const list = state.nodes.filter(n => n.id.toLowerCase().includes(filter.toLowerCase()));
        nodesTableBody.innerHTML = list.map(n => `
            <tr class="node-row" data-id="${n.id}">
                <td>${n.id}</td>
                <td>${n.geometry}</td>
                <td style="color:var(--cyan-primary)">${n.latency}</td>
                <td><span class="status-tag ${n.status}">${n.status.toUpperCase()}</span></td>
            </tr>
        `).join('');
    }

    // Demo nav switching
    document.querySelectorAll('.dnav-link').forEach(link => {
        link.addEventListener('click', e => {
            e.preventDefault();
            document.querySelectorAll('.dnav-link').forEach(l => l.classList.remove('active'));
            link.classList.add('active');
            const view = link.dataset.view;
            document.querySelectorAll('.demo-view').forEach(v => {
                v.classList.remove('active');
                if (v.id === `${view}-view`) v.classList.add('active');
            });
            if (view === 'system') initVisuals();
        });
    });

    if (nodeSearch)  nodeSearch.addEventListener('input',  e => renderNodesTable(e.target.value));
    if (logFilter)   logFilter.addEventListener('input',   e => renderFullLogs(e.target.value));

    if (addNodeBtn) {
        addNodeBtn.addEventListener('click', () => {
            const id = '0x' + Math.floor(Math.random() * 0xFFFFF).toString(16).toUpperCase();
            state.nodes.push({ id, geometry: 'MOD_Z', latency: '2ms', status: 'online' });
            addLog(`MANUAL_NODE_INJECTION: ${id}`);
            renderNodesTable(nodeSearch ? nodeSearch.value : '');
            renderTopology();
        });
    }

    if (purgeBtn) {
        purgeBtn.addEventListener('click', () => {
            state.allLogs = [];
            if (fullLogsList) fullLogsList.innerHTML = '';
            if (logsOutput)   logsOutput.innerHTML  = '';
            addLog('BUFFER_PURGED // MANUAL_OVERRIDE', 'WARN');
        });
    }

    // Node row → modal
    if (nodesTableBody) {
        nodesTableBody.addEventListener('click', e => {
            const row = e.target.closest('.node-row');
            if (row) openModal(row.dataset.id);
        });
    }

    // Modal
    const modal    = document.getElementById('node-modal');
    const closeBtn = document.getElementById('close-modal');

    function openModal(nodeId) {
        const n = state.nodes.find(x => x.id === nodeId);
        if (!n || !modal) return;
        document.getElementById('modal-id').textContent       = n.id;
        document.getElementById('modal-geometry').textContent = n.geometry;
        document.getElementById('modal-latency').textContent  = n.latency;
        document.getElementById('modal-status').textContent   = n.status.toUpperCase();
        document.getElementById('modal-packet').textContent   = '0x' + Math.floor(Math.random() * 0xFFFF).toString(16).toUpperCase();
        modal.style.display = 'flex';
    }

    if (closeBtn) closeBtn.onclick = () => { modal.style.display = 'none'; };
    if (modal)    modal.addEventListener('click', e => { if (e.target === modal) modal.style.display = 'none'; });

    // Initialize button
    const initBtn = document.getElementById('initialize-button');
    if (initBtn) {
        initBtn.addEventListener('click', () => {
            addLog('INITIALIZATION_SEQUENCE_TRIGGERED', 'WARN');
            initBtn.textContent = t('demo.init_btn_loading');
            initBtn.disabled    = true;
            setTimeout(() => {
                initBtn.textContent = t('demo.init_btn_done');
                initBtn.style.background = 'rgba(0,183,194,0.3)';
                initBtn.style.color      = 'var(--cyan-primary)';
                addLog('INITIALIZATION_COMPLETE // ALL_NODES_SYNCED');
            }, 1800);
        });
    }

    // Metrics update loop
    setInterval(() => {
        const v    = (38 + Math.random() * 24).toFixed(1);
        const vEl  = document.getElementById('core-load-val');
        const bar  = document.getElementById('core-load-bar');
        if (vEl)  vEl.textContent  = `${v}%`;
        if (bar)  bar.style.width  = `${v}%`;
        if (Math.random() > 0.6) addLog(`CORE_LOAD_UPDATE: ${v}%`);
    }, 3000);

    // Boot sequence logs
    addLog('SYSTEM_CORE_ONLINE');
    addLog('ARABIC_INTERFACE_MODULE_LOADED');
    addLog('LOW_BANDWIDTH_MODE_ACTIVE // TARGET: 4.05_MBPS');
    addLog('AUDIT_LOGGING_ENABLED');

    initVisuals();
    renderNodesTable();

    // =========================================
    // SCROLL REVEAL
    // =========================================

    const revealTargets = [
        '.pm-block', '.fg-item', '.layer-card', '.ts-block',
        '.ml-item', '.rs-row', '.team-card', '.model-card',
        '.proj-row', '.traction-stats', '.milestones',
        '.failure-grid', '.connectivity-panel', '.rs-table',
        '.team-evidence', '.projections', '.cta-box'
    ].join(', ');

    const revealObserver = new IntersectionObserver((entries) => {
        entries.forEach((entry, i) => {
            if (entry.isIntersecting) {
                // Stagger within batches
                const delay = (i % 6) * 70;
                setTimeout(() => {
                    entry.target.style.opacity   = '1';
                    entry.target.style.transform = 'translateY(0)';
                }, delay);
                revealObserver.unobserve(entry.target);
            }
        });
    }, { threshold: 0.08, rootMargin: '0px 0px -40px 0px' });

    document.querySelectorAll(revealTargets).forEach(el => {
        el.style.opacity    = '0';
        el.style.transform  = 'translateY(18px)';
        el.style.transition = 'opacity 0.7s ease, transform 0.7s ease';
        revealObserver.observe(el);
    });

    // =========================================
    // HEADER SCROLL EFFECT
    // =========================================

    const siteHeader = document.getElementById('site-header');
    window.addEventListener('scroll', () => {
        if (siteHeader) {
            siteHeader.classList.toggle('scrolled', window.scrollY > 48);
        }
    }, { passive: true });

    // =========================================
    // ACTIVE NAV ON SCROLL
    // =========================================

    const sections  = document.querySelectorAll('section[id]');
    const navLinks  = document.querySelectorAll('.nav-link');

    const navObserver = new IntersectionObserver(entries => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                navLinks.forEach(l => l.classList.remove('active'));
                const active = document.querySelector(`.nav-link[href="#${entry.target.id}"]`);
                if (active) active.classList.add('active');
            }
        });
    }, { threshold: 0.4 });

    sections.forEach(s => navObserver.observe(s));

    // =========================================
    // MOBILE HAMBURGER (shows nav on click)
    // =========================================

    const hamburger = document.getElementById('hamburger');
    const mainNav   = document.getElementById('main-nav');

    if (hamburger && mainNav) {
        hamburger.addEventListener('click', () => {
            const open = mainNav.style.display === 'flex';
            mainNav.style.cssText = open
                ? ''
                : 'display:flex;flex-direction:column;position:absolute;top:68px;left:0;right:0;background:rgba(11,26,42,0.97);border-bottom:1px solid var(--cyan);padding:20px 24px;gap:20px;z-index:999;';
        });

        // Close on link click
        mainNav.querySelectorAll('a').forEach(a => {
            a.addEventListener('click', () => { mainNav.style.cssText = ''; });
        });
    }

    // =========================================
    // CTA FORM
    // =========================================

    const accessForm  = document.getElementById('access-form');
    const formSuccess = document.getElementById('form-success');

    if (accessForm) {
        accessForm.addEventListener('submit', e => {
            e.preventDefault();
            const email = document.getElementById('cta-email').value;
            const role  = document.getElementById('cta-role').value || 'UNKNOWN';
            accessForm.style.display = 'none';
            if (formSuccess) {
                formSuccess.textContent = t('cta.success');
                formSuccess.style.display = 'block';
            }
            addLog(`ACCESS_REQUEST_QUEUED: ${email} // ROLE: ${role.toUpperCase()}`);
        });
    }

    // =========================================
    // PROJECTION BARS ANIMATE ON SCROLL
    // =========================================

    const projObserver = new IntersectionObserver(entries => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.querySelectorAll('.pr-bar').forEach(bar => {
                    const target = bar.dataset.target || '0';
                    bar.style.width = '0';
                    setTimeout(() => { bar.style.width = `${target}%`; }, 200);
                });
                projObserver.unobserve(entry.target);
            }
        });
    }, { threshold: 0.3 });

    document.querySelectorAll('.projections').forEach(el => projObserver.observe(el));

});
