//
// traits.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use std::{fmt, ops, slice};

use anyhow::Error;
use nalgebra::allocator::Allocator;
use nalgebra::{DefaultAllocator, DimName, VectorN};
use rand::Rng;
use serde::Serialize;
use svg::node::element::Group;
use svg::Document;

use crate::{StandardBasis, Transform2};

pub trait Transformer {
    fn as_simple(&self) -> String;
}

pub trait Basis {
    fn set_value(&mut self, new_value: f64);
    fn get_value(&self) -> f64;
    fn reset_value(&self);
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R, step_size: f64) -> f64;
    fn set_sampled<R: Rng + ?Sized>(&mut self, rng: &mut R, step_size: f64);
}

pub trait Periodic<Rhs = Self> {
    type Output;

    fn periodic(&self, rhs: Rhs) -> Self::Output;
}

pub trait PeriodicAssign<Rhs = Self> {
    fn periodic_assign(&mut self, rhs: Rhs);
}

pub trait AdjustPeriod<D: DimName>
where
    DefaultAllocator: Allocator<f64, D> + Allocator<f64, D, D>,
{
    type Output;
    fn adjust_period(&self, adjustment: VectorN<f64, D>) -> Self::Output;
}

pub trait Intersect {
    fn intersects(&self, other: &Self) -> bool;
    fn area(&self) -> f64;
}

pub trait Potential {
    fn energy(&self, other: &Self) -> f64;
}

pub trait Shape:
    Clone + Send + Sync + Serialize + fmt::Debug + fmt::Display + ToSVG<Value = Group>
{
    type Component: Clone
        + Send
        + Sync
        + Serialize
        + fmt::Debug
        + fmt::Display
        + ops::Mul<Transform2, Output = Self::Component>
        + ToSVG;

    fn score(&self, other: &Self) -> Option<f64>;
    fn enclosing_radius(&self) -> f64;
    fn get_items(&self) -> Vec<Self::Component>;
    fn rotational_symmetries(&self) -> u64 {
        1
    }
    fn iter(&self) -> slice::Iter<'_, Self::Component>;
    fn transform(&self, transform: &Transform2) -> Self;
}

pub trait FromSymmetry: Sized {
    fn from_operations(ops: &str) -> Result<Self, Error>;
}

pub trait State:
    Eq
    + PartialEq
    + PartialOrd
    + Ord
    + Clone
    + Send
    + Sync
    + Serialize
    + fmt::Debug
    + ToSVG<Value = Document>
{
    fn score(&self) -> Option<f64>;
    fn generate_basis(&self) -> Vec<StandardBasis>;
    fn total_shapes(&self) -> usize;
    fn as_positions(&self) -> Result<String, Error>;
}

pub trait ToSVG {
    type Value: svg::Node;
    fn as_svg(&self) -> Self::Value;
}
