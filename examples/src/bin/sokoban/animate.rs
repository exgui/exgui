use std::{
    ops::{Add, Deref, Sub},
    time::Duration,
};

#[derive(Debug, Default)]
pub struct Animate<T> {
    current: T,
    target: T,
    speed: f32,
}

impl<T> Animate<T> {
    pub fn new(start: T, target: T, speed: f32) -> Self {
        Self {
            current: start,
            target,
            speed,
        }
    }

    pub fn from(&mut self, start: T) {
        self.current = start;
    }

    pub fn to(&mut self, target: T) {
        self.target = target;
    }
}

impl<T: Copy> Animate<T> {
    pub fn val(&self) -> T {
        self.current
    }
}

impl<T: Copy + PartialEq> Animate<T> {
    pub fn is_transient(&self) -> bool {
        self.current != self.target
    }

    pub fn set(&mut self, value: T) {
        if !self.is_transient() {
            self.current = value;
        }
        self.target = value;
    }
}

impl<T: Copy + PartialOrd + Add<Output = T> + Sub<Output = T> + From<f32>> Animate<T> {
    pub fn animate(&mut self, elapsed: Duration) {
        if self.current > self.target {
            let next = self.current - (elapsed.as_millis() as f32 * self.speed).into();
            self.current = if next < self.target { self.target } else { next };
        } else if self.current < self.target {
            let next = self.current + (elapsed.as_millis() as f32 * self.speed).into();
            self.current = if next > self.target { self.target } else { next };
        }
    }
}

impl<T> Deref for Animate<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}
