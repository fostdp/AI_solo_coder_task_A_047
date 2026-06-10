use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;

use crate::models::*;
use crate::errors::AppError;
use crate::spatial_syntax::*;
use crate::fractal::*;
use crate::mann_kendall::*;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "古代城市遗址空间结构复原与形态演化分析系统 API 运行正常",
        "version": "1.0.0"
    }))
}

pub async fn get_dynasties(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, name, start_year, end_year, period, description, created_at
        FROM dynasties
        ORDER BY start_year ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    let dynasties: Vec<Dynasty> = rows
        .into_iter()
        .map(|row| Dynasty {
            id: row.id,
            name: row.name,
            start_year: row.start_year,
            end_year: row.end_year,
            period: row.period,
            description: row.description,
            created_at: row.created_at.map(|dt| dt.and_utc()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(dynasties)))
}

pub async fn get_city_sites(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT cs.id, cs.name, cs.dynasty_id, cs.location, cs.center_longitude,
               cs.center_latitude, cs.estimated_population, cs.area_sq_km,
               cs.description, cs.archaeological_notes,
               ST_AsGeoJSON(cs.geom)::jsonb as geom_json,
               cs.created_at, cs.updated_at,
               d.name as dynasty_name
        FROM city_sites cs
        JOIN dynasties d ON cs.dynasty_id = d.id
        ORDER BY d.start_year ASC, cs.name ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    let sites: Vec<CitySite> = rows
        .into_iter()
        .map(|row| CitySite {
            id: row.id,
            name: row.name,
            dynasty_id: row.dynasty_id,
            location: row.location,
            center_longitude: row.center_longitude,
            center_latitude: row.center_latitude,
            estimated_population: row.estimated_population,
            area_sq_km: row.area_sq_km,
            description: row.description,
            archaeological_notes: row.archaeological_notes,
            geom: row.geom_json.map(|j| serde_json::from_value(j).unwrap_or(json!({}))),
            created_at: row.created_at.map(|dt| dt.and_utc()),
            updated_at: row.updated_at.map(|dt| dt.and_utc()),
            dynasty_name: Some(row.dynasty_name),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(sites)))
}

pub async fn get_city_site_by_id(
    pool: web::Data<PgPool>,
    site_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT cs.id, cs.name, cs.dynasty_id, cs.location, cs.center_longitude,
               cs.center_latitude, cs.estimated_population, cs.area_sq_km,
               cs.description, cs.archaeological_notes,
               ST_AsGeoJSON(cs.geom)::jsonb as geom_json,
               cs.created_at, cs.updated_at,
               d.name as dynasty_name
        FROM city_sites cs
        JOIN dynasties d ON cs.dynasty_id = d.id
        WHERE cs.id = $1
        "#,
        site_id.into_inner()
    )
    .fetch_optional(pool.get_ref())
    .await?;

    match row {
        Some(row) => {
            let site = CitySite {
                id: row.id,
                name: row.name,
                dynasty_id: row.dynasty_id,
                location: row.location,
                center_longitude: row.center_longitude,
                center_latitude: row.center_latitude,
                estimated_population: row.estimated_population,
                area_sq_km: row.area_sq_km,
                description: row.description,
                archaeological_notes: row.archaeological_notes,
                geom: row.geom_json.map(|j| serde_json::from_value(j).unwrap_or(json!({}))),
                created_at: row.created_at.map(|dt| dt.and_utc()),
                updated_at: row.updated_at.map(|dt| dt.and_utc()),
                dynasty_name: Some(row.dynasty_name),
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(site)))
        }
        None => Err(AppError::NotFound("City site not found".to_string())),
    }
}

pub async fn get_sites_by_dynasty(
    pool: web::Data<PgPool>,
    dynasty_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT cs.id, cs.name, cs.dynasty_id, cs.location, cs.center_longitude,
               cs.center_latitude, cs.estimated_population, cs.area_sq_km,
               cs.description, cs.archaeological_notes,
               ST_AsGeoJSON(cs.geom)::jsonb as geom_json,
               cs.created_at, cs.updated_at,
               d.name as dynasty_name
        FROM city_sites cs
        JOIN dynasties d ON cs.dynasty_id = d.id
        WHERE cs.dynasty_id = $1
        ORDER BY cs.name ASC
        "#,
        dynasty_id.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await?;

    let sites: Vec<CitySite> = rows
        .into_iter()
        .map(|row| CitySite {
            id: row.id,
            name: row.name,
            dynasty_id: row.dynasty_id,
            location: row.location,
            center_longitude: row.center_longitude,
            center_latitude: row.center_latitude,
            estimated_population: row.estimated_population,
            area_sq_km: row.area_sq_km,
            description: row.description,
            archaeological_notes: row.archaeological_notes,
            geom: row.geom_json.map(|j| serde_json::from_value(j).unwrap_or(json!({}))),
            created_at: row.created_at.map(|dt| dt.and_utc()),
            updated_at: row.updated_at.map(|dt| dt.and_utc()),
            dynasty_name: Some(row.dynasty_name),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(sites)))
}

pub async fn get_functional_zones(
    pool: web::Data<PgPool>,
    site_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, city_site_id, zone_type, name, description,
               archaeological_findings, functional_inference, confidence_level,
               ST_AsGeoJSON(geom)::jsonb as geom_json, created_at
        FROM functional_zones
        WHERE city_site_id = $1
        ORDER BY zone_type, name
        "#,
        site_id.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await?;

    let zones: Vec<FunctionalZone> = rows
        .into_iter()
        .map(|row| FunctionalZone {
            id: row.id,
            city_site_id: row.city_site_id,
            zone_type: row.zone_type,
            name: row.name,
            description: row.description,
            archaeological_findings: row.archaeological_findings,
            functional_inference: row.functional_inference,
            confidence_level: row.confidence_level,
            geom: row.geom_json.map(|j| serde_json::from_value(j).unwrap_or(json!({}))),
            created_at: row.created_at.map(|dt| dt.and_utc()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(zones)))
}

pub async fn get_roads(
    pool: web::Data<PgPool>,
    site_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, city_site_id, road_name, road_type, width, description,
               ST_AsGeoJSON(geom)::jsonb as geom_json, created_at
        FROM roads
        WHERE city_site_id = $1
        ORDER BY road_name
        "#,
        site_id.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await?;

    let roads: Vec<Road> = rows
        .into_iter()
        .map(|row| Road {
            id: row.id,
            city_site_id: row.city_site_id,
            road_name: row.road_name,
            road_type: row.road_type,
            width: row.width,
            description: row.description,
            geom: row.geom_json.map(|j| serde_json::from_value(j).unwrap_or(json!({}))),
            created_at: row.created_at.map(|dt| dt.and_utc()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(roads)))
}

pub async fn get_buildings(
    pool: web::Data<PgPool>,
    site_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, city_site_id, building_type, name, area_sq_m, rooms_count,
               description, archaeological_findings,
               ST_AsGeoJSON(geom)::jsonb as geom_json, created_at
        FROM building_foundations
        WHERE city_site_id = $1
        ORDER BY building_type, name
        "#,
        site_id.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await?;

    let buildings: Vec<BuildingFoundation> = rows
        .into_iter()
        .map(|row| BuildingFoundation {
            id: row.id,
            city_site_id: row.city_site_id,
            building_type: row.building_type,
            name: row.name,
            area_sq_m: row.area_sq_m,
            rooms_count: row.rooms_count,
            description: row.description,
            archaeological_findings: row.archaeological_findings,
            geom: row.geom_json.map(|j| serde_json::from_value(j).unwrap_or(json!({}))),
            created_at: row.created_at.map(|dt| dt.and_utc()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(buildings)))
}

pub async fn get_population(
    pool: web::Data<PgPool>,
    site_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, city_site_id, estimate_year, population_min, population_max,
               population_mean, estimation_method, source, confidence_level, created_at
        FROM population_estimates
        WHERE city_site_id = $1
        ORDER BY estimate_year
        "#,
        site_id.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await?;

    let estimates: Vec<PopulationEstimate> = rows
        .into_iter()
        .map(|row| PopulationEstimate {
            id: row.id,
            city_site_id: row.city_site_id,
            estimate_year: row.estimate_year,
            population_min: row.population_min,
            population_max: row.population_max,
            population_mean: row.population_mean,
            estimation_method: row.estimation_method,
            source: row.source,
            confidence_level: row.confidence_level,
            created_at: row.created_at.map(|dt| dt.and_utc()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(estimates)))
}

pub async fn get_morphology(
    pool: web::Data<PgPool>,
    site_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT id, city_site_id, analysis_date, integration_global, integration_local,
               choice_global, choice_local, mean_depth, total_depth, connectivity,
               boundary_fractal_dimension, road_network_fractal_dimension,
               compactness_index, elongation_ratio, road_density, intersection_density,
               functional_diversity, functional_mixing,
               boundary_fd_quality, road_fd_quality,
               boundary_fd_confidence_lower, boundary_fd_confidence_upper,
               road_fd_confidence_lower, road_fd_confidence_upper,
               notes, created_at
        FROM morphology_analyses
        WHERE city_site_id = $1
        ORDER BY analysis_date DESC
        LIMIT 1
        "#,
        site_id.into_inner()
    )
    .fetch_optional(pool.get_ref())
    .await?;

    match row {
        Some(row) => {
            let analysis = MorphologyAnalysis {
                id: row.id,
                city_site_id: row.city_site_id,
                analysis_date: row.analysis_date.map(|dt| dt.and_utc()),
                integration_global: row.integration_global,
                integration_local: row.integration_local,
                choice_global: row.choice_global,
                choice_local: row.choice_local,
                mean_depth: row.mean_depth,
                total_depth: row.total_depth,
                connectivity: row.connectivity,
                boundary_fractal_dimension: row.boundary_fractal_dimension,
                road_network_fractal_dimension: row.road_network_fractal_dimension,
                compactness_index: row.compactness_index,
                elongation_ratio: row.elongation_ratio,
                road_density: row.road_density,
                intersection_density: row.intersection_density,
                functional_diversity: row.functional_diversity,
                functional_mixing: row.functional_mixing,
                boundary_fd_quality: row.boundary_fd_quality,
                road_fd_quality: row.road_fd_quality,
                boundary_fd_confidence_lower: row.boundary_fd_confidence_lower,
                boundary_fd_confidence_upper: row.boundary_fd_confidence_upper,
                road_fd_confidence_lower: row.road_fd_confidence_lower,
                road_fd_confidence_upper: row.road_fd_confidence_upper,
                notes: row.notes,
                created_at: row.created_at.map(|dt| dt.and_utc()),
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(analysis)))
        }
        None => Err(AppError::NotFound("Morphology analysis not found".to_string())),
    }
}

pub async fn analyze_morphology(
    pool: web::Data<PgPool>,
    site_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let site_id = site_id.into_inner();

    let city_row = sqlx::query!(
        r#"
        SELECT ST_AsGeoJSON(geom)::jsonb as geom_json, area_sq_km
        FROM city_sites WHERE id = $1
        "#,
        site_id
    )
    .fetch_optional(pool.get_ref())
    .await?;

    let city_row = match city_row {
        Some(r) => r,
        None => return Err(AppError::NotFound("City site not found".to_string())),
    };

    let roads_rows = sqlx::query!(
        r#"
        SELECT id, ST_AsGeoJSON(geom)::jsonb as geom_json
        FROM roads WHERE city_site_id = $1
        "#,
        site_id
    )
    .fetch_all(pool.get_ref())
    .await?;

    let zones_rows = sqlx::query!(
        r#"
        SELECT id, zone_type, ST_Area(geom::geography) as area
        FROM functional_zones WHERE city_site_id = $1
        "#,
        site_id
    )
    .fetch_all(pool.get_ref())
    .await?;

    let boundary_points = extract_polygon_points(&city_row.geom_json);
    let boundary_fd_result = boundary_fractal_dimension_robust(&boundary_points, 8);
    let boundary_fd = boundary_fd_result.weighted_average;

    let road_segments = extract_road_segments(&roads_rows);
    let road_fd_result = network_fractal_dimension_robust(&road_segments, 8);
    let road_fd = road_fd_result.weighted_average;

    let (min_x, max_x, min_y, max_y) = bounding_box_points(&boundary_points);
    let area = polygon_area(&boundary_points);
    let perimeter = polygon_perimeter(&boundary_points);
    let compactness = compactness_index(area, perimeter);
    let elongation = elongation_ratio(min_x, max_x, min_y, max_y);

    let mut axial_lines = Vec::with_capacity(road_segments.len());
    for (i, segment) in road_segments.iter().enumerate() {
        let line = AxialLine::new(i, segment.0.0, segment.0.1, segment.1.0, segment.1.1);
        axial_lines.push(line);
    }

    let graph = if axial_lines.len() > 100 {
        build_axial_graph_optimized(&axial_lines)
    } else {
        let mut g = SpatialGraph::with_capacity(axial_lines.len());
        for line in &axial_lines {
            let (mx, my) = line.midpoint();
            g.add_node(mx, my);
        }
        for i in 0..axial_lines.len() {
            for j in (i + 1)..axial_lines.len() {
                if lines_intersect(
                    axial_lines[i].start_x, axial_lines[i].start_y,
                    axial_lines[i].end_x, axial_lines[i].end_y,
                    axial_lines[j].start_x, axial_lines[j].start_y,
                    axial_lines[j].end_x, axial_lines[j].end_y,
                ) {
                    g.add_edge(i, j);
                }
            }
        }
        g.optimize_memory();
        g
    };

    let avg_integration = graph.average_integration_global();
    let avg_connectivity = graph.average_connectivity();
    let avg_mean_depth = graph.average_mean_depth();
    let avg_total_depth = graph.average_total_depth();
    let local_integration = if graph.node_count() > 0 {
        graph.integration_local(0, 3)
    } else {
        0.0
    };

    let all_choice = if graph.node_count() > 500 {
        graph.choice_global_brandes_chunked(32)
    } else {
        graph.choice_global_brandes()
    };
    let avg_choice = if all_choice.is_empty() {
        0.0
    } else {
        all_choice.iter().sum::<f64>() / all_choice.len() as f64
    };
    let local_choice = if graph.node_count() > 0 && !all_choice.is_empty() {
        *all_choice.get(0).unwrap_or(&0.0)
    } else {
        0.0
    };

    let per_road_metrics = if graph.node_count() > 0 && axial_lines.len() >= graph.node_count() {
        graph.compute_all_metrics(3)
    } else {
        Vec::new()
    };

    let road_density = if area > 0.0 {
        road_segments.len() as f64 / area * 1_000_000.0
    } else {
        0.0
    };

    let intersection_density = if area > 0.0 {
        graph.node_count() as f64 / area * 1_000_000.0
    } else {
        0.0
    };

    let mut zone_type_areas = std::collections::HashMap::new();
    for row in &zones_rows {
        let area = row.area.unwrap_or(0.0);
        let entry = zone_type_areas.entry(row.zone_type.clone()).or_insert(0.0);
        *entry += area;
    }

    let zone_counts: Vec<usize> = zone_type_areas.values().map(|_| 1).collect();
    let functional_diversity = shannon_diversity(&zone_counts);

    let zone_areas: Vec<f64> = zone_type_areas.values().copied().collect();
    let functional_mixing = functional_mixing_index(&zone_areas);

    if !per_road_metrics.is_empty() {
        for (i, metrics) in per_road_metrics.iter().enumerate() {
            if i < roads_rows.len() {
                let road_id = roads_rows[i].id;
                sqlx::query!(
                    r#"
                    INSERT INTO road_syntax_results
                    (road_id, city_site_id, integration, choice, depth, connectivity, control)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (road_id) DO UPDATE
                    SET integration = EXCLUDED.integration,
                        choice = EXCLUDED.choice,
                        depth = EXCLUDED.depth,
                        connectivity = EXCLUDED.connectivity,
                        control = EXCLUDED.control
                    "#,
                    road_id,
                    site_id,
                    metrics.integration,
                    metrics.choice,
                    metrics.depth,
                    metrics.connectivity,
                    metrics.control
                )
                .execute(pool.get_ref())
                .await?;
            }
        }
    } else {
        for (i, _line) in axial_lines.iter().enumerate() {
            if i < roads_rows.len() {
                let road_id = roads_rows[i].id;
                let integration = graph.integration_global(i);
                let choice = all_choice.get(i).copied().unwrap_or(0.0);
                let depth = graph.mean_depth(i);
                let connectivity = graph.connectivity(i) as i32;
                let control = graph.control(i);

                sqlx::query!(
                    r#"
                    INSERT INTO road_syntax_results
                    (road_id, city_site_id, integration, choice, depth, connectivity, control)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    ON CONFLICT (road_id) DO UPDATE
                    SET integration = EXCLUDED.integration,
                        choice = EXCLUDED.choice,
                        depth = EXCLUDED.depth,
                        connectivity = EXCLUDED.connectivity,
                        control = EXCLUDED.control
                    "#,
                    road_id,
                    site_id,
                    integration,
                    choice,
                    depth,
                    connectivity,
                    control
                )
                .execute(pool.get_ref())
                .await?;
            }
        }
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO morphology_analyses
        (city_site_id, integration_global, integration_local, choice_global, choice_local,
         mean_depth, total_depth, connectivity, boundary_fractal_dimension,
         road_network_fractal_dimension, compactness_index, elongation_ratio,
         road_density, intersection_density, functional_diversity, functional_mixing,
         boundary_fd_quality, road_fd_quality,
         boundary_fd_confidence_lower, boundary_fd_confidence_upper,
         road_fd_confidence_lower, road_fd_confidence_upper, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22, $23)
        RETURNING id, city_site_id, analysis_date, integration_global, integration_local,
                  choice_global, choice_local, mean_depth, total_depth, connectivity,
                  boundary_fractal_dimension, road_network_fractal_dimension,
                  compactness_index, elongation_ratio, road_density, intersection_density,
                  functional_diversity, functional_mixing,
                  boundary_fd_quality, road_fd_quality,
                  boundary_fd_confidence_lower, boundary_fd_confidence_upper,
                  road_fd_confidence_lower, road_fd_confidence_upper,
                  notes, created_at
        "#,
        site_id,
        avg_integration,
        local_integration,
        avg_choice,
        local_choice,
        avg_mean_depth,
        avg_total_depth,
        avg_connectivity,
        boundary_fd,
        road_fd,
        compactness,
        elongation,
        road_density,
        intersection_density,
        functional_diversity,
        functional_mixing,
        Some(boundary_fd_result.overall_quality),
        Some(road_fd_result.overall_quality),
        Some(boundary_fd_result.confidence_lower),
        Some(boundary_fd_result.confidence_upper),
        Some(road_fd_result.confidence_lower),
        Some(road_fd_result.confidence_upper),
        Some("自动生成的形态分析结果".to_string())
    )
    .fetch_one(pool.get_ref())
    .await?;

    let analysis = MorphologyAnalysis {
        id: result.id,
        city_site_id: result.city_site_id,
        analysis_date: result.analysis_date.map(|dt| dt.and_utc()),
        integration_global: result.integration_global,
        integration_local: result.integration_local,
        choice_global: result.choice_global,
        choice_local: result.choice_local,
        mean_depth: result.mean_depth,
        total_depth: result.total_depth,
        connectivity: result.connectivity,
        boundary_fractal_dimension: result.boundary_fractal_dimension,
        road_network_fractal_dimension: result.road_network_fractal_dimension,
        compactness_index: result.compactness_index,
        elongation_ratio: result.elongation_ratio,
        road_density: result.road_density,
        intersection_density: result.intersection_density,
        functional_diversity: result.functional_diversity,
        functional_mixing: result.functional_mixing,
        boundary_fd_quality: result.boundary_fd_quality,
        road_fd_quality: result.road_fd_quality,
        boundary_fd_confidence_lower: result.boundary_fd_confidence_lower,
        boundary_fd_confidence_upper: result.boundary_fd_confidence_upper,
        road_fd_confidence_lower: result.road_fd_confidence_lower,
        road_fd_confidence_upper: result.road_fd_confidence_upper,
        notes: result.notes,
        created_at: result.created_at.map(|dt| dt.and_utc()),
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(analysis)))
}

pub async fn get_road_syntax(
    pool: web::Data<PgPool>,
    site_id: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, road_id, city_site_id, integration, choice, depth,
               connectivity, control, created_at
        FROM road_syntax_results
        WHERE city_site_id = $1
        ORDER BY integration DESC
        "#,
        site_id.into_inner()
    )
    .fetch_all(pool.get_ref())
    .await?;

    let results: Vec<RoadSyntaxResult> = rows
        .into_iter()
        .map(|row| RoadSyntaxResult {
            id: row.id,
            road_id: row.road_id,
            city_site_id: row.city_site_id,
            integration: row.integration,
            choice: row.choice,
            depth: row.depth,
            connectivity: row.connectivity,
            control: row.control,
            created_at: row.created_at.map(|dt| dt.and_utc()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(results)))
}

pub async fn analyze_trends(
    pool: web::Data<PgPool>,
    req: web::Json<TrendAnalysisRequest>,
) -> Result<HttpResponse, AppError> {
    let indicator = &req.indicator;

    let rows = sqlx::query!(
        r#"
        SELECT d.start_year, d.name as dynasty_name,
               CASE $1
                   WHEN 'integration_global' THEN ma.integration_global
                   WHEN 'integration_local' THEN ma.integration_local
                   WHEN 'choice_global' THEN ma.choice_global
                   WHEN 'choice_local' THEN ma.choice_local
                   WHEN 'mean_depth' THEN ma.mean_depth
                   WHEN 'connectivity' THEN ma.connectivity
                   WHEN 'boundary_fractal_dimension' THEN ma.boundary_fractal_dimension
                   WHEN 'road_network_fractal_dimension' THEN ma.road_network_fractal_dimension
                   WHEN 'compactness_index' THEN ma.compactness_index
                   WHEN 'elongation_ratio' THEN ma.elongation_ratio
                   WHEN 'road_density' THEN ma.road_density
                   WHEN 'intersection_density' THEN ma.intersection_density
                   WHEN 'functional_diversity' THEN ma.functional_diversity
                   WHEN 'functional_mixing' THEN ma.functional_mixing
                   ELSE ma.integration_global
               END as value
        FROM morphology_analyses ma
        JOIN city_sites cs ON ma.city_site_id = cs.id
        JOIN dynasties d ON cs.dynasty_id = d.id
        ORDER BY d.start_year ASC
        "#,
        indicator
    )
    .fetch_all(pool.get_ref())
    .await?;

    let values: Vec<f64> = rows
        .iter()
        .filter_map(|r| r.value)
        .collect();

    let years: Vec<i32> = rows
        .iter()
        .map(|r| r.start_year)
        .collect();

    let dynasty_names: Vec<String> = rows
        .iter()
        .map(|r| r.dynasty_name.clone())
        .collect();

    let mk_result = mann_kendall_test(&values, 0.05);

    let time_points_json = json!(years);
    let values_json = json!(values);

    let result = sqlx::query!(
        r#"
        INSERT INTO evolution_trends
        (analysis_name, indicator_name, mk_statistic, mk_p_value, mk_z_score,
         sen_slope, trend_direction, trend_significance, time_points, values, description)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10::jsonb, $11)
        RETURNING id, analysis_name, indicator_name, mk_statistic, mk_p_value, mk_z_score,
                  sen_slope, trend_direction, trend_significance,
                  time_points, values, description, created_at
        "#,
        format!("{} 演化趋势分析", indicator),
        indicator,
        mk_result.s,
        mk_result.p_value,
        mk_result.z_score,
        mk_result.sen_slope,
        mk_result.trend_direction,
        mk_result.trend_significance,
        time_points_json,
        values_json,
        Some(format!(
            "基于Mann-Kendall检验的{}演化趋势分析，涉及{}个朝代：{}",
            indicator,
            dynasty_names.len(),
            dynasty_names.join(", ")
        ))
    )
    .fetch_one(pool.get_ref())
    .await?;

    let trend = EvolutionTrend {
        id: result.id,
        analysis_name: result.analysis_name,
        indicator_name: result.indicator_name,
        mk_statistic: result.mk_statistic,
        mk_p_value: result.mk_p_value,
        mk_z_score: result.mk_z_score,
        sen_slope: result.sen_slope,
        trend_direction: result.trend_direction,
        trend_significance: result.trend_significance,
        time_points: result.time_points.map(|j| serde_json::from_value(j).unwrap_or(json!([]))),
        values: result.values.map(|j| serde_json::from_value(j).unwrap_or(json!([]))),
        description: result.description,
        created_at: result.created_at.map(|dt| dt.and_utc()),
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(trend)))
}

pub async fn get_trends(pool: web::Data<PgPool>) -> Result<HttpResponse, AppError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, analysis_name, indicator_name, mk_statistic, mk_p_value, mk_z_score,
               sen_slope, trend_direction, trend_significance,
               time_points, values, description, created_at
        FROM evolution_trends
        ORDER BY created_at DESC
        LIMIT 20
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    let trends: Vec<EvolutionTrend> = rows
        .into_iter()
        .map(|row| EvolutionTrend {
            id: row.id,
            analysis_name: row.analysis_name,
            indicator_name: row.indicator_name,
            mk_statistic: row.mk_statistic,
            mk_p_value: row.mk_p_value,
            mk_z_score: row.mk_z_score,
            sen_slope: row.sen_slope,
            trend_direction: row.trend_direction,
            trend_significance: row.trend_significance,
            time_points: row.time_points.map(|j| serde_json::from_value(j).unwrap_or(json!([]))),
            values: row.values.map(|j| serde_json::from_value(j).unwrap_or(json!([]))),
            description: row.description,
            created_at: row.created_at.map(|dt| dt.and_utc()),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(trends)))
}

pub async fn compare_sites(
    pool: web::Data<PgPool>,
    req: web::Json<CompareRequest>,
) -> Result<HttpResponse, AppError> {
    let mut comparisons = Vec::new();

    for &site_id in &req.site_ids {
        let site_row = sqlx::query!(
            r#"
            SELECT cs.id, cs.name, d.name as dynasty_name, d.start_year,
                   cs.estimated_population, cs.area_sq_km,
                   ST_AsGeoJSON(cs.geom)::jsonb as geom_json
            FROM city_sites cs
            JOIN dynasties d ON cs.dynasty_id = d.id
            WHERE cs.id = $1
            "#,
            site_id
        )
        .fetch_optional(pool.get_ref())
        .await?;

        if let Some(site) = site_row {
            let morph_row = sqlx::query!(
                r#"
                SELECT * FROM morphology_analyses
                WHERE city_site_id = $1
                ORDER BY analysis_date DESC
                LIMIT 1
                "#,
                site_id
            )
            .fetch_optional(pool.get_ref())
            .await?;

            let site_data = json!({
                "id": site.id,
                "name": site.name,
                "dynasty": site.dynasty_name,
                "start_year": site.start_year,
                "population": site.estimated_population,
                "area_sq_km": site.area_sq_km,
                "morphology": morph_row.map(|m| {
                    json!({
                        "integration_global": m.integration_global,
                        "choice_global": m.choice_global,
                        "boundary_fractal_dimension": m.boundary_fractal_dimension,
                        "road_network_fractal_dimension": m.road_network_fractal_dimension,
                        "compactness_index": m.compactness_index,
                        "elongation_ratio": m.elongation_ratio,
                        "functional_diversity": m.functional_diversity
                    })
                })
            });

            comparisons.push(site_data);
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(comparisons)))
}

fn extract_polygon_points(geom_json: &Option<serde_json::Value>) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    if let Some(geom) = geom_json {
        if let Some(coords) = geom.get("coordinates") {
            if let Some(ring) = coords.get(0) {
                if let Some(ring_arr) = ring.as_array() {
                    for p in ring_arr {
                        if let Some(arr) = p.as_array() {
                            if arr.len() >= 2 {
                                let x = arr[0].as_f64().unwrap_or(0.0);
                                let y = arr[1].as_f64().unwrap_or(0.0);
                                points.push((x, y));
                            }
                        }
                    }
                }
            }
        }
    }
    points
}

fn extract_road_segments(
    roads: &[sqlx::postgres::PgRow],
) -> Vec<((f64, f64), (f64, f64))> {
    let mut segments = Vec::new();

    for row in roads {
        let geom_json: Option<&serde_json::Value> = row
            .try_get("geom_json")
            .ok()
            .flatten();

        if let Some(geom) = geom_json {
            if let Some(coords) = geom.get("coordinates") {
                if let Some(coords_arr) = coords.as_array() {
                    for i in 0..coords_arr.len().saturating_sub(1) {
                        if let (Some(p1), Some(p2)) = (coords_arr[i].as_array(), coords_arr[i + 1].as_array()) {
                            if p1.len() >= 2 && p2.len() >= 2 {
                                let x1 = p1[0].as_f64().unwrap_or(0.0);
                                let y1 = p1[1].as_f64().unwrap_or(0.0);
                                let x2 = p2[0].as_f64().unwrap_or(0.0);
                                let y2 = p2[1].as_f64().unwrap_or(0.0);
                                segments.push(((x1, y1), (x2, y2)));
                            }
                        }
                    }
                }
            }
        }
    }

    segments
}

fn bounding_box_points(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut min_x = points[0].0;
    let mut max_x = points[0].0;
    let mut min_y = points[0].1;
    let mut max_y = points[0].1;

    for &(x, y) in points {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    (min_x, max_x, min_y, max_y)
}
