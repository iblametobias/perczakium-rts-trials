use std::ops::{Index, IndexMut};

use macroquad::math::{UVec2, IVec2, Vec2};

/// 2D dense grid stored in a flat vector.
///
/// Layout:
/// ```text
/// [0,0] [1,0] [2,0]
/// [0,1] [1,1] [2,1]
/// ```
///
/// Internal indexing:
/// ```text
/// index = y * width + x
/// ```
pub struct Grid<T: Default + Clone> {
    size: IVec2,
    tile_size: f32,

    tiles: Vec<T>,
}

impl<T: Default + Clone> Grid<T> {
    /// Creates a new grid filled with default values.
    pub fn new(size: UVec2, tile_size: f32) -> Self {
        Self {
            size: size.as_ivec2(),
            tile_size: tile_size,

            // Fill every tile with T::default().
            tiles: vec![T::default(); (size.x * size.y) as usize],
        }
    }

    /// Converts tile coordinates into a flat vector index.
    ///
    /// Returns None if tile is outside the grid.
    pub fn tile_to_index(&self, tile: IVec2) -> Option<usize> {
        if !self.in_bounds(tile) {
            return None;
        }

        Some((tile.y * self.size.x + tile.x) as usize)
    }

    /// Converts tile coordinates into world-space position.
    ///
    /// Position refers to tile origin, not center.
    pub fn tile_to_position(&self, tile: IVec2) -> Vec2 {
        tile.as_vec2() * self.tile_size
    }

    /// Converts flat index into world-space position.
    pub fn index_to_position(&self, index: usize) -> Vec2 {
        self.tile_to_position(self.index_to_tile(index))
    }

    /// Converts flat index into tile coordinates.
    pub fn index_to_tile(&self, index: usize) -> IVec2 {
        IVec2::new(
            index as i32 % self.size.x,
            index as i32 / self.size.x,
        )
    }

    /// Converts world-space position into tile coordinates.
    ///
    /// Uses Euclidean division so negative positions work correctly.
    pub fn position_to_tile(&self, position: Vec2) -> IVec2 {
        position
            .div_euclid(Vec2::splat(self.tile_size))
            .as_ivec2()
    }

    /// Converts world-space position into flat index.
    ///
    /// Returns None if outside grid.
    pub fn position_to_index(&self, position: Vec2) -> Option<usize> {
        self.tile_to_index(self.position_to_tile(position))
    }

    /// Returns true if tile coordinates are inside grid bounds.
    pub fn in_bounds(&self, tile: IVec2) -> bool {
        0 <= tile.x && tile.x < self.size.x &&
        0 <= tile.y && tile.y < self.size.y
    }

    /// Returns total grid size in world-space units.
    pub fn world_bounds(&self) -> Vec2 {
        self.size.as_vec2() * self.tile_size
    }

    /// Returns a reference to the value at a tile position.
    ///
    /// Returns None if the tile is outside grid bounds.
    pub fn get(&self, tile: IVec2) -> Option<&T> {
        self.tile_to_index(tile).map(|i| &self.tiles[i])
    }

    /// Returns a mutable reference to the value at a tile position.
    ///
    /// Returns None if the tile is outside grid bounds.
    pub fn get_mut(&mut self, tile: IVec2) -> Option<&mut T> {
        self.tile_to_index(tile).map(|i| &mut self.tiles[i])
    }

    /// Returns a reference to the value at a tile position without bounds checks.
    ///
    /// Safety:
    /// - Tile must be inside grid bounds
    /// - Out-of-bounds access causes undefined behavior
    pub unsafe fn get_unchecked(&self, tile: IVec2) -> &T {
        let i = (tile.y * self.size.x + tile.x) as usize;
        unsafe { self.tiles.get_unchecked(i) }
    }

    /// Returns a mutable reference to the value at a tile position without bounds checks.
    ///
    /// Safety:
    /// - Tile must be inside grid bounds
    /// - Out-of-bounds access causes undefined behavior
    pub unsafe fn get_unchecked_mut(&mut self, tile: IVec2) -> &mut T {
        let i = (tile.y * self.size.x + tile.x) as usize;
        unsafe { self.tiles.get_unchecked_mut(i) }
    }

    /// Attempts to set tile value.
    ///
    /// Does nothing if index is invalid.
    pub fn try_set_tile(&mut self, index: usize, value: T) {
        if let Some(tile) = self.tiles.get_mut(index) {
            *tile = value;
        }
    }

    /// Returns grid dimensions in tiles.
    pub fn get_size(&self) -> IVec2 {
        self.size
    }

    /// Returns world-space size of a single tile.
    pub fn get_tile_size(&self) -> f32 {
        self.tile_size
    }

    /// Returns total number of tiles in the grid.
    pub fn get_tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Sets all tiles to default
    pub fn clear(&mut self) {
        for t in self.tiles.iter_mut() {
            *t = T::default();
        }
    }
}

impl<T: Default + Clone> Index<usize> for Grid<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tiles[index]
    }
}

impl<T: Default + Clone> IndexMut<usize> for Grid<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.tiles[index]
    }
}

impl<'a, T: Default + Clone> IntoIterator for &'a Grid<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.tiles.iter()
    }
}

impl<'a, T: Default + Clone> IntoIterator for &'a mut Grid<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.tiles.iter_mut()
    }
}
