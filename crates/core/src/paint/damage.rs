use crate::geometry::rect::Rect;

/// Past this many rectangles a frame is cheaper to redraw as one region than to track piece by
/// piece, so the damage collapses into their bounding rectangle.
const MAXIMUM_RECTANGLES: usize = 16;

/// The parts of a surface whose pixels changed since the last display list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Damage {
    rectangles: Vec<Rect>,
}

impl Damage {
    /// Rectangles that together cover every changed pixel; none of them contains another.
    pub fn rectangles(&self) -> &[Rect] {
        &self.rectangles
    }

    pub fn is_empty(&self) -> bool {
        self.rectangles.is_empty()
    }

    pub(crate) fn add(&mut self, rectangle: Rect) {
        if rectangle.is_empty() || self.rectangles.iter().any(|damaged| damaged.contains(&rectangle)) {
            return;
        }
        self.rectangles.retain(|damaged| !rectangle.contains(damaged));
        if self.rectangles.len() == MAXIMUM_RECTANGLES {
            let bounds = self
                .rectangles
                .iter()
                .fold(rectangle, |bounds, damaged| bounds.union(damaged));
            self.rectangles.clear();
            self.rectangles.push(bounds);
        } else {
            self.rectangles.push(rectangle);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.rectangles.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{Damage, MAXIMUM_RECTANGLES};
    use crate::geometry::rect::Rect;

    #[test]
    fn a_rectangle_inside_the_damage_adds_nothing() {
        let mut damage = Damage::default();

        damage.add(Rect::new(0.0, 0.0, 100.0, 100.0));
        damage.add(Rect::new(10.0, 10.0, 20.0, 20.0));
        damage.add(Rect::new(0.0, 0.0, 100.0, 100.0));

        assert_eq!(damage.rectangles(), [Rect::new(0.0, 0.0, 100.0, 100.0)]);
    }

    #[test]
    fn a_rectangle_that_covers_others_replaces_them() {
        let mut damage = Damage::default();

        damage.add(Rect::new(10.0, 10.0, 20.0, 20.0));
        damage.add(Rect::new(200.0, 0.0, 10.0, 10.0));
        damage.add(Rect::new(0.0, 0.0, 100.0, 100.0));

        assert_eq!(
            damage.rectangles(),
            [Rect::new(200.0, 0.0, 10.0, 10.0), Rect::new(0.0, 0.0, 100.0, 100.0)]
        );
    }

    #[test]
    fn a_rectangle_without_area_is_no_damage() {
        let mut damage = Damage::default();

        damage.add(Rect::new(5.0, 5.0, 0.0, 30.0));

        assert!(damage.is_empty());
    }

    #[test]
    fn too_many_rectangles_collapse_into_their_bounds() {
        let mut damage = Damage::default();
        for position in 0..MAXIMUM_RECTANGLES {
            damage.add(Rect::new(position as f32 * 20.0, 0.0, 10.0, 10.0));
        }
        assert_eq!(damage.rectangles().len(), MAXIMUM_RECTANGLES);

        damage.add(Rect::new(0.0, 50.0, 10.0, 10.0));

        assert_eq!(damage.rectangles(), [Rect::new(0.0, 0.0, 310.0, 60.0)]);
    }
}
