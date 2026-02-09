use crate::{
    Coordinate, Error, Polygon,
    geometry::helpers::{aabb::AABB, vec2::Vec2, vec3::Vec3},
    triangle::Triangle,
};
use std::collections::HashMap;

/// A structure to store triangulation results with surface index.
#[derive(Debug)]
struct IndexedTriangle {
    surface_idx: usize,
    v: [Vec3; 3],
    aabb: AABB,
}

#[derive(Debug, Default)]
struct EdgeStats {
    pub forward: Vec<usize>,
    pub backward: Vec<usize>,
}

/// A completely closed solid with no gaps or holes.
///
/// # Guarantees
///
/// When created, this type guarantees:
/// - At least one surface exists
/// - Each surface is a valid Polygon
/// - All vertices are normalized with epsilon tolerance
/// - Every edge is shared by exactly two surfaces in opposite directions (manifold condition)
/// - Surface orientations are consistent
/// - No degenerate edges exist
/// - All surfaces are topologically connected
/// - Signed volume is non-zero
/// - No geometric intersections between surfaces from different faces
#[derive(Debug, Clone)]
pub struct Solid {
    surfaces: Vec<Polygon>,
    epsilon: f64,
}

impl Solid {
    /// Default epsilon value for geometric operations.
    pub const DEFAULT_EPSILON: f64 = 1.0e-12;

    /// Creates a new solid from a list of surfaces.
    pub fn new(surfaces: Vec<Polygon>) -> Result<Self, Error> {
        Self::new_with_epsilon(surfaces, Self::DEFAULT_EPSILON)
    }

    /// Creates a new solid with a custom epsilon value.
    pub fn new_with_epsilon(surfaces: Vec<Polygon>, epsilon: f64) -> Result<Self, Error> {
        if surfaces.is_empty() {
            return Err(Error::EmptySolid);
        }

        let surfaces = Self::normalize_vertices(surfaces, epsilon);

        let solid = Self { surfaces, epsilon };

        solid.validate_closed_manifold()?;
        solid.validate_connectivity()?;
        solid.validate_positive_volume()?;
        solid.validate_no_geometric_intersection()?;

        Ok(solid)
    }

    // Vertex normalization
    fn normalize_vertices(surfaces: Vec<Polygon>, epsilon: f64) -> Vec<Polygon> {
        let mut all_points: Vec<Coordinate> = Vec::new();
        let mut surface_ranges: Vec<(usize, usize)> = Vec::new();

        for surface in &surfaces {
            let points = surface.points();
            let start = all_points.len();
            for point in points.iter().take(points.len() - 1) {
                all_points.push(point.clone());
            }
            let end = all_points.len();
            surface_ranges.push((start, end));
        }

        let n = all_points.len();
        if n == 0 {
            return surfaces;
        }

        let mut parent: Vec<usize> = (0..n).collect();

        fn find(parent: &mut [usize], i: usize) -> usize {
            if parent[i] != i {
                parent[i] = find(parent, parent[i]);
            }
            parent[i]
        }

        fn union(parent: &mut [usize], a: usize, b: usize) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                if ra < rb {
                    parent[rb] = ra;
                } else {
                    parent[ra] = rb;
                }
            }
        }

        let eps_sq = epsilon * epsilon;
        for i in 0..n {
            for j in (i + 1)..n {
                let pi = &all_points[i];
                let pj = &all_points[j];
                let dist_sq = (pi.as_longitude() - pj.as_longitude()).powi(2)
                    + (pi.as_latitude() - pj.as_latitude()).powi(2)
                    + (pi.as_altitude() - pj.as_altitude()).powi(2);

                if dist_sq <= eps_sq {
                    union(&mut parent, i, j);
                }
            }
        }

        let mut representative: HashMap<usize, Coordinate> = HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            representative
                .entry(root)
                .or_insert_with(|| all_points[root].clone());
        }

        let mut normalized_points: Vec<Coordinate> = Vec::with_capacity(n);
        for i in 0..n {
            let root = find(&mut parent, i);
            normalized_points.push(representative[&root].clone());
        }

        let mut new_surfaces = Vec::with_capacity(surfaces.len());
        for (idx, (start, end)) in surface_ranges.iter().enumerate() {
            let mut coords: Vec<Coordinate> = normalized_points[*start..*end].to_vec();
            if let Some(first) = coords.first() {
                coords.push(first.clone());
            }

            match Polygon::new_with_epsilon(coords, epsilon) {
                Ok(polygon) => new_surfaces.push(polygon),
                Err(_) => {
                    new_surfaces.push(surfaces[idx].clone());
                }
            }
        }

        new_surfaces
    }

    // Edge collection and validation
    fn coord_to_bits(c: &Coordinate) -> [u64; 3] {
        [
            c.as_longitude().to_bits(),
            c.as_latitude().to_bits(),
            c.as_altitude().to_bits(),
        ]
    }

    fn edge_key(a: [u64; 3], b: [u64; 3]) -> ([u64; 3], [u64; 3]) {
        if a < b { (a, b) } else { (b, a) }
    }

    fn is_forward(a: [u64; 3], b: [u64; 3]) -> bool {
        a < b
    }

    fn collect_edges(&self) -> HashMap<([u64; 3], [u64; 3]), EdgeStats> {
        let mut edge_map: HashMap<([u64; 3], [u64; 3]), EdgeStats> = HashMap::new();

        for (surface_idx, surface) in self.surfaces.iter().enumerate() {
            let points = surface.points();
            for i in 0..points.len() - 1 {
                let p1_bits = Self::coord_to_bits(&points[i]);
                let p2_bits = Self::coord_to_bits(&points[i + 1]);

                let key = Self::edge_key(p1_bits, p2_bits);
                let stats = edge_map.entry(key).or_default();

                if Self::is_forward(p1_bits, p2_bits) {
                    stats.forward.push(surface_idx);
                } else {
                    stats.backward.push(surface_idx);
                }
            }
        }

        edge_map
    }

    fn validate_closed_manifold(&self) -> Result<(), Error> {
        let edge_map = self.collect_edges();

        for ((a, b), stats) in &edge_map {
            if a == b {
                return Err(Error::DegenerateEdge(
                    *stats
                        .forward
                        .first()
                        .or(stats.backward.first())
                        .unwrap_or(&0),
                ));
            }
            if stats.forward.is_empty() || stats.backward.is_empty() {
                return Err(Error::OpenHoleDetected);
            }
            if stats.forward.len() > 1 || stats.backward.len() > 1 {
                return Err(Error::NonManifoldEdge);
            }

            // 正確に1回ずつ出現していることを確認
            if stats.forward.len() != 1 || stats.backward.len() != 1 {
                return Err(Error::InvalidEdgeTopology {
                    forward: stats.forward.len(),
                    backward: stats.backward.len(),
                });
            }
        }

        Ok(())
    }

    // Connectivity validation
    fn validate_connectivity(&self) -> Result<(), Error> {
        let n = self.surfaces.len();
        if n <= 1 {
            return Ok(());
        }

        let edge_map = self.collect_edges();
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (_, stats) in &edge_map {
            if let (Some(&f), Some(&b)) = (stats.forward.first(), stats.backward.first()) {
                if f != b {
                    adjacency[f].push(b);
                    adjacency[b].push(f);
                }
            }
        }

        let mut visited = vec![false; n];
        let mut queue = std::collections::VecDeque::new();
        visited[0] = true;
        queue.push_back(0);

        while let Some(current) = queue.pop_front() {
            for &neighbor in &adjacency[current] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }

        if visited.iter().any(|&v| !v) {
            return Err(Error::DisconnectedSolid);
        }

        Ok(())
    }

    // Signed volume validation
    fn validate_positive_volume(&self) -> Result<(), Error> {
        let triangles = self.triangulate()?;

        let mut volume = 0.0;
        for tri in &triangles {
            let coords = tri.points();
            let a = &coords[0];
            let b = &coords[1];
            let c = &coords[2];

            let ax = a.as_longitude();
            let ay = a.as_latitude();
            let az = a.as_altitude();
            let bx = b.as_longitude();
            let by = b.as_latitude();
            let bz = b.as_altitude();
            let cx = c.as_longitude();
            let cy = c.as_latitude();
            let cz = c.as_altitude();

            volume +=
                ax * (by * cz - bz * cy) + ay * (bz * cx - bx * cz) + az * (bx * cy - by * cx);
        }

        volume /= 6.0;

        if volume.abs() < self.epsilon {
            return Err(Error::DegenerateSolid);
        }

        Ok(())
    }

    // Geometric intersection validation

    /// Validates that triangles from different surfaces do not geometrically intersect.
    ///
    /// Steps:
    /// 1. Triangulate all surfaces and record which surface each triangle belongs to
    /// 2. Collect adjacent surface pairs (surfaces sharing edges)
    /// 3. Use AABB for fast filtering of all triangle pairs
    /// 4. For AABB overlapping pairs, perform Möller triangle intersection test
    /// 5. For adjacent surface triangles, exclude contact at shared vertices
    fn validate_no_geometric_intersection(&self) -> Result<(), Error> {
        // 1. 面ごとの三角形分割 + インデックス付与
        let indexed_triangles = self.build_indexed_triangles()?;

        if indexed_triangles.len() < 2 {
            return Ok(());
        }

        // 2. 隣接面ペアの収集（辺を共有する面の組み合わせ）
        let adjacent_pairs = self.collect_adjacent_surface_pairs();

        // 3. 全ペアの交差判定
        for i in 0..indexed_triangles.len() {
            for j in (i + 1)..indexed_triangles.len() {
                let tri_a = &indexed_triangles[i];
                let tri_b = &indexed_triangles[j];

                // Skip triangles from the same surface
                if tri_a.surface_idx == tri_b.surface_idx {
                    continue;
                }

                // AABB pre-filter
                if !tri_a.aabb.intersects(&tri_b.aabb, self.epsilon) {
                    continue;
                }

                // Adjacent surface triangles allow contact at shared elements
                let are_adjacent = adjacent_pairs
                    .contains(&Self::surface_pair(tri_a.surface_idx, tri_b.surface_idx));

                if Self::triangles_intersect_3d(&tri_a.v, &tri_b.v, are_adjacent, self.epsilon) {
                    return Err(Error::GeometricIntersection);
                }
            }
        }

        Ok(())
    }

    /// Normalizes surface pair key (smaller index first).
    fn surface_pair(a: usize, b: usize) -> (usize, usize) {
        if a < b { (a, b) } else { (b, a) }
    }

    /// Triangulates all surfaces and assigns surface indices.
    fn build_indexed_triangles(&self) -> Result<Vec<IndexedTriangle>, Error> {
        let mut result = Vec::new();

        for (surface_idx, surface) in self.surfaces.iter().enumerate() {
            let triangles = surface.triangulate()?;
            for tri in &triangles {
                let coords = tri.points();
                let v0 = Vec3::from_coord(&coords[0]);
                let v1 = Vec3::from_coord(&coords[1]);
                let v2 = Vec3::from_coord(&coords[2]);

                result.push(IndexedTriangle {
                    surface_idx,
                    aabb: AABB::from_triangle(v0, v1, v2),
                    v: [v0, v1, v2],
                });
            }
        }

        Ok(result)
    }

    /// Collects pairs of surfaces that share edges.
    fn collect_adjacent_surface_pairs(&self) -> std::collections::HashSet<(usize, usize)> {
        let edge_map = self.collect_edges();
        let mut pairs = std::collections::HashSet::new();

        for (_, stats) in &edge_map {
            if let (Some(&f), Some(&b)) = (stats.forward.first(), stats.backward.first()) {
                if f != b {
                    pairs.insert(Self::surface_pair(f, b));
                }
            }
        }

        pairs
    }

    // Triangle-triangle intersection test (3D)
    // Based on Möller's separating axis algorithm

    /// Determines if two triangles intersect (penetrate) in 3D space.
    ///
    /// When `are_adjacent` is true, contact at shared vertices or edges is not considered penetration.
    fn triangles_intersect_3d(
        tri_a: &[Vec3; 3],
        tri_b: &[Vec3; 3],
        are_adjacent: bool,
        epsilon: f64,
    ) -> bool {
        // For adjacent surfaces, count shared vertices
        if are_adjacent {
            let shared = Self::count_shared_vertices(tri_a, tri_b, epsilon);
            // 2 or more shared vertices → only shares edge (contact, not penetration)
            if shared >= 2 {
                return false;
            }
        }

        // --- Möller triangle intersection test ---
        // Calculate plane for each triangle and determine which side vertices are on

        // Plane of triangle A
        let e1_a = tri_a[1].sub(tri_a[0]);
        let e2_a = tri_a[2].sub(tri_a[0]);
        let n_a = e1_a.cross(e2_a);

        // Signed distance of triangle B's vertices to plane A
        let db0 = n_a.dot(tri_b[0].sub(tri_a[0]));
        let db1 = n_a.dot(tri_b[1].sub(tri_a[0]));
        let db2 = n_a.dot(tri_b[2].sub(tri_a[0]));

        // Round floating point errors
        let db0 = if db0.abs() < epsilon { 0.0 } else { db0 };
        let db1 = if db1.abs() < epsilon { 0.0 } else { db1 };
        let db2 = if db2.abs() < epsilon { 0.0 } else { db2 };

        // All vertices of triangle B are on the same side of plane A → no intersection
        if db0 > 0.0 && db1 > 0.0 && db2 > 0.0 {
            return false;
        }
        if db0 < 0.0 && db1 < 0.0 && db2 < 0.0 {
            return false;
        }

        // Plane of triangle B
        let e1_b = tri_b[1].sub(tri_b[0]);
        let e2_b = tri_b[2].sub(tri_b[0]);
        let n_b = e1_b.cross(e2_b);

        // Signed distance of triangle A's vertices to plane B
        let da0 = n_b.dot(tri_a[0].sub(tri_b[0]));
        let da1 = n_b.dot(tri_a[1].sub(tri_b[0]));
        let da2 = n_b.dot(tri_a[2].sub(tri_b[0]));

        let da0 = if da0.abs() < epsilon { 0.0 } else { da0 };
        let da1 = if da1.abs() < epsilon { 0.0 } else { da1 };
        let da2 = if da2.abs() < epsilon { 0.0 } else { da2 };

        // All vertices of triangle A are on the same side of plane B → no intersection
        if da0 > 0.0 && da1 > 0.0 && da2 > 0.0 {
            return false;
        }
        if da0 < 0.0 && da1 < 0.0 && da2 < 0.0 {
            return false;
        }

        // --- Handle coplanar case ---
        if db0 == 0.0 && db1 == 0.0 && db2 == 0.0 {
            return Self::coplanar_triangles_intersect(tri_a, tri_b, &n_a, are_adjacent, epsilon);
        }

        // --- Interval overlap test on intersection line ---
        // Calculate the interval each triangle creates on the intersection line of the two planes
        let cross_dir = n_a.cross(n_b);

        // Determine projection axis onto intersection line (axis with maximum component of cross_dir)
        let ax = cross_dir.x().abs();
        let ay = cross_dir.y().abs();
        let az = cross_dir.z().abs();

        let project = |v: Vec3| -> f64 {
            if ax >= ay && ax >= az {
                v.x()
            } else if ay >= ax && ay >= az {
                v.y()
            } else {
                v.z()
            }
        };

        // Interval of triangle A on intersection line
        let pa0 = project(tri_a[0]);
        let pa1 = project(tri_a[1]);
        let pa2 = project(tri_a[2]);
        let interval_a = Self::compute_intersection_interval(pa0, pa1, pa2, da0, da1, da2);

        // Interval of triangle B on intersection line
        let pb0 = project(tri_b[0]);
        let pb1 = project(tri_b[1]);
        let pb2 = project(tri_b[2]);
        let interval_b = Self::compute_intersection_interval(pb0, pb1, pb2, db0, db1, db2);

        let (interval_a, interval_b) = match (interval_a, interval_b) {
            (Some(a), Some(b)) => (a, b),
            _ => return false,
        };

        // Interval overlap test
        // For adjacent surfaces sharing a vertex, allow endpoint contact only
        if are_adjacent {
            let shared = Self::count_shared_vertices(tri_a, tri_b, epsilon);
            if shared >= 1 {
                // Endpoint contact only → only interior overlap is considered intersection
                let overlap_start = interval_a.0.max(interval_b.0);
                let overlap_end = interval_a.1.min(interval_b.1);
                let overlap_len = overlap_end - overlap_start;

                // Only intersection if there is actual overlap length
                return overlap_len > epsilon;
            }
        }

        // Intervals overlap → intersection
        interval_a.0 < interval_b.1 - epsilon && interval_b.0 < interval_a.1 - epsilon
    }

    /// Computes the interval [t_min, t_max] that a triangle creates on the intersection line.
    /// d0, d1, d2 are signed distances of each vertex from the plane.
    /// p0, p1, p2 are projection values of each vertex onto the intersection line.
    fn compute_intersection_interval(
        p0: f64,
        p1: f64,
        p2: f64,
        d0: f64,
        d1: f64,
        d2: f64,
    ) -> Option<(f64, f64)> {
        // Linear interpolation between vertices with different signs to find intersection
        let mut ts = Vec::with_capacity(2);

        // d0 and d1 have different signs
        if (d0 > 0.0 && d1 < 0.0) || (d0 < 0.0 && d1 > 0.0) {
            let t = p0 + (p1 - p0) * d0 / (d0 - d1);
            ts.push(t);
        }

        // d0 and d2 have different signs
        if (d0 > 0.0 && d2 < 0.0) || (d0 < 0.0 && d2 > 0.0) {
            let t = p0 + (p2 - p0) * d0 / (d0 - d2);
            ts.push(t);
        }

        // d1 and d2 have different signs
        if ts.len() < 2 && ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0)) {
            let t = p1 + (p2 - p1) * d1 / (d1 - d2);
            ts.push(t);
        }

        // Vertices on the plane
        if ts.len() < 2 && d0 == 0.0 {
            ts.push(p0);
        }
        if ts.len() < 2 && d1 == 0.0 {
            ts.push(p1);
        }
        if ts.len() < 2 && d2 == 0.0 {
            ts.push(p2);
        }

        if ts.len() < 2 {
            return None;
        }

        let t_min = ts[0].min(ts[1]);
        let t_max = ts[0].max(ts[1]);

        Some((t_min, t_max))
    }

    /// Counts the number of shared vertices between two triangles.
    fn count_shared_vertices(tri_a: &[Vec3; 3], tri_b: &[Vec3; 3], epsilon: f64) -> usize {
        let eps_sq = epsilon * epsilon;
        let mut count = 0;

        for a in tri_a {
            for b in tri_b {
                if a.sub(*b).length_sq() <= eps_sq {
                    count += 1;
                    break; // One match per vertex is sufficient
                }
            }
        }

        count
    }

    // Coplanar triangle intersection test (projected to 2D)

    /// Intersection test for two triangles on the same plane.
    /// Uses normal direction to project to 2D and tests edge intersections.
    fn coplanar_triangles_intersect(
        tri_a: &[Vec3; 3],
        tri_b: &[Vec3; 3],
        normal: &Vec3,
        are_adjacent: bool,
        epsilon: f64,
    ) -> bool {
        // Project to 2D using axis with maximum normal component
        let ax = normal.x().abs();
        let ay = normal.y().abs();
        let az = normal.z().abs();

        let project_2d = |v: &Vec3| -> Vec2 {
            if ax >= ay && ax >= az {
                Vec2::new(v.y(), v.z())
            } else if ay >= ax && ay >= az {
                Vec2::new(v.x(), v.z())
            } else {
                Vec2::new(v.x(), v.y())
            }
        };

        let a2d: [Vec2; 3] = [
            project_2d(&tri_a[0]),
            project_2d(&tri_a[1]),
            project_2d(&tri_a[2]),
        ];
        let b2d: [Vec2; 3] = [
            project_2d(&tri_b[0]),
            project_2d(&tri_b[1]),
            project_2d(&tri_b[2]),
        ];

        // For adjacent surfaces sharing an edge, detect only non-shared edge intersections
        if are_adjacent {
            let shared = Self::count_shared_vertices(tri_a, tri_b, epsilon);
            if shared >= 2 {
                return false;
            }
        }

        // Edge-edge intersection test
        let edges_a = [(0, 1), (1, 2), (2, 0)];
        let edges_b = [(0, 1), (1, 2), (2, 0)];

        for &(a0, a1) in &edges_a {
            for &(b0, b1) in &edges_b {
                if are_adjacent {
                    // Skip endpoint contact of edges containing shared vertices
                    let a0_shared = Self::is_vertex_shared(&tri_a[a0], tri_b, epsilon);
                    let a1_shared = Self::is_vertex_shared(&tri_a[a1], tri_b, epsilon);
                    let b0_shared = Self::is_vertex_shared(&tri_b[b0], tri_a, epsilon);
                    let b1_shared = Self::is_vertex_shared(&tri_b[b1], tri_a, epsilon);

                    if (a0_shared || a1_shared) && (b0_shared || b1_shared) {
                        continue;
                    }
                }

                if Self::segments_intersect_2d(a2d[a0], a2d[a1], b2d[b0], b2d[b1], epsilon) {
                    return true;
                }
            }
        }

        // Case where one triangle completely contains the other
        if Self::point_in_triangle_2d(a2d[0], &b2d, epsilon)
            || Self::point_in_triangle_2d(b2d[0], &a2d, epsilon)
        {
            // For adjacent surfaces sharing vertices, containment of only shared vertex is contact
            if are_adjacent {
                let shared = Self::count_shared_vertices(tri_a, tri_b, epsilon);
                if shared >= 1 {
                    return false;
                }
            }
            return true;
        }

        false
    }

    /// Checks if a vertex matches any vertex in the triangle.
    fn is_vertex_shared(v: &Vec3, tri: &[Vec3; 3], epsilon: f64) -> bool {
        let eps_sq = epsilon * epsilon;
        tri.iter().any(|t| v.sub(*t).length_sq() <= eps_sq)
    }

    /// 2D line segment intersection test (excludes endpoint contact).
    fn segments_intersect_2d(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2, epsilon: f64) -> bool {
        let d1 = Self::cross_2d_val(a1, a2, b1);
        let d2 = Self::cross_2d_val(a1, a2, b2);
        let d3 = Self::cross_2d_val(b1, b2, a1);
        let d4 = Self::cross_2d_val(b1, b2, a2);

        if ((d1 > epsilon && d2 < -epsilon) || (d1 < -epsilon && d2 > epsilon))
            && ((d3 > epsilon && d4 < -epsilon) || (d3 < -epsilon && d4 > epsilon))
        {
            return true;
        }

        false
    }

    fn cross_2d_val(a: Vec2, b: Vec2, c: Vec2) -> f64 {
        (b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x())
    }
    /// Tests if a point is inside a triangle (2D).
    fn point_in_triangle_2d(p: Vec2, tri: &[Vec2; 3], epsilon: f64) -> bool {
        let d1 = Self::cross_2d_val(tri[0], tri[1], p);
        let d2 = Self::cross_2d_val(tri[1], tri[2], p);
        let d3 = Self::cross_2d_val(tri[2], tri[0], p);

        let has_neg = d1 < -epsilon || d2 < -epsilon || d3 < -epsilon;
        let has_pos = d1 > epsilon || d2 > epsilon || d3 > epsilon;

        // All same sign → interior (excludes boundary)
        !(has_neg && has_pos) && (d1.abs() > epsilon && d2.abs() > epsilon && d3.abs() > epsilon)
    }

    // Public methods

    /// Triangulates all surfaces into a list of triangles.
    pub fn triangulate(&self) -> Result<Vec<Triangle>, Error> {
        let mut all_triangles = Vec::new();
        for surface in &self.surfaces {
            let mut triangles = surface.triangulate()?;
            all_triangles.append(&mut triangles);
        }
        Ok(all_triangles)
    }

    /// Returns the surfaces defining this solid.
    pub fn surfaces(&self) -> &[Polygon] {
        &self.surfaces
    }

    /// Returns the epsilon value used for geometric operations.
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    pub fn single_ids(&self, _z: u8)
    //-> Result<impl Iterator<Item = SingleId>, Error>
    {
        todo!()
    }

    pub fn range_ids(&self, _z: u8)
    //-> Result<impl Iterator<Item = RangeId>, Error>
    {
        todo!()
    }
}
