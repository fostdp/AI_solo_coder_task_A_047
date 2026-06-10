# 古代城市遗址空间结构复原与形态演化分析系统

## 项目概述

本系统是一套完整的古代城市遗址空间结构复原与形态演化分析平台，集成了考古数据管理、空间形态分析、历史演化研究等功能。

## 技术栈

### 后端
- **语言**: Rust
- **Web框架**: Actix-web
- **数据库**: PostgreSQL + PostGIS
- **ORM**: SQLx
- **空间分析**: 自研空间句法 + 分形维数算法

### 前端
- **地图**: Leaflet.js
- **绘制**: Canvas + SVG
- **样式**: 原生 CSS3

### 数据分析
- **空间句法**: 整合度、选择度、深度、连接度
- **分形维数**: 盒计数法计算边界和路网分形维数
- **趋势检验**: Mann-Kendall 检验 + Sen 斜率估计

## 功能特性

### 1. 城市遗址可视化
- 城墙范围展示
- 道路网络绘制
- 功能区分布（不同颜色填充）
- 建筑基址标记（圆点标记）
- 历史地图叠加

### 2. 时间轴对比
- 朝代时间轴可拖动
- 自动播放功能
- 不同朝代城市形态对比

### 3. 空间形态量化分析
- **空间句法分析**:
  - 全局/局部整合度 (Integration)
  - 全局/局部选择度 (Choice)
  - 平均深度 (Mean Depth)
  - 连接度 (Connectivity)
  
- **分形维数**:
  - 城市边界分形维数
  - 道路网络分形维数
  
- **形态指标**:
  - 紧凑度指数
  - 延展率
  - 道路密度
  - 交叉口密度

### 4. 功能区分析
- 功能多样性指数 (Shannon)
- 功能混合度指数
- 考古发现展示
- 功能推断说明

### 5. 演化趋势分析
- Mann-Kendall 趋势检验
- Sen 斜率估计
- 时间序列图表
- 趋势显著性判定

## 项目结构

```
.
├── backend/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs            # 程序入口
│   │   ├── db.rs              # 数据库连接
│   │   ├── models.rs          # 数据模型
│   │   ├── handlers.rs        # API 处理器
│   │   ├── spatial_syntax.rs  # 空间句法算法
│   │   ├── fractal.rs         # 分形维数算法
│   │   ├── mann_kendall.rs    # Mann-Kendall检验
│   │   └── errors.rs          # 错误处理
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── .env.example
├── frontend/                   # 前端应用
│   ├── index.html
│   ├── css/
│   │   └── style.css
│   └── js/
│       ├── config.js
│       ├── api.js
│       ├── map.js
│       ├── timeline.js
│       ├── morphology.js
│       ├── trend.js
│       ├── compare.js
│       └── app.js
├── database/                   # 数据库脚本
│   └── init.sql               # 数据库初始化
├── scripts/                    # 工具脚本
│   └── generate_data.py       # 模拟数据生成
├── docker-compose.yml
├── nginx.conf
└── README.md
```

## 快速开始

### 方式一：Docker 部署（推荐）

1. 克隆项目
2. 启动服务：
```bash
docker-compose up -d
```

3. 生成模拟数据：
```bash
pip install psycopg2
python scripts/generate_data.py
```

4. 访问应用：
   - 前端: http://localhost
   - 后端 API: http://localhost:8080/api

### 方式二：本地开发

#### 1. 数据库设置

安装 PostgreSQL + PostGIS，然后：

```bash
createdb ancient_city
psql -d ancient_city -c "CREATE EXTENSION postgis;"
psql -d ancient_city -f database/init.sql
```

#### 2. 后端启动

```bash
cd backend
cp .env.example .env
# 修改 .env 中的数据库连接信息
cargo run --release
```

#### 3. 前端启动

```bash
cd frontend
# 使用任意静态文件服务器，例如：
python -m http.server 8081
# 或使用 npx serve
npx serve .
```

#### 4. 生成模拟数据

```bash
pip install psycopg2
python scripts/generate_data.py
```

## API 接口

### 朝代
- `GET /api/dynasties` - 获取所有朝代列表

### 城市遗址
- `GET /api/sites` - 获取所有城市遗址
- `GET /api/sites/{id}` - 获取单个遗址详情
- `GET /api/sites/dynasty/{dynasty_id}` - 按朝代获取遗址

### 功能区
- `GET /api/zones/{site_id}` - 获取遗址功能区

### 道路
- `GET /api/roads/{site_id}` - 获取遗址道路

### 建筑
- `GET /api/buildings/{site_id}` - 获取建筑基址

### 人口
- `GET /api/population/{site_id}` - 获取人口估算数据

### 形态分析
- `GET /api/morphology/{site_id}` - 获取形态分析结果
- `POST /api/morphology/analyze/{site_id}` - 执行形态分析
- `GET /api/syntax/roads/{site_id}` - 获取道路空间句法结果

### 趋势分析
- `GET /api/trends` - 获取历史趋势分析
- `POST /api/trends/analyze` - 执行趋势分析
  - 请求体: `{ "indicator": "integration_global" }`

### 对比
- `POST /api/compare` - 对比多个遗址
  - 请求体: `{ "site_ids": [1, 2, 3] }`

## 核心算法说明

### 空间句法
- **整合度 (Integration)**: 衡量空间单元在系统中的可达性
- **选择度 (Choice)**: 衡量空间单元在路径选择中的通过潜力
- **深度 (Depth)**: 从一个空间到所有其他空间的最短路径之和
- **连接度 (Connectivity)**: 直接连接的空间数量

### 分形维数
使用盒计数法 (Box-counting method) 计算：
- 城市边界分形维数：反映城市形态的复杂程度
- 道路网络分形维数：反映路网的空间填充能力

### Mann-Kendall 趋势检验
非参数统计检验方法，用于检测时间序列的单调趋势：
- S 统计量
- Z 分数
- P 值（显著性水平）
- Sen 斜率（趋势大小）

## 数据模拟

数据生成脚本 `scripts/generate_data.py` 可生成：
- 50 个古代城市遗址（从殷商到明清）
- 每个遗址包含城墙、道路、功能区、建筑基址
- 人口估算数据
- 历史地图元数据

## 许可证

MIT License
