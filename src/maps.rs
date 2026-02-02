use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MapRectangle {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub space_id: String,
}

#[derive(Debug, Clone)]
pub struct FloorMap {
    pub name: String,
    pub view_box_width: f64,
    pub view_box_height: f64,
    pub rectangles: Vec<MapRectangle>,
}

pub fn load_all_maps() -> Vec<FloorMap> {
    let data = match std::fs::read_to_string("assets/map_structure.json") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let map_list = match json["mapsStructure"]["maps"].as_array() {
        Some(m) => m,
        None => return Vec::new(),
    };

    let mut maps = Vec::new();

    for map_val in map_list {
        let name = map_val["name"].as_str().unwrap_or("").to_string();

        let vb_str = map_val["viewBox"].as_str().unwrap_or("0 0 100 100");
        let vb_parts: Vec<f64> = vb_str
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        let (vb_w, vb_h) = if vb_parts.len() >= 4 {
            (vb_parts[2], vb_parts[3])
        } else {
            (100.0, 100.0)
        };

        let mut rect_map: HashMap<String, MapRectangle> = HashMap::new();
        if let Some(rects) = map_val["dynamicRectangles"].as_array() {
            for r in rects {
                let space_id = r["spaceId"]
                    .as_i64()
                    .map(|n| n.to_string())
                    .or_else(|| r["spaceId"].as_str().map(String::from))
                    .unwrap_or_default();

                if space_id.is_empty() {
                    continue;
                }

                let x = r["x"].as_f64().unwrap_or(0.0);
                let y = r["y"].as_f64().unwrap_or(0.0);
                let w = r["w"].as_f64().unwrap_or(0.0);
                let h = r["h"].as_f64().unwrap_or(0.0);
                let area = w * h;

                let replace = rect_map
                    .get(&space_id)
                    .map(|existing| existing.w * existing.h < area)
                    .unwrap_or(true);

                if replace {
                    rect_map.insert(
                        space_id.clone(),
                        MapRectangle {
                            x,
                            y,
                            w,
                            h,
                            space_id,
                        },
                    );
                }
            }
        }

        let mut rects: Vec<MapRectangle> = rect_map.into_values().collect();
        rects.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });

        maps.push(FloorMap {
            name,
            view_box_width: vb_w,
            view_box_height: vb_h,
            rectangles: rects,
        });
    }

    maps
}

pub fn get_floor_map<'a>(maps: &'a [FloorMap], location_name: &str) -> Option<&'a FloorMap> {
    let normalized = location_name.replace('-', " ");
    maps.iter().find(|m| m.name == normalized)
}
