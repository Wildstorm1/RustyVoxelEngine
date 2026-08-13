/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use std::ops::{Add, Sub};

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub struct Coord {
    x: i64,
    y: i64,
    z: i64,
}

impl Coord {
    pub fn new(
        x: i64,
        y: i64,
        z: i64
    ) -> Self {
        Self { x: x, y: y, z: z }
    }

    pub fn unit_x() -> Self {
        Self { x: 1, y: 0, z: 0 }
    }

    pub fn unit_y() -> Self {
        Self { x: 0, y: 1, z: 0 }
    }

    pub fn unit_z() -> Self {
        Self { x: 0, y: 0, z: 1 }
    }

    pub fn x(&self) -> i64 {
        self.x
    }

    pub fn y(&self) -> i64 {
        self.y
    }

    pub fn z(&self) -> i64 {
        self.z
    }
}

impl Add for Coord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z
        }
    }
}

impl Sub for Coord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Coord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z
        }
    }
}