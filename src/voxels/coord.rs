/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub struct ChunkCoord {
    cx: usize,
    cy: usize,
    cz: usize,
}

impl ChunkCoord {
    pub fn new(
        cx: usize,
        cy: usize,
        cz: usize
    ) -> Self {
        Self {
            cx: cx,
            cy: cy,
            cz: cz
        }
    }
}