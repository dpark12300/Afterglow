use iced::mouse;
use iced::widget::scrollable::Viewport;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollRegion {
    Sidebar,
    SearchResults,
    ImageSelection,
    BoopMatches,
}

#[derive(Debug, Default)]
pub struct SmoothScrollController {
    regions: HashMap<ScrollRegion, RegionScrollState>,
}

impl SmoothScrollController {
    pub fn handle_viewport_change(&mut self, region: ScrollRegion, viewport: Viewport) {
        let state = self.regions.entry(region).or_default();
        state.metrics = Some(ViewportMetrics::new(viewport));
        let observed = viewport.relative_offset().y.clamp(0.0, 1.0);

        if let Some(expected) = state.expected.front().copied() {
            if roughly_equal(expected, observed) {
                state.expected.pop_front();
                state.smooth.confirm(observed);
                return;
            }
        }

        state.expected.clear();
        state.smooth.reset(observed);
    }

    pub fn handle_wheel(&mut self, region: ScrollRegion, delta: mouse::ScrollDelta) {
        let state = self.regions.entry(region).or_default();
        let extent = state
            .metrics
            .map(|metrics| metrics.scroll_extent())
            .unwrap_or(DEFAULT_SCROLL_EXTENT);
        if extent <= f32::EPSILON {
            return;
        }

        let pixel_delta = match delta {
            mouse::ScrollDelta::Lines { y, .. } => -y * LINE_SCROLL_PIXELS,
            mouse::ScrollDelta::Pixels { y, .. } => -y,
        };

        if pixel_delta.abs() <= f32::EPSILON {
            return;
        }

        let relative = pixel_delta / extent;
        if relative.abs() <= f32::EPSILON {
            return;
        }

        let base = if state.smooth.is_active() {
            state.smooth.target()
        } else {
            state.smooth.current()
        };

        let target = (base + relative).clamp(0.0, 1.0);
        state.smooth.set_target(target);
    }

    pub fn step(&mut self) -> Vec<(ScrollRegion, f32)> {
        let mut updates = Vec::new();

        for (region, state) in self.regions.iter_mut() {
            if let Some(position) = state.smooth.step() {
                state.expected.push_back(position);
                updates.push((*region, position));
            }
        }

        updates
    }

    pub fn is_animating(&self) -> bool {
        self.regions.values().any(|state| state.smooth.is_active())
    }
}

#[derive(Debug)]
struct RegionScrollState {
    smooth: SmoothScrollState,
    expected: VecDeque<f32>,
    metrics: Option<ViewportMetrics>,
}

impl Default for RegionScrollState {
    fn default() -> Self {
        Self {
            smooth: SmoothScrollState::default(),
            expected: VecDeque::new(),
            metrics: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ViewportMetrics {
    viewport: f32,
    content: f32,
}

impl ViewportMetrics {
    fn new(viewport: Viewport) -> Self {
        Self {
            viewport: viewport.bounds().height,
            content: viewport.content_bounds().height,
        }
    }

    fn scroll_extent(&self) -> f32 {
        (self.content - self.viewport).max(1.0)
    }
}

#[derive(Debug, Clone, Copy)]
struct SmoothScrollState {
    current: f32,
    target: f32,
    active: bool,
}

impl Default for SmoothScrollState {
    fn default() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            active: false,
        }
    }
}

impl SmoothScrollState {
    fn confirm(&mut self, position: f32) {
        let clamped = position.clamp(0.0, 1.0);
        self.current = clamped;
        if roughly_equal(self.target, clamped) {
            self.target = clamped;
            self.active = false;
        } else {
            self.active = true;
        }
    }

    fn reset(&mut self, position: f32) {
        let clamped = position.clamp(0.0, 1.0);
        self.current = clamped;
        self.target = clamped;
        self.active = false;
    }

    fn set_target(&mut self, target: f32) {
        self.target = target.clamp(0.0, 1.0);
        if roughly_equal(self.current, self.target) {
            self.current = self.target;
            self.active = false;
        } else {
            self.active = true;
        }
    }

    fn target(&self) -> f32 {
        self.target
    }

    fn current(&self) -> f32 {
        self.current
    }

    fn step(&mut self) -> Option<f32> {
        if !self.active {
            return None;
        }

        let diff = self.target - self.current;
        if diff.abs() <= SMOOTH_EPSILON {
            self.current = self.target;
            self.active = false;
            Some(self.current)
        } else {
            self.current += diff * SMOOTH_FACTOR;
            Some(self.current)
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }
}

fn roughly_equal(a: f32, b: f32) -> bool {
    (a - b).abs() <= SMOOTH_EPSILON
}

const SMOOTH_FACTOR: f32 = 0.2;
const SMOOTH_EPSILON: f32 = 0.001;
const LINE_SCROLL_PIXELS: f32 = 120.0;
const DEFAULT_SCROLL_EXTENT: f32 = 1000.0;
