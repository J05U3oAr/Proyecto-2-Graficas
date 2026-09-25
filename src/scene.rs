use std::cmp::Ordering;

use crate::geometry::{Aabb, Cube, Hit, Ray, Vec3};
use crate::materials::Material;

const LEAF: usize = usize::MAX;

#[derive(Clone, Copy)]
struct BvhNode {
    bounds: Aabb,
    left: usize,
    right: usize,
    start: usize,
    count: usize,
}

pub struct Scene {
    cubes: Vec<Cube>,
    materials: Vec<Material>,
    nodes: Vec<BvhNode>,
    order: Vec<usize>,
    root: usize,
}

impl Scene {
    pub fn new(materials: Vec<Material>) -> Self {
        Self {
            cubes: Vec::new(),
            materials,
            nodes: Vec::new(),
            order: Vec::new(),
            root: 0,
        }
    }

    pub fn add_cube(&mut self, x: f32, y: f32, z: f32, material: usize) {
        self.cubes.push(Cube {
            min: Vec3::new(x, y, z),
            max: Vec3::new(x + 1., y + 1., z + 1.),
            material,
        });
    }

    pub fn rebuild_bvh(&mut self) {
        self.nodes.clear();
        self.order.clear();
        let mut indices: Vec<usize> = (0..self.cubes.len()).collect();
        self.root = self.build_node(&mut indices);
    }

    fn build_node(&mut self, indices: &mut [usize]) -> usize {
        let mut bounds = self.cubes[indices[0]].bounds();
        for &index in &indices[1..] {
            bounds = Aabb::union(bounds, self.cubes[index].bounds());
        }

        if indices.len() <= 8 {
            let start = self.order.len();
            self.order.extend_from_slice(indices);
            let index = self.nodes.len();
            self.nodes.push(BvhNode {
                bounds,
                left: LEAF,
                right: LEAF,
                start,
                count: indices.len(),
            });
            return index;
        }

        let extent = bounds.max - bounds.min;
        let axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };
        indices.sort_by(|a, b| {
            let center_a = (self.cubes[*a].min[axis] + self.cubes[*a].max[axis]) * 0.5;
            let center_b = (self.cubes[*b].min[axis] + self.cubes[*b].max[axis]) * 0.5;
            center_a.partial_cmp(&center_b).unwrap_or(Ordering::Equal)
        });
        let middle = indices.len() / 2;
        let (first, second) = indices.split_at_mut(middle);
        let left = self.build_node(first);
        let right = self.build_node(second);
        let index = self.nodes.len();
        self.nodes.push(BvhNode {
            bounds,
            left,
            right,
            start: 0,
            count: 0,
        });
        index
    }

    pub fn hit(&self, ray: Ray, max_t: f32) -> Option<Hit> {
        let mut closest = max_t;
        let mut result = None;
        let mut stack = vec![self.root];
        while let Some(node_index) = stack.pop() {
            let node = self.nodes[node_index];
            if node.bounds.entry(ray, closest).is_none() {
                continue;
            }
            if node.left == LEAF {
                for &cube_index in &self.order[node.start..node.start + node.count] {
                    if let Some(hit) = self.cubes[cube_index].intersect(ray, closest) {
                        closest = hit.t;
                        result = Some(hit);
                    }
                }
            } else {
                let left = self.nodes[node.left].bounds.entry(ray, closest);
                let right = self.nodes[node.right].bounds.entry(ray, closest);
                match (left, right) {
                    (Some(l), Some(r)) if l < r => {
                        stack.push(node.right);
                        stack.push(node.left);
                    }
                    (Some(_), Some(_)) => {
                        stack.push(node.left);
                        stack.push(node.right);
                    }
                    (Some(_), None) => stack.push(node.left),
                    (None, Some(_)) => stack.push(node.right),
                    _ => {}
                }
            }
        }
        result
    }

    /// Shadow rays only need any blocker; unlike `hit`, this exits immediately.
    pub fn occluded(&self, ray: Ray) -> bool {
        let mut stack = vec![self.root];
        while let Some(node_index) = stack.pop() {
            let node = self.nodes[node_index];
            if node.bounds.entry(ray, f32::INFINITY).is_none() {
                continue;
            }
            if node.left == LEAF {
                for &cube_index in &self.order[node.start..node.start + node.count] {
                    if self.cubes[cube_index]
                        .intersect(ray, f32::INFINITY)
                        .is_some()
                    {
                        return true;
                    }
                }
            } else {
                stack.push(node.left);
                stack.push(node.right);
            }
        }
        false
    }

    pub fn material(&self, index: usize) -> Material {
        self.materials[index]
    }
    pub fn cube_count(&self) -> usize {
        self.cubes.len()
    }
}
