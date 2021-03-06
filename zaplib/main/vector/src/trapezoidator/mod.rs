use crate::geometry::{LineSegment, Point, Trapezoid};
use crate::internal_iter::InternalIterator;
use crate::path::{LinePathCommand, LinePathIterator};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::mem;
use std::ops::Range;

/// Converts a sequence of line path commands to a sequence of trapezoids. The line path commands
/// should define a set of closed contours.
#[derive(Clone, Debug, Default)]
pub struct Trapezoidator {
    event_queue: BinaryHeap<Event>,
    active_segments: Vec<ActiveSegment>,
}

impl Trapezoidator {
    /// Creates a new trapezoidator.

    /// Returns an iterator over trapezoids corresponding to the given iterator over line path
    /// commands.
    pub fn trapezoidate<P: LinePathIterator>(&mut self, path: P) -> Option<Trapezoidate> {
        let mut initial_point = None;
        let mut current_point = None;

        // TODO(Paras): Extract this to a variable. I could not think of a descriptive name
        // so leaving for later.
        #[warn(clippy::blocks_in_if_conditions)]
        if !path.for_each(&mut |command| {
            match command {
                LinePathCommand::MoveTo(p) => {
                    //assert!(initial_point == current_point);
                    initial_point = Some(p);
                    current_point = Some(p);
                }
                LinePathCommand::LineTo(p) => {
                    let p0 = current_point.replace(p).unwrap();
                    if self.push_events_for_segment(LineSegment::new(p0, p)) {
                        return false;
                    }
                }
                LinePathCommand::Close => {
                    let p = initial_point.take().unwrap();
                    let p0 = current_point.replace(p).unwrap();
                    if self.push_events_for_segment(LineSegment::new(p0, p)) {
                        return false;
                    }
                }
            }
            true
        }) {
            return None;
        };
        Some(Trapezoidate { trapezoidator: self })
    }

    fn push_events_for_segment(&mut self, segment: LineSegment) -> bool {
        let (winding, p0, p1) = match segment.p0.partial_cmp(&segment.p1) {
            None => return true,
            Some(Ordering::Less) => (1, segment.p0, segment.p1),
            Some(Ordering::Equal) => return false,
            Some(Ordering::Greater) => (-1, segment.p1, segment.p0),
        };
        self.event_queue.push(Event { point: p0, pending_segment: Some(PendingSegment { winding, p1 }) });
        self.event_queue.push(Event { point: p1, pending_segment: None });
        false
    }

    fn pop_events_for_point(&mut self, pending_segments: &mut Vec<PendingSegment>) -> Option<Point> {
        self.event_queue.pop().map(|event| {
            if let Some(pending_segment) = event.pending_segment {
                pending_segments.push(pending_segment)
            }
            while let Some(&next_event) = self.event_queue.peek() {
                if next_event != event {
                    break;
                }
                self.event_queue.pop();
                if let Some(pending_segment) = next_event.pending_segment {
                    pending_segments.push(pending_segment);
                }
            }
            event.point
        })
    }

    fn handle_events_for_point<F>(
        &mut self,
        point: Point,
        right_segments: &mut Vec<PendingSegment>,
        trapezoid_segments: &mut Vec<ActiveSegment>,
        f: &mut F,
    ) -> bool
    where
        F: FnMut(Trapezoid) -> bool,
    {
        let mut incident_segment_range = self.find_incident_segment_range(point);
        if let Some(trapezoid_segment) = self.find_lower_trapezoid_segment(point, incident_segment_range.start) {
            trapezoid_segments.push(trapezoid_segment);
        }
        self.remove_incident_segments(point, &mut incident_segment_range, right_segments, trapezoid_segments);
        self.sort_right_segments(point, right_segments);
        self.insert_right_segments(point, &mut incident_segment_range, right_segments);
        if let Some(trapezoid_segment) = self.find_upper_trapezoid_segment(point, incident_segment_range.end) {
            trapezoid_segments.push(trapezoid_segment);
        }
        self.generate_trapezoids(trapezoid_segments, f)
    }

    fn find_incident_segment_range(&self, point: Point) -> Range<usize> {
        Range {
            start: self
                .active_segments
                .iter()
                .position(|active_segment| active_segment.segment.compare_to_point(point).unwrap() != Ordering::Less)
                .unwrap_or(self.active_segments.len()),
            end: self
                .active_segments
                .iter()
                .rposition(|active_segment| active_segment.segment.compare_to_point(point).unwrap() != Ordering::Greater)
                .map_or(0, |index| index + 1),
        }
    }

    fn find_lower_trapezoid_segment(&mut self, point: Point, incident_segment_start: usize) -> Option<ActiveSegment> {
        if 0 == incident_segment_start || !self.active_segments[incident_segment_start - 1].upper_region.is_inside {
            return None;
        }
        let intersection =
            self.active_segments[incident_segment_start - 1].segment.intersect_with_vertical_line(point.x).unwrap();
        self.active_segments[incident_segment_start - 1].split_front_mut(intersection)
    }

    fn remove_incident_segments(
        &mut self,
        point: Point,
        incident_segment_range: &mut Range<usize>,
        pending_segments: &mut Vec<PendingSegment>,
        trapezoid_segments: &mut Vec<ActiveSegment>,
    ) {
        #[allow(clippy::float_cmp)]
        trapezoid_segments.extend(
            Iterator::map(self.active_segments.drain(incident_segment_range.clone()), |mut active_segment| {
                if let Some(pending_segment) = active_segment.split_back_mut(point) {
                    pending_segments.push(pending_segment);
                }
                active_segment
            })
            .filter(|active_segment| active_segment.segment.p0.x != active_segment.segment.p1.x),
        );
        incident_segment_range.end = incident_segment_range.start;
    }

    fn sort_right_segments(&mut self, point: Point, right_segments: &mut Vec<PendingSegment>) {
        right_segments.sort_by(|&right_segment_0, &right_segment_1| right_segment_0.compare(right_segment_1, point).unwrap());
        let mut index_0 = 0;
        for index_1 in 1..right_segments.len() {
            let right_segment_1 = right_segments[index_1];
            let right_segment_0 = &mut right_segments[index_0];
            if right_segment_0.overlaps(right_segment_1, point) {
                if let Some(event) = right_segment_0.splice_mut(right_segment_1) {
                    self.event_queue.push(event);
                }
            } else {
                index_0 += 1;
                right_segments[index_0] = right_segment_1;
            }
        }
        right_segments.truncate(index_0 + 1);
    }

    fn insert_right_segments(
        &mut self,
        point: Point,
        incident_segment_range: &mut Range<usize>,
        right_segments: &[PendingSegment],
    ) {
        let mut lower_region = if incident_segment_range.end == 0 {
            Region { is_inside: false, winding: 0 }
        } else {
            self.active_segments[incident_segment_range.end - 1].upper_region
        };
        self.active_segments.splice(
            incident_segment_range.end..incident_segment_range.end,
            Iterator::map(right_segments.iter(), |right_segment| {
                let upper_region = {
                    let winding = lower_region.winding + right_segment.winding;
                    Region { is_inside: winding != 0, winding }
                };
                let right_segment = ActiveSegment {
                    winding: right_segment.winding,
                    segment: LineSegment::new(point, right_segment.p1),
                    upper_region,
                };
                lower_region = upper_region;
                right_segment
            }),
        );
        incident_segment_range.end += right_segments.len();
    }

    fn find_upper_trapezoid_segment(&mut self, point: Point, incident_segment_end: usize) -> Option<ActiveSegment> {
        if 0 == incident_segment_end || !self.active_segments[incident_segment_end - 1].upper_region.is_inside {
            return None;
        }
        let intersection = self.active_segments[incident_segment_end].segment.intersect_with_vertical_line(point.x).unwrap();
        if let Some(pending_segment) = self.active_segments[incident_segment_end].split_back_mut(intersection) {
            self.event_queue.push(Event { point: intersection, pending_segment: Some(pending_segment) });
        }
        Some(self.active_segments[incident_segment_end])
    }

    fn generate_trapezoids<F>(&self, trapezoid_segments: &[ActiveSegment], f: &mut F) -> bool
    where
        F: FnMut(Trapezoid) -> bool,
    {
        for trapezoid_segment_pair in trapezoid_segments.windows(2) {
            if !trapezoid_segment_pair[0].upper_region.is_inside {
                continue;
            }
            let lower_segment = trapezoid_segment_pair[0].segment;
            let upper_segment = trapezoid_segment_pair[1].segment;
            if !f(Trapezoid {
                xs: [lower_segment.p0.x, lower_segment.p1.x],
                ys: [lower_segment.p0.y, lower_segment.p1.y, upper_segment.p0.y, upper_segment.p1.y],
            }) {
                return false;
            }
        }
        true
    }
}

/// An iterator over trapezoids corresponding to the given iterator over line path commands.
#[derive(Debug)]
pub struct Trapezoidate<'a> {
    trapezoidator: &'a mut Trapezoidator,
}

impl<'a> InternalIterator for Trapezoidate<'a> {
    type Item = Trapezoid;

    fn for_each<F>(self, f: &mut F) -> bool
    where
        F: FnMut(Trapezoid) -> bool,
    {
        let mut right_segments = Vec::new();
        let mut trapezoid_segments = Vec::new();
        while let Some(point) = self.trapezoidator.pop_events_for_point(&mut right_segments) {
            let ok = self.trapezoidator.handle_events_for_point(point, &mut right_segments, &mut trapezoid_segments, f);
            right_segments.clear();
            trapezoid_segments.clear();
            if !ok {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Copy, Debug)]
struct Event {
    point: Point,
    pending_segment: Option<PendingSegment>,
}

impl Eq for Event {}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        self.point.partial_cmp(&other.point).unwrap().reverse()
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingSegment {
    winding: i32,
    p1: Point,
}

impl PendingSegment {
    fn to_segment(self, p0: Point) -> LineSegment {
        LineSegment::new(p0, self.p1)
    }

    fn overlaps(self, other: PendingSegment, p0: Point) -> bool {
        self.compare(other, p0) == Some(Ordering::Equal)
    }

    fn compare(self, other: PendingSegment, p0: Point) -> Option<Ordering> {
        if self.p1 <= other.p1 {
            other.to_segment(p0).compare_to_point(self.p1).map(|ordering| ordering.reverse())
        } else {
            self.to_segment(p0).compare_to_point(other.p1)
        }
    }

    fn splice_mut(&mut self, mut other: Self) -> Option<Event> {
        if other.p1 < self.p1 {
            mem::swap(self, &mut other);
        }
        self.winding += other.winding;
        if self.p1 == other.p1 {
            return None;
        }
        Some(Event { point: self.p1, pending_segment: Some(other) })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ActiveSegment {
    winding: i32,
    segment: LineSegment,
    upper_region: Region,
}

impl ActiveSegment {
    fn split_front_mut(&mut self, p: Point) -> Option<ActiveSegment> {
        let p0 = self.segment.p0;
        if p == p0 {
            return None;
        }
        self.segment.p0 = p;
        Some(ActiveSegment { winding: self.winding, segment: LineSegment::new(p0, p), upper_region: self.upper_region })
    }

    fn split_back_mut(&mut self, p: Point) -> Option<PendingSegment> {
        let p1 = self.segment.p1;
        if p == p1 {
            return None;
        }
        self.segment.p1 = p;
        Some(PendingSegment { winding: self.winding, p1 })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Region {
    is_inside: bool,
    winding: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Point, Transform, Transformation};
    use crate::path::PathCommand;
    use crate::path::PathIterator;
    use std::iter::Cloned;
    use std::slice::Iter;

    /// A sequence of commands that defines a set of contours, each of which consists of a sequence of
    /// curve segments. Each contour is either open or closed.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub(crate) struct Path {
        verbs: Vec<Verb>,
        points: Vec<Point>,
    }

    impl Path {
        /// Creates a new empty path.

        // /// Returns a slice of the points that make up `self`.
        // pub(crate) fn points(&self) -> &[Point] {
        //     &self.points
        // }

        /// Returns an iterator over the commands that make up `self`.
        pub(crate) fn commands(&self) -> Commands {
            Commands { verbs: self.verbs.iter().cloned(), points: self.points.iter().cloned() }
        }

        /// Returns a mutable slice of the points that make up `self`.
        pub(crate) fn points_mut(&mut self) -> &mut [Point] {
            &mut self.points
        }

        /// Adds a new contour, starting at the given point.
        pub(crate) fn move_to(&mut self, p: Point) {
            self.verbs.push(Verb::MoveTo);
            self.points.push(p);
        }

        // /// Adds a line segment to the current contour, starting at the current point.
        // pub(crate) fn line_to(&mut self, p: Point) {
        //     self.verbs.push(Verb::LineTo);
        //     self.points.push(p);
        // }

        // Adds a quadratic Bezier curve segment to the current contour, starting at the current point.
        pub(crate) fn quadratic_to(&mut self, p1: Point, p: Point) {
            self.verbs.push(Verb::QuadraticTo);
            self.points.push(p1);
            self.points.push(p);
        }

        /// Closes the current contour.
        pub(crate) fn close(&mut self) {
            self.verbs.push(Verb::Close);
        }

        // /// Clears `self`.
        // pub(crate) fn clear(&mut self) {
        //     self.verbs.clear();
        //     self.points.clear();
        // }
    }

    impl Transform for Path {
        fn transform<T>(mut self, t: &T) -> Path
        where
            T: Transformation,
        {
            self.transform_mut(t);
            self
        }

        fn transform_mut<T>(&mut self, t: &T)
        where
            T: Transformation,
        {
            for point in self.points_mut() {
                point.transform_mut(t);
            }
        }
    }

    /// An iterator over the commands that make up a path.
    #[derive(Clone, Debug)]
    pub struct Commands<'a> {
        verbs: Cloned<Iter<'a, Verb>>,
        points: Cloned<Iter<'a, Point>>,
    }

    impl<'a> Iterator for Commands<'a> {
        type Item = PathCommand;

        fn next(&mut self) -> Option<PathCommand> {
            self.verbs.next().map(|verb| match verb {
                Verb::MoveTo => PathCommand::MoveTo(self.points.next().unwrap()),
                // Verb::LineTo => PathCommand::LineTo(self.points.next().unwrap()),
                Verb::QuadraticTo => PathCommand::QuadraticTo(self.points.next().unwrap(), self.points.next().unwrap()),
                Verb::Close => PathCommand::Close,
            })
        }
    }

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    enum Verb {
        MoveTo,
        // LineTo,
        QuadraticTo,
        Close,
    }

    #[test]
    fn test() {
        let mut path = Path::default();
        path.move_to(Point::new(1.0, 0.0));
        path.quadratic_to(Point::new(1.0, 1.0), Point::new(0.0, 1.0));
        path.quadratic_to(Point::new(-1.0, 1.0), Point::new(-1.0, 0.0));
        path.quadratic_to(Point::new(-1.0, -1.0), Point::new(0.0, -1.0));
        path.quadratic_to(Point::new(-1.0, -1.0), Point::new(0.0, -1.0));
        path.close();

        let mut result = Vec::new();
        Trapezoidator::default().trapezoidate(path.commands().linearize(0.1)).unwrap().for_each(&mut |item| {
            result.push(item);
            true
        });
        assert_eq!(
            result,
            [
                Trapezoid { xs: [-1.0, -0.9375], ys: [0.0, -0.4375, 0.0, 0.4375] },
                Trapezoid { xs: [-0.9375, -0.75], ys: [-0.4375, -0.75, 0.4375, 0.75] },
                Trapezoid { xs: [-0.75, -0.4375], ys: [-0.75, -0.9375, 0.75, 0.9375] },
                Trapezoid { xs: [-0.4375, 0.0], ys: [-0.9375, -1.0, 0.9375, 1.0] },
                Trapezoid { xs: [0.0, 0.4375], ys: [-1.0, -0.5625, 1.0, 0.9375] },
                Trapezoid { xs: [0.4375, 0.75], ys: [-0.5625, -0.25, 0.9375, 0.75] },
                Trapezoid { xs: [0.75, 0.9375], ys: [-0.25, -0.0625, 0.75, 0.4375] },
                Trapezoid { xs: [0.9375, 1.0], ys: [-0.0625, 0.0, 0.4375, 0.0] }
            ]
        );
    }
}
