//! Apply Monaco-style (line, column) 1-based ranges to a UTF-8 string.
//! Columns count Unicode scalar values within the line (good enough for typical code).

use crate::ws::messages::TextChangeDelta;

/// Apply a single edit. Returns `Err` if range is invalid for `content`.
pub fn apply_text_delta(content: &mut String, delta: &TextChangeDelta) -> Result<(), ()> {
  let start = offset_for_position(content, delta.range.start_line, delta.range.start_column).ok_or(())?;
  let end = offset_for_position(content, delta.range.end_line, delta.range.end_column).ok_or(())?;
  if start > end || end > content.len() {
    return Err(());
  }
  content.replace_range(start..end, &delta.text);
  Ok(())
}

fn line_start_byte(s: &str, line: u32) -> Option<usize> {
  if line < 1 {
    return None;
  }
  if line == 1 {
    return Some(0);
  }
  let mut current = 1u32;
  for (i, ch) in s.char_indices() {
    if ch == '\n' {
      current += 1;
      if current == line {
        return Some(i + 1);
      }
    }
  }
  None
}

fn offset_for_position(s: &str, line: u32, col: u32) -> Option<usize> {
  if col < 1 {
    return None;
  }
  let line_start = line_start_byte(s, line)?;
  let rest = s.get(line_start..)?;
  let mut col_idx = 1u32;
  if col == 1 {
    return Some(line_start);
  }
  for (i, ch) in rest.char_indices() {
    if ch == '\n' {
      break;
    }
    col_idx += 1;
    if col_idx == col {
      return Some(line_start + i + ch.len_utf8());
    }
  }
  // Cursor after last character on line (valid in Monaco)
  let line_text = rest.split('\n').next().unwrap_or("");
  if col_idx == col {
    return Some(line_start + line_text.len());
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ws::messages::EditorRange;

  #[test]
  fn insert_at_start() {
    let mut s = String::from("hello");
    let d = TextChangeDelta {
      range: EditorRange {
        start_line: 1,
        start_column: 1,
        end_line: 1,
        end_column: 1,
      },
      text: "X".into(),
      version: 0,
    };
    apply_text_delta(&mut s, &d).unwrap();
    assert_eq!(s, "Xhello");
  }

  #[test]
  fn replace_mid() {
    let mut s = String::from("ab\ncd");
    let d = TextChangeDelta {
      range: EditorRange {
        start_line: 1,
        start_column: 2,
        end_line: 1,
        end_column: 3,
      },
      text: "z".into(),
      version: 0,
    };
    apply_text_delta(&mut s, &d).unwrap();
    assert_eq!(s, "az\ncd");
  }
}
