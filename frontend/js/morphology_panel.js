class MorphologyPanel {
    constructor() {
        this.currentSiteId = null;
        this.currentResult = null;
    }

    init() {
        document.getElementById('analyzeBtn').addEventListener('click', () => this.analyze());
    }

    setSite(siteId) {
        this.currentSiteId = siteId;
        this.currentResult = null;
        this.clearResult();
    }

    clearResult() {
        document.getElementById('morphologyResult').innerHTML = 
            '<p class="placeholder">点击按钮进行形态分析</p>';
    }

    async analyze() {
        if (!this.currentSiteId) {
            alert('请先选择一个城市遗址');
            return;
        }

        const btn = document.getElementById('analyzeBtn');
        btn.disabled = true;
        btn.textContent = '分析中...';

        try {
            const result = await API.analyzeMorphology(this.currentSiteId);
            this.currentResult = result;
            this.renderResult(result);
        } catch (error) {
            console.error('形态分析失败:', error);
            document.getElementById('morphologyResult').innerHTML = 
                `<p class="placeholder" style="color: #e74c3c">分析失败: ${error.message}</p>`;
        } finally {
            btn.disabled = false;
            btn.textContent = '开始形态分析';
        }
    }

    renderResult(result) {
        const container = document.getElementById('morphologyResult');
        
        container.innerHTML = `
            <div class="detail-section">
                <h4>空间句法指标</h4>
                <div class="metric-grid">
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.integration_global, 4)}</div>
                        <div class="metric-label">全局整合度</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.integration_local, 4)}</div>
                        <div class="metric-label">局部整合度</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.choice_global, 2)}</div>
                        <div class="metric-label">全局选择度</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.choice_local, 2)}</div>
                        <div class="metric-label">局部选择度</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.mean_depth, 2)}</div>
                        <div class="metric-label">平均深度</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.connectivity, 2)}</div>
                        <div class="metric-label">连接度</div>
                    </div>
                </div>
            </div>

            <div class="detail-section">
                <h4>分形维数</h4>
                <div class="metric-grid">
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.boundary_fractal_dimension, 4)}</div>
                        <div class="metric-label">边界分形维数</div>
                        <div class="metric-meta">
                            ${this.renderFractalConfidence(
                                result.boundary_fd_confidence_lower,
                                result.boundary_fd_confidence_upper,
                                result.boundary_fd_quality
                            )}
                        </div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.road_network_fractal_dimension, 4)}</div>
                        <div class="metric-label">路网分形维数</div>
                        <div class="metric-meta">
                            ${this.renderFractalConfidence(
                                result.road_fd_confidence_lower,
                                result.road_fd_confidence_upper,
                                result.road_fd_quality
                            )}
                        </div>
                    </div>
                </div>
                <div class="metric-note">
                    <small>采用盒计数法、周长-面积法、分规法三算法加权融合，Bootstrap 500次重采样计算95%置信区间</small>
                </div>
            </div>

            <div class="detail-section">
                <h4>形态指标</h4>
                <div class="metric-grid">
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.compactness_index, 4)}</div>
                        <div class="metric-label">紧凑度指数</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.elongation_ratio, 4)}</div>
                        <div class="metric-label">延展率</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.road_density, 2)}</div>
                        <div class="metric-label">道路密度 (km/km²)</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.intersection_density, 2)}</div>
                        <div class="metric-label">交叉口密度 (个/km²)</div>
                    </div>
                </div>
            </div>

            <div class="detail-section">
                <h4>功能区指标</h4>
                <div class="metric-grid">
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.functional_diversity, 4)}</div>
                        <div class="metric-label">功能多样性</div>
                    </div>
                    <div class="metric-card">
                        <div class="metric-value">${this.formatNum(result.functional_mixing, 4)}</div>
                        <div class="metric-label">功能混合度</div>
                    </div>
                </div>
            </div>
        `;
    }

    formatNum(value, decimals) {
        if (value === null || value === undefined || isNaN(value)) {
            return 'N/A';
        }
        return value.toFixed(decimals);
    }

    renderFractalConfidence(lower, upper, quality) {
        const parts = [];
        if (lower !== null && lower !== undefined && upper !== null && upper !== undefined) {
            parts.push(`95% CI: [${this.formatNum(lower, 3)}, ${this.formatNum(upper, 3)}]`);
        }
        if (quality !== null && quality !== undefined) {
            const qClass = quality >= 0.7 ? 'quality-high' : quality >= 0.4 ? 'quality-medium' : 'quality-low';
            const qLabel = quality >= 0.7 ? '高' : quality >= 0.4 ? '中' : '低';
            parts.push(`<span class="${qClass}">数据质量: ${qLabel} (${this.formatNum(quality, 2)})</span>`);
        }
        return parts.join('<br>');
    }

    getCurrentResult() {
        return this.currentResult;
    }
}
