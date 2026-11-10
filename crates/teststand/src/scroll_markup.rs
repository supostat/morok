use std::fmt::Write;

use morok_core::{ElementId, Role};

use crate::scene::Scene;

/// Marks a `Scroll` element by its place in document order; its thumbs name it by that place.
const SCROLLER_ATTRIBUTE: &str = "data-scroller";
/// The offset a `Scroll` element is scrolled to, across and down.
const SCROLL_LEFT_ATTRIBUTE: &str = "data-scroll-left";
const SCROLL_TOP_ATTRIBUTE: &str = "data-scroll-top";
/// Marks the thumb of a scroll bar with the place of its `Scroll` element.
const THUMB_ATTRIBUTE: &str = "data-thumb-of";
/// `vertical` or `horizontal`.
const THUMB_AXIS_ATTRIBUTE: &str = "data-thumb-axis";

/// Native scroll bars take no room and draw nothing: the bars of the kit are elements.
const SCROLL_STYLE: &str = "* { scrollbar-width: none; }";

/// Scrolls every `Scroll` element to its offset, then sizes and moves the thumbs of the kit by
/// the rule the kit sizes and moves them: as long as the share of the content the viewport
/// shows, never shorter than `SHORTEST_THUMB`, along the content box of the bar.
const SCROLL_SCRIPT: &str = "for (const scroller of document.querySelectorAll('[SCROLLER_ATTRIBUTE]')) {\n\
    scroller.scrollLeft = Number(scroller.getAttribute('SCROLL_LEFT_ATTRIBUTE'));\n\
    scroller.scrollTop = Number(scroller.getAttribute('SCROLL_TOP_ATTRIBUTE'));\n\
    }\n\
    for (const thumb of document.querySelectorAll('[THUMB_ATTRIBUTE]')) {\n\
    const scroller = document.querySelector(`[SCROLLER_ATTRIBUTE=\"${thumb.getAttribute('THUMB_ATTRIBUTE')}\"]`);\n\
    const vertical = thumb.getAttribute('THUMB_AXIS_ATTRIBUTE') === 'vertical';\n\
    const bar = getComputedStyle(thumb.parentElement);\n\
    const track = vertical\n\
    ? thumb.parentElement.clientHeight - parseFloat(bar.paddingTop) - parseFloat(bar.paddingBottom)\n\
    : thumb.parentElement.clientWidth - parseFloat(bar.paddingLeft) - parseFloat(bar.paddingRight);\n\
    const viewport = vertical ? scroller.clientHeight : scroller.clientWidth;\n\
    const limit = vertical ? scroller.scrollHeight - scroller.clientHeight : scroller.scrollWidth - scroller.clientWidth;\n\
    const offset = vertical ? scroller.scrollTop : scroller.scrollLeft;\n\
    let length = 0;\n\
    let travel = 0;\n\
    if (limit > 0) {\n\
    length = Math.min(Math.max(track * viewport / (viewport + limit), SHORTEST_THUMB), track);\n\
    travel = (track - length) * Math.min(Math.max(offset / limit, 0), 1);\n\
    }\n\
    thumb.style[vertical ? 'height' : 'width'] = `${length}px`;\n\
    thumb.style.transform = vertical ? `translateY(${travel}px)` : `translateX(${travel}px)`;\n\
    }";

/// The shortest a thumb of the kit gets.
const SHORTEST_THUMB: &str = "24";

/// What the markup of an element carries of scrolling: a `Scroll` element its place and the
/// offset the case scrolls it to; a thumb the place of its `Scroll` element and its axis.
pub(crate) fn scroll_attributes(scene: &Scene, element: ElementId) -> String {
    let tree = scene.tree();
    let mut attributes = String::new();
    for (place, scroller) in scene.elements().into_iter().enumerate() {
        if tree.role(scroller) != Role::Scroll {
            continue;
        }
        if scroller == element {
            let offset = scene.scroll_offset_asked(scroller);
            let _ = write!(
                attributes,
                " {SCROLLER_ATTRIBUTE}=\"{place}\" {SCROLL_LEFT_ATTRIBUTE}=\"{}\" {SCROLL_TOP_ATTRIBUTE}=\"{}\"",
                offset.x, offset.y
            );
        }
        if let Some(bars) = tree.scroll_bars(scroller) {
            for (thumb, axis) in [(bars.vertical, "vertical"), (bars.horizontal, "horizontal")] {
                if thumb == Some(element) {
                    let _ = write!(
                        attributes,
                        " {THUMB_ATTRIBUTE}=\"{place}\" {THUMB_AXIS_ATTRIBUTE}=\"{axis}\""
                    );
                }
            }
        }
    }
    attributes
}

/// The style and the script the page of a tree with a `Scroll` element needs; none for a
/// tree without one.
pub(crate) fn scroll_support(scene: &Scene) -> Option<String> {
    let tree = scene.tree();
    let scrolls = scene
        .elements()
        .into_iter()
        .any(|element| tree.role(element) == Role::Scroll);
    if !scrolls {
        return None;
    }
    let script = SCROLL_SCRIPT
        .replace("SCROLLER_ATTRIBUTE", SCROLLER_ATTRIBUTE)
        .replace("SCROLL_LEFT_ATTRIBUTE", SCROLL_LEFT_ATTRIBUTE)
        .replace("SCROLL_TOP_ATTRIBUTE", SCROLL_TOP_ATTRIBUTE)
        .replace("THUMB_AXIS_ATTRIBUTE", THUMB_AXIS_ATTRIBUTE)
        .replace("THUMB_ATTRIBUTE", THUMB_ATTRIBUTE)
        .replace("SHORTEST_THUMB", SHORTEST_THUMB);
    Some(format!(
        "<style>\n{SCROLL_STYLE}\n</style><script>\n{script}\n</script>"
    ))
}
