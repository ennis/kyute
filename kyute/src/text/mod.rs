// Related reading:
// https://manishearth.github.io/blog/2017/01/14/stop-ascribing-meaning-to-unicode-code-points/
// https://unicode.org/reports/tr29/

use kyute_shell::drawing::Point;
use std::cell::Cell;
use kyute_shell::text::TextLayout;

// Persistent text editor state:
// - the text being edited
// - the text layout
// - the selection range: two positions, each pos falls between grapheme cluster boundaries
// - the cursor position

// when receiving an input event
// - character: insert character, update cursor pos, request relayout and redraw

