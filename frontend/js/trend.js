class TrendAnalyzer {
    constructor() {
        this.currentTrend = null;
        this.chartCanvas = null;
        this.chartCtx = null;
    }

    init() {
        document.getElementById('analyzeTrendBtn').addEventListener('click', () => this.analyze());
        this.chartCanvas = document.getElementById('trendChart');
        this.chartCtx = this.chartCanvas.getContext('2d');
    }

    async analyze() {
        const indicator = document.getElementById('trendIndicator').value;
        const indicatorName = CONFIG.MORPHOLOGY_INDICATORS[indicator] || indicator;

        const btn = document.getElementById('analyzeTrendBtn');
        btn.disabled = true;
        btn.textContent = '分析中...';

        try {
            const result = await API.analyzeTrends(indicator);
            this.currentTrend = result;
            this.renderResult(result, indicatorName);
            this.renderChart(result, indicatorName);
        } catch (error) {
            console.error('趋势分析失败:', error);
            document.getElementById('trendResult').innerHTML = 
                `<p class="placeholder" style="color: #e74c3c">分析失败: ${error.message}</p>`;
        } finally {
            btn.disabled = false;
            btn.textContent = '分析演化趋势';
        }
    }

    renderResult(result, indicatorName) {
        const container = document.getElementById('trendResult');
        
        const directionClass = result.trend_direction === 'increasing' ? 'increasing' :
                               result.trend_direction === 'decreasing' ? 'decreasing' : 'no-trend';
        
        const directionText = result.trend_direction === 'increasing' ? '上升趋势' :
                             result.trend_direction === 'decreasing' ? '下降趋势' : '无显著趋势';

        const significanceText = result.trend_significance ? '显著' : '不显著';

        container.innerHTML = `
            <div class="trend-summary">
                <h4>${indicatorName}演化趋势</h4>
                <div style="margin: 8px 0;">
                    <span class="trend-direction ${directionClass}">${directionText}</span>
                    <span style="font-size: 12px; color: #666; margin-left: 8px;">(${significanceText})</span>
                </div>
                <div class="trend-stats">
                    <span class="trend-label">Z统计量:</span>
                    <span class="trend-value">${result.mk_z_score ? result.mk_z_score.toFixed(4) : 'N/A'}</span>
                    
                    <span class="trend-label">P值:</span>
                    <span class="trend-value">${result.mk_p_value ? result.mk_p_value.toFixed(4) : 'N/A'}</span>
                    
                    <span class="trend-label">S统计量:</span>
                    <span class="trend-value">${result.mk_statistic ? result.mk_statistic.toFixed(2) : 'N/A'}</span>
                    
                    <span class="trend-label">Sen斜率:</span>
                    <span class="trend-value">${result.sen_slope ? result.sen_slope.toFixed(6) : 'N/A'}</span>
                </div>
            </div>
        `;
    }

    renderChart(result, indicatorName) {
        const ctx = this.chartCtx;
        const canvas = this.chartCanvas;
        
        const width = canvas.width;
        const height = canvas.height;
        
        ctx.clearRect(0, 0, width, height);
        
        if (!result.time_points || !result.values) return;
        
        const timePoints = result.time_points;
        const values = result.values;
        
        if (!Array.isArray(timePoints) || !Array.isArray(values) || timePoints.length === 0) return;

        const padding = { top: 20, right: 20, bottom: 40, left: 50 };
        const chartWidth = width - padding.left - padding.right;
        const chartHeight = height - padding.top - padding.bottom;

        const validValues = values.filter(v => v !== null && v !== undefined);
        if (validValues.length === 0) return;

        const minVal = Math.min(...validValues);
        const maxVal = Math.max(...validValues);
        const valRange = maxVal - minVal || 1;

        const minYear = Math.min(...timePoints);
        const maxYear = Math.max(...timePoints);
        const yearRange = maxYear - minYear || 1;

        ctx.strokeStyle = '#e0e0e0';
        ctx.lineWidth = 1;
        
        for (let i = 0; i <= 4; i++) {
            const y = padding.top + (chartHeight / 4) * i;
            ctx.beginPath();
            ctx.moveTo(padding.left, y);
            ctx.lineTo(width - padding.right, y);
            ctx.stroke();
        }

        ctx.strokeStyle = '#333';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(padding.left, padding.top);
        ctx.lineTo(padding.left, height - padding.bottom);
        ctx.stroke();
        
        ctx.beginPath();
        ctx.moveTo(padding.left, height - padding.bottom);
        ctx.lineTo(width - padding.right, height - padding.bottom);
        ctx.stroke();

        ctx.fillStyle = '#666';
        ctx.font = '10px sans-serif';
        ctx.textAlign = 'right';
        
        for (let i = 0; i <= 4; i++) {
            const val = maxVal - (valRange / 4) * i;
            const y = padding.top + (chartHeight / 4) * i;
            ctx.fillText(val.toFixed(2), padding.left - 5, y + 3);
        }

        ctx.textAlign = 'center';
        const labelStep = Math.max(1, Math.floor(timePoints.length / 5));
        for (let i = 0; i < timePoints.length; i += labelStep) {
            const x = padding.left + (chartWidth / (timePoints.length - 1)) * i;
            const year = timePoints[i];
            const yearStr = year > 0 ? `${year}年` : `前${Math.abs(year)}年`;
            ctx.save();
            ctx.translate(x, height - padding.bottom + 15);
            ctx.rotate(-30 * Math.PI / 180);
            ctx.fillText(yearStr, 0, 0);
            ctx.restore();
        }

        ctx.strokeStyle = '#3498db';
        ctx.lineWidth = 2;
        ctx.beginPath();
        
        let started = false;
        for (let i = 0; i < values.length; i++) {
            if (values[i] === null || values[i] === undefined) continue;
            
            const x = padding.left + (chartWidth / (timePoints.length - 1)) * i;
            const y = padding.top + chartHeight - ((values[i] - minVal) / valRange) * chartHeight;
            
            if (!started) {
                ctx.moveTo(x, y);
                started = true;
            } else {
                ctx.lineTo(x, y);
            }
        }
        ctx.stroke();

        ctx.fillStyle = '#e74c3c';
        for (let i = 0; i < values.length; i++) {
            if (values[i] === null || values[i] === undefined) continue;
            
            const x = padding.left + (chartWidth / (timePoints.length - 1)) * i;
            const y = padding.top + chartHeight - ((values[i] - minVal) / valRange) * chartHeight;
            
            ctx.beginPath();
            ctx.arc(x, y, 4, 0, Math.PI * 2);
            ctx.fill();
        }

        ctx.fillStyle = '#333';
        ctx.font = 'bold 11px sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText(indicatorName + ' 时间序列', width / 2, 12);
    }
}
