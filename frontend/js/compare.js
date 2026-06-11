class CompareModal {
    constructor() {
        this.allSites = [];
    }

    init(sites) {
        this.allSites = sites;
        
        document.getElementById('compareBtn').addEventListener('click', () => this.open());
        document.getElementById('closeCompare').addEventListener('click', () => this.close());
        document.getElementById('compareModal').addEventListener('click', (e) => {
            if (e.target.id === 'compareModal') this.close();
        });
        
        document.getElementById('doCompareBtn').addEventListener('click', () => this.doCompare());

        this.populateSelects();
    }

    populateSelects() {
        const select1 = document.getElementById('compareSite1');
        const select2 = document.getElementById('compareSite2');
        
        const options = this.allSites
            .sort((a, b) => (a.dynasty_name || '').localeCompare(b.dynasty_name || ''))
            .map(site => `<option value="${site.id}">${site.dynasty_name} - ${site.name}</option>`)
            .join('');
        
        select1.innerHTML = options;
        select2.innerHTML = options;
        
        if (this.allSites.length >= 2) {
            select2.value = this.allSites[1].id;
        }
    }

    open() {
        document.getElementById('compareModal').classList.remove('hidden');
    }

    close() {
        document.getElementById('compareModal').classList.add('hidden');
    }

    async doCompare() {
        const site1Id = parseInt(document.getElementById('compareSite1').value);
        const site2Id = parseInt(document.getElementById('compareSite2').value);
        
        if (!site1Id || !site2Id) {
            alert('请选择两个城市遗址进行对比');
            return;
        }
        
        if (site1Id === site2Id) {
            alert('请选择两个不同的城市遗址');
            return;
        }

        const btn = document.getElementById('doCompareBtn');
        btn.disabled = true;
        btn.textContent = '对比中...';

        try {
            const result = await API.compareSites([site1Id, site2Id]);
            this.renderCompareResult(result);
        } catch (error) {
            console.error('对比失败:', error);
            document.getElementById('compareResult').innerHTML = 
                `<p style="color: #e74c3c">对比失败: ${error.message}</p>`;
        } finally {
            btn.disabled = false;
            btn.textContent = '开始对比';
        }
    }

    renderCompareResult(data) {
        const container = document.getElementById('compareResult');
        
        if (!data || data.length < 2) {
            container.innerHTML = '<p>无法获取对比数据</p>';
            return;
        }

        const site1 = data[0];
        const site2 = data[1];
        
        const metrics = [
            { key: 'population', label: '人口估算', format: v => v ? v.toLocaleString() + ' 人' : 'N/A' },
            { key: 'area_sq_km', label: '面积', format: v => v ? v.toFixed(2) + ' km²' : 'N/A' },
        ];

        const morphMetrics = [
            { key: 'integration_global', label: '全局整合度', format: v => v?.toFixed(4) || 'N/A' },
            { key: 'choice_global', label: '全局选择度', format: v => v?.toFixed(2) || 'N/A' },
            { key: 'boundary_fractal_dimension', label: '边界分形维数', format: v => v?.toFixed(4) || 'N/A' },
            { key: 'road_network_fractal_dimension', label: '路网分形维数', format: v => v?.toFixed(4) || 'N/A' },
            { key: 'compactness_index', label: '紧凑度指数', format: v => v?.toFixed(4) || 'N/A' },
            { key: 'elongation_ratio', label: '延展率', format: v => v?.toFixed(4) || 'N/A' },
            { key: 'functional_diversity', label: '功能多样性', format: v => v?.toFixed(4) || 'N/A' },
        ];

        let html = `
            <table class="compare-table">
                <thead>
                    <tr>
                        <th>指标</th>
                        <th>${site1.name} (${site1.dynasty})</th>
                        <th>${site2.name} (${site2.dynasty})</th>
                        <th>差异</th>
                    </tr>
                </thead>
                <tbody>
        `;

        metrics.forEach(m => {
            const v1 = site1[m.key];
            const v2 = site2[m.key];
            let diff = '-';
            
            if (typeof v1 === 'number' && typeof v2 === 'number' && v2 !== 0) {
                const pct = ((v1 - v2) / v2 * 100).toFixed(1);
                diff = `${pct > 0 ? '+' : ''}${pct}%`;
            }
            
            html += `
                <tr>
                    <td>${m.label}</td>
                    <td>${m.format(v1)}</td>
                    <td>${m.format(v2)}</td>
                    <td>${diff}</td>
                </tr>
            `;
        });

        const morph1 = site1.morphology || {};
        const morph2 = site2.morphology || {};

        morphMetrics.forEach(m => {
            const v1 = morph1[m.key];
            const v2 = morph2[m.key];
            let diff = '-';
            
            if (typeof v1 === 'number' && typeof v2 === 'number' && v2 !== 0) {
                const pct = ((v1 - v2) / v2 * 100).toFixed(1);
                diff = `${pct > 0 ? '+' : ''}${pct}%`;
            }
            
            html += `
                <tr>
                    <td>${m.label}</td>
                    <td>${m.format(v1)}</td>
                    <td>${m.format(v2)}</td>
                    <td>${diff}</td>
                </tr>
            `;
        });

        html += `
                </tbody>
            </table>
        `;

        container.innerHTML = html;
    }
}
