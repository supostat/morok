use std::ops::Range;

use accesskit::{
    Action, Node, NodeId, Rect as NodeRect, Role, TextDirection, TextPosition, TextSelection as NodeSelection,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::editing::caret_affinity::CaretAffinity;
use crate::editing::editable_text::EditableText;
use crate::editing::text_selection::TextSelection;
use crate::element_id::ElementId;
use crate::geometry::rect::Rect;
use crate::geometry::transform::Transform;

/// Marks the node of a run of an edited text. The node of an element is named by the handle
/// of the element, whose generation never reaches this bit.
const TEXT_RUN_MARK: u64 = 1 << 63;
const RUN_BITS: u32 = 31;
const RUN_MASK: u64 = (1 << RUN_BITS) - 1;
/// A run tells where its words start by the number of the character, in a byte, and how long
/// each character is, in a byte.
const MAXIMUM_RUN_CHARACTERS: usize = u8::MAX as usize;
const MAXIMUM_CHARACTER_BYTES: usize = u8::MAX as usize;

/// The node of a run of the edited text of an element.
pub(crate) fn text_run_node(element: ElementId, run: usize) -> NodeId {
    let run = u64::try_from(run).expect("the number of a run fits in 64 bits") & RUN_MASK;
    NodeId(TEXT_RUN_MARK | u64::from(element.index()) << RUN_BITS | run)
}

/// The run a node of the edited text of `element` stands for; none for any other node.
fn run_of_node(element: ElementId, node: NodeId) -> Option<usize> {
    let run = usize::try_from(node.0 & RUN_MASK).ok()?;
    (text_run_node(element, run) == node).then_some(run)
}

/// The characters of a text as an assistive technology steps through them: its grapheme
/// clusters, each with where it starts. A cluster too long for its length to be told in a
/// byte is stepped through a code point at a time.
fn characters(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.grapheme_indices(true).flat_map(|(start, cluster)| {
        let whole = (cluster.len() <= MAXIMUM_CHARACTER_BYTES).then_some((start, cluster));
        let code_points = cluster
            .char_indices()
            .filter(move |_| whole.is_none())
            .map(move |(position, code_point)| {
                (start + position, &cluster[position..position + code_point.len_utf8()])
            });
        whole.into_iter().chain(code_points)
    })
}

/// A stretch of one line that has a node of its own: a whole line, or a part of a line too
/// long for one node.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RunSpan {
    line: usize,
    /// The bytes of the text the run holds.
    text: Range<usize>,
    /// Whether the line goes on in the next run.
    continues: bool,
}

/// The edited text of an element as an assistive technology reads it: a node for every run of
/// every line, with its characters and where they stand, and the selection as places in those
/// runs.
pub(crate) struct TextRuns<'a> {
    pub(crate) element: ElementId,
    pub(crate) editable: &'a EditableText,
    /// The border box of the element.
    pub(crate) border_box: Rect,
    /// Takes a point of the element to the surface.
    pub(crate) on_surface: &'a Transform,
}

impl TextRuns<'_> {
    /// Gives the node of the element what an edited text has: the actions that move its
    /// selection and change its text, its placeholder, its runs and its selection; the nodes
    /// of the runs are added to `nodes`. Lines that were laid out for an earlier text are left
    /// out: the frame that lays the text out tells of them.
    pub(crate) fn describe(&self, node: &mut Node, nodes: &mut Vec<(NodeId, Node)>) {
        let editable = self.editable;
        for action in [Action::SetTextSelection, Action::ReplaceSelectedText, Action::SetValue] {
            node.add_action(action);
        }
        if !editable.placeholder.is_empty() {
            node.set_placeholder(editable.placeholder.as_str());
        }
        if !editable.has_current_lines() {
            return;
        }
        let spans = self.spans();
        let run_nodes: Vec<NodeId> = (0..spans.len()).map(|run| text_run_node(self.element, run)).collect();
        for (run, span) in spans.iter().enumerate() {
            let mut run_node = self.run(span);
            if span.continues {
                run_node.set_next_on_line(run_nodes[run + 1]);
            }
            if run > 0 && spans[run - 1].continues {
                run_node.set_previous_on_line(run_nodes[run - 1]);
            }
            nodes.push((run_nodes[run], run_node));
        }
        node.set_children(run_nodes);
        let selection = editable.editor.selection();
        node.set_text_selection(NodeSelection {
            anchor: self.position(&spans, selection.anchor, CaretAffinity::Downstream),
            focus: self.position(&spans, selection.head, editable.affinity()),
        });
    }

    /// The selection of the text that positions in the runs name; none when a position is in
    /// no run of this text, or the lines were laid out for an earlier text.
    pub(crate) fn selection_of(&self, selection: &NodeSelection) -> Option<TextSelection> {
        if !self.editable.has_current_lines() {
            return None;
        }
        let spans = self.spans();
        Some(TextSelection {
            anchor: self.offset_of(&spans, &selection.anchor)?,
            head: self.offset_of(&spans, &selection.focus)?,
        })
    }

    fn text(&self) -> &str {
        self.editable.editor.text()
    }

    /// The runs of the text, line by line. A line without characters has an empty run.
    fn spans(&self) -> Vec<RunSpan> {
        let layout = &self.editable.layout;
        let mut spans = Vec::new();
        for (line, caret_line) in layout.carets.lines.iter().enumerate() {
            let line_text = caret_line.text.as_range();
            let mut run_start = line_text.start;
            let mut run_characters = 0;
            for (start, _) in characters(&self.text()[line_text.clone()]) {
                if run_characters == MAXIMUM_RUN_CHARACTERS {
                    spans.push(RunSpan {
                        line,
                        text: run_start..line_text.start + start,
                        continues: true,
                    });
                    run_start = line_text.start + start;
                    run_characters = 0;
                }
                run_characters += 1;
            }
            spans.push(RunSpan {
                line,
                text: run_start..line_text.end,
                continues: false,
            });
        }
        spans
    }

    fn offset_of(&self, spans: &[RunSpan], position: &TextPosition) -> Option<usize> {
        let span = spans.get(run_of_node(self.element, position.node)?)?;
        let offset = characters(&self.text()[span.text.clone()])
            .nth(position.character_index)
            .map_or(span.text.end, |(start, _)| span.text.start + start);
        Some(offset)
    }

    /// The position of a place of the text: in the last run of its line that starts at or
    /// before it.
    fn position(&self, spans: &[RunSpan], offset: usize, affinity: CaretAffinity) -> TextPosition {
        let line = self.editable.layout.line_of(offset, affinity);
        let run = spans
            .iter()
            .rposition(|span| span.line == line && span.text.start <= offset)
            .expect("every line has a run that starts where the line starts");
        TextPosition {
            node: text_run_node(self.element, run),
            character_index: characters(&self.text()[spans[run].text.start..offset]).count(),
        }
    }

    /// The node of a run: its characters, the line break its line ends with among them, how
    /// many bytes each takes, where each stands in the run and where the words start.
    fn run(&self, span: &RunSpan) -> Node {
        let layout = &self.editable.layout;
        let line_end = layout.caret_range(span.line).end;
        let x_of = |offset: usize| layout.x_of(span.line, offset.min(line_end));
        let run_left = x_of(span.text.start);
        let run_text = &self.text()[span.text.clone()];
        let mut lengths = Vec::new();
        let mut positions = Vec::new();
        let mut widths = Vec::new();
        let mut word_starts = Vec::new();
        let mut words = run_text.unicode_word_indices().map(|(start, _)| start).peekable();
        for (character_index, (start, character)) in characters(run_text).enumerate() {
            let offset = span.text.start + start;
            let (left, right) = (x_of(offset), x_of(offset + character.len()));
            lengths.push(u8::try_from(character.len()).expect("a character of a run is told in a byte"));
            positions.push(left - run_left);
            widths.push(right - left);
            if words.next_if_eq(&start).is_some() {
                word_starts
                    .push(u8::try_from(character_index).expect("a run holds no more characters than a byte tells"));
            }
        }
        let run_box = self.on_surface.bounds_of(&Rect::new(
            self.border_box.x + layout.content_box.x + layout.line_left(span.line) + run_left - self.editable.scroll.x,
            self.border_box.y + layout.content_box.y + layout.line_top(span.line) - self.editable.scroll.y,
            x_of(span.text.end) - run_left,
            layout.line_height,
        ));
        let mut node = Node::new(Role::TextRun);
        node.set_value(run_text);
        node.set_text_direction(TextDirection::LeftToRight);
        node.set_character_lengths(lengths);
        node.set_character_positions(positions);
        node.set_character_widths(widths);
        node.set_word_starts(word_starts);
        node.set_bounds(NodeRect {
            x0: f64::from(run_box.x),
            y0: f64::from(run_box.y),
            x1: f64::from(run_box.right()),
            y1: f64::from(run_box.bottom()),
        });
        node
    }
}

#[cfg(test)]
mod tests {
    use accesskit::NodeId;

    use super::{characters, run_of_node, text_run_node};
    use crate::element_id::ElementId;

    #[test]
    fn the_node_of_a_run_names_its_element_and_its_run_and_is_no_node_of_an_element() {
        let element = ElementId::new(7, 3);

        let run = text_run_node(element, 2);

        assert_eq!(run_of_node(element, run), Some(2));
        assert_eq!(run_of_node(ElementId::new(8, 3), run), None);
        assert_eq!(run_of_node(element, NodeId::from(element)), None);
        assert_ne!(run, NodeId::from(element));
    }

    #[test]
    fn a_cluster_too_long_for_a_byte_is_stepped_through_by_code_points() {
        let long_cluster = format!("e{}", "\u{301}".repeat(200));
        let text = format!("a{long_cluster}b");

        let stepped: Vec<(usize, usize)> = characters(&text)
            .map(|(start, character)| (start, character.len()))
            .collect();

        assert_eq!(stepped.len(), 1 + 201 + 1);
        assert_eq!(stepped[0], (0, 1));
        assert_eq!(stepped[1], (1, 1));
        assert_eq!(stepped[2], (2, 2));
        assert_eq!(stepped[202], (1 + long_cluster.len(), 1));
    }
}
