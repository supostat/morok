use crate::render::atlas_kind::AtlasKind;
use crate::render::draw_batch::{BatchSource, DrawBatch};
use crate::render::frame_command::FrameCommand;
use crate::render::pixel_rect::PixelRect;
use crate::render::tile_occupancy::TileOccupancy;

/// Groups the instances of a frame into as few draws as the paint order allows.
///
/// Rects and glyphs are drawn by different pipelines, so a frame that alternates between them
/// — a labelled box after a labelled box — would cost a draw per run. The batcher keeps one run
/// of rects and one run of glyphs open at a time, the rects drawn first: a rect that arrives
/// while glyphs are pending joins the rects when it covers none of the pixels the glyphs touch,
/// because drawing it under them then shows the same picture. A rect that does cover them
/// closes both runs and starts a new run after them.
#[derive(Debug, Default)]
pub(crate) struct DrawBatcher {
    rects: Option<DrawBatch>,
    glyphs: Option<DrawBatch>,
    /// The pixels the pending glyphs touch.
    glyph_pixels: TileOccupancy,
    /// The pixels of the last rect found clear of the pending glyphs: the border of a box
    /// follows its background over the same pixels, and needs no second look.
    clear_pixels: Option<PixelRect>,
}

impl DrawBatcher {
    /// Forgets the runs of the previous frame and sizes the map of glyph pixels for the target.
    pub(crate) fn begin_frame(&mut self, target: PixelRect) {
        self.rects = None;
        self.glyphs = None;
        self.glyph_pixels.reset(target);
        self.clear_pixels = None;
    }

    /// The rect instance at `position` of its buffer, covering `pixels` under `scissor`.
    pub(crate) fn rect(
        &mut self,
        position: usize,
        pixels: PixelRect,
        scissor: PixelRect,
        draws: &mut Vec<FrameCommand>,
    ) {
        if self.scissor_differs(scissor) || self.covers_pending_glyphs(pixels) {
            self.flush(draws);
        }
        self.clear_pixels = Some(pixels);
        match &mut self.rects {
            Some(batch) => batch.instance_count += 1,
            None => self.rects = Some(DrawBatch::starting(BatchSource::Rects, scissor, position)),
        }
    }

    /// The glyph instance at `position` of its buffer, drawn from `page` of `atlas` under
    /// `scissor`. The pixels it covers are reported by [`glyphs_cover`](Self::glyphs_cover),
    /// once for a whole run.
    pub(crate) fn glyph(
        &mut self,
        position: usize,
        scissor: PixelRect,
        atlas: AtlasKind,
        page: usize,
        draws: &mut Vec<FrameCommand>,
    ) {
        let source = BatchSource::Glyphs { atlas, page };
        let page_differs = self.glyphs.is_some_and(|batch| batch.source != source);
        if self.scissor_differs(scissor) || page_differs {
            self.flush(draws);
        }
        match &mut self.glyphs {
            Some(batch) => batch.instance_count += 1,
            None => self.glyphs = Some(DrawBatch::starting(source, scissor, position)),
        }
    }

    /// The pixels the glyphs added since the last call cover: a rect over them from now on
    /// is drawn after them.
    pub(crate) fn glyphs_cover(&mut self, pixels: PixelRect) {
        self.glyph_pixels.mark(pixels);
        self.clear_pixels = None;
    }

    /// Closes the open runs: the frame has no more instances.
    pub(crate) fn finish(&mut self, draws: &mut Vec<FrameCommand>) {
        self.flush(draws);
    }

    /// Whether an open run is drawn under another scissor; both runs share one.
    fn scissor_differs(&self, scissor: PixelRect) -> bool {
        self.rects.or(self.glyphs).is_some_and(|batch| batch.scissor != scissor)
    }

    fn covers_pending_glyphs(&self, pixels: PixelRect) -> bool {
        self.clear_pixels != Some(pixels) && self.glyph_pixels.touches(pixels)
    }

    fn flush(&mut self, draws: &mut Vec<FrameCommand>) {
        draws.extend(self.rects.take().map(FrameCommand::Draw));
        draws.extend(self.glyphs.take().map(FrameCommand::Draw));
        self.glyph_pixels.clear();
    }
}

impl DrawBatch {
    fn starting(source: BatchSource, scissor: PixelRect, first_instance: usize) -> Self {
        Self {
            source,
            scissor,
            first_instance,
            instance_count: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DrawBatcher;
    use crate::render::atlas_kind::AtlasKind;
    use crate::render::draw_batch::{BatchSource, DrawBatch};
    use crate::render::frame_command::FrameCommand;
    use crate::render::pixel_rect::PixelRect;

    const TARGET: PixelRect = PixelRect {
        x: 0,
        y: 0,
        width: 1000,
        height: 1000,
    };
    const MASK_PAGE: BatchSource = BatchSource::Glyphs {
        atlas: AtlasKind::Mask,
        page: 0,
    };

    fn pixels(x: usize, y: usize, width: usize, height: usize) -> PixelRect {
        PixelRect { x, y, width, height }
    }

    fn batch(source: BatchSource, scissor: PixelRect, first_instance: usize, instance_count: usize) -> FrameCommand {
        FrameCommand::Draw(DrawBatch {
            source,
            scissor,
            first_instance,
            instance_count,
        })
    }

    /// Boxes of 32 by 20 pixels, 4 apart, every one labelled by a glyph in its middle.
    fn labelled_boxes(count: usize) -> Vec<FrameCommand> {
        let mut batcher = DrawBatcher::default();
        batcher.begin_frame(TARGET);
        let mut draws = Vec::new();
        for position in 0..count {
            let left = 4 + position * 36;
            batcher.rect(position, pixels(left, 4, 32, 20), TARGET, &mut draws);
            batcher.glyph(position, TARGET, AtlasKind::Mask, 0, &mut draws);
            batcher.glyphs_cover(pixels(left + 12, 10, 8, 8));
        }
        batcher.finish(&mut draws);
        draws
    }

    #[test]
    fn labelled_boxes_beside_each_other_draw_in_two_calls() {
        assert_eq!(
            labelled_boxes(20),
            [
                batch(BatchSource::Rects, TARGET, 0, 20),
                batch(MASK_PAGE, TARGET, 0, 20)
            ]
        );
    }

    #[test]
    fn a_rect_over_pending_glyphs_is_drawn_after_them() {
        let mut batcher = DrawBatcher::default();
        batcher.begin_frame(TARGET);
        let mut draws = Vec::new();
        batcher.rect(0, pixels(0, 0, 100, 100), TARGET, &mut draws);
        batcher.glyph(0, TARGET, AtlasKind::Mask, 0, &mut draws);
        batcher.glyphs_cover(pixels(40, 40, 10, 10));
        batcher.rect(1, pixels(30, 30, 40, 40), TARGET, &mut draws);
        batcher.glyph(1, TARGET, AtlasKind::Mask, 0, &mut draws);
        batcher.glyphs_cover(pixels(45, 45, 10, 10));
        batcher.finish(&mut draws);

        assert_eq!(
            draws,
            [
                batch(BatchSource::Rects, TARGET, 0, 1),
                batch(MASK_PAGE, TARGET, 0, 1),
                batch(BatchSource::Rects, TARGET, 1, 1),
                batch(MASK_PAGE, TARGET, 1, 1),
            ]
        );
    }

    #[test]
    fn a_change_of_scissor_closes_both_runs() {
        let mut batcher = DrawBatcher::default();
        batcher.begin_frame(TARGET);
        let mut draws = Vec::new();
        let clipped = pixels(0, 0, 500, 500);
        batcher.rect(0, pixels(0, 0, 100, 100), TARGET, &mut draws);
        batcher.glyph(0, TARGET, AtlasKind::Mask, 0, &mut draws);
        batcher.glyphs_cover(pixels(40, 40, 10, 10));
        batcher.rect(1, pixels(600, 600, 100, 100), clipped, &mut draws);
        batcher.finish(&mut draws);

        assert_eq!(
            draws,
            [
                batch(BatchSource::Rects, TARGET, 0, 1),
                batch(MASK_PAGE, TARGET, 0, 1),
                batch(BatchSource::Rects, clipped, 1, 1),
            ]
        );
    }

    #[test]
    fn glyphs_of_another_page_are_drawn_after_the_rects_that_came_before_them() {
        let mut batcher = DrawBatcher::default();
        batcher.begin_frame(TARGET);
        let mut draws = Vec::new();
        batcher.glyph(0, TARGET, AtlasKind::Mask, 0, &mut draws);
        batcher.glyphs_cover(pixels(40, 40, 10, 10));
        batcher.rect(0, pixels(600, 600, 100, 100), TARGET, &mut draws);
        batcher.glyph(1, TARGET, AtlasKind::Color, 0, &mut draws);
        batcher.glyphs_cover(pixels(640, 640, 10, 10));
        batcher.finish(&mut draws);

        assert_eq!(
            draws,
            [
                batch(BatchSource::Rects, TARGET, 0, 1),
                batch(MASK_PAGE, TARGET, 0, 1),
                batch(
                    BatchSource::Glyphs {
                        atlas: AtlasKind::Color,
                        page: 0
                    },
                    TARGET,
                    1,
                    1
                ),
            ]
        );
    }

    #[test]
    fn a_glyph_over_a_box_makes_the_same_box_drawn_again_wait_for_it() {
        let mut batcher = DrawBatcher::default();
        batcher.begin_frame(TARGET);
        let mut draws = Vec::new();
        let box_pixels = pixels(0, 0, 100, 100);
        batcher.rect(0, box_pixels, TARGET, &mut draws);
        batcher.glyph(0, TARGET, AtlasKind::Mask, 0, &mut draws);
        batcher.glyphs_cover(pixels(40, 40, 10, 10));
        batcher.rect(1, box_pixels, TARGET, &mut draws);
        batcher.finish(&mut draws);

        assert_eq!(
            draws,
            [
                batch(BatchSource::Rects, TARGET, 0, 1),
                batch(MASK_PAGE, TARGET, 0, 1),
                batch(BatchSource::Rects, TARGET, 1, 1),
            ]
        );
    }

    #[test]
    fn a_new_frame_starts_with_no_pending_glyphs() {
        let mut batcher = DrawBatcher::default();
        batcher.begin_frame(TARGET);
        let mut draws = Vec::new();
        batcher.glyph(0, TARGET, AtlasKind::Mask, 0, &mut draws);
        batcher.glyphs_cover(pixels(40, 40, 10, 10));
        batcher.begin_frame(TARGET);
        batcher.rect(0, pixels(40, 40, 10, 10), TARGET, &mut draws);
        batcher.finish(&mut draws);

        assert_eq!(draws, [batch(BatchSource::Rects, TARGET, 0, 1)]);
    }
}
