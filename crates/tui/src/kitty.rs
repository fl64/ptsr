//! Kitty graphics protocol: the wave as a picture under the text. The encoder builds
//! the escape commands, the rasteriser turns wave peaks into RGBA pixels, [`Shown`]
//! keeps what the terminal already shows so only changes are sent. Braille stays the
//! fallback for terminals without the protocol.

use std::collections::{HashMap, HashSet};

use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Color;

/// Base64 payload cap of one graphics command; the protocol chunks commands at 4096 bytes.
const CHUNK: usize = 4096;

/// A wave drawn as a picture instead of Braille: what it shows, where it goes.
pub struct WaveImage {
    /// Fixed id of the area: the chop wave is 1.
    pub id: u32,
    /// The cells the picture covers — the area where the Braille wave would be.
    pub area: Rect,
    /// Peak of every Braille dot column (two per cell), 0..1.
    pub peaks: Vec<f32>,
    /// One colour per cell column, the same the Braille wave of this area uses.
    pub colors: Vec<Color>,
}

/// What the terminal shows now: the pictures sent to it and the cells they were placed
/// at. `update` turns the wanted pictures into the commands that bring the terminal
/// there; without graphics support the state stays empty and nothing is ever sent.
#[derive(Default)]
pub struct Shown {
    /// Per area id: the picture it shows, the image id that holds it now, where it sits.
    sent: HashMap<u32, Vec<u8>>,
    image: HashMap<u32, u32>,
    placed: HashMap<u32, Rect>,
}

/// Every area has two image ids, so a changed picture goes over the old one before the
/// old one is taken off: deleting first left the screen empty while the new one went out.
fn spare(area: u32, now: Option<u32>) -> u32 {
    let first = area * 2;
    if now == Some(first) { first + 1 } else { first }
}

impl Shown {
    /// The commands to show `want`: transmit and place the pictures that changed, place
    /// the ones that moved, delete the ones whose area is not on the screen.
    pub fn update(&mut self, want: &[WaveImage], cell_px: (u16, u16)) -> Vec<String> {
        let mut cmds = Vec::new();
        let mut kept = HashSet::new();
        for img in want {
            if img.area.width == 0 || img.area.height == 0 || cell_px.0 == 0 || cell_px.1 == 0 {
                continue;
            }
            kept.insert(img.id);
            let (px_w, px_h, rgba) = rasterise(img, cell_px);
            let now = self.image.get(&img.id).copied();
            if self.sent.get(&img.id) == Some(&rgba) {
                // the same picture: place it again only if it moved; p=1 moves the one placement
                if self.placed.get(&img.id) != Some(&img.area)
                    && let Some(id) = now
                {
                    cmds.push(place(id, img.area));
                    self.placed.insert(img.id, img.area);
                }
                continue;
            }
            let id = spare(img.id, now);
            cmds.extend(transmit(id, px_w, px_h, &rgba));
            cmds.push(place(id, img.area));
            if let Some(old) = now {
                cmds.push(delete(old));
            }
            self.sent.insert(img.id, rgba);
            self.image.insert(img.id, id);
            self.placed.insert(img.id, img.area);
        }
        for area in self.sent.keys().copied().collect::<Vec<_>>() {
            if !kept.contains(&area) {
                if let Some(id) = self.image.remove(&area) {
                    cmds.push(delete(id));
                }
                self.sent.remove(&area);
                self.placed.remove(&area);
            }
        }
        cmds
    }

    /// Remove intersecting pictures before text draws. The next unobstructed update restores them.
    pub fn suspend(&mut self, blockers: &[Rect]) -> Vec<String> {
        let mut areas = self
            .placed
            .iter()
            .filter(|(_, area)| {
                blockers.iter().any(|blocker| {
                    blocker.width > 0 && blocker.height > 0 && area.intersects(*blocker)
                })
            })
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        areas.sort_unstable();
        let mut commands = Vec::new();
        for area in areas {
            if let Some(id) = self.image.remove(&area) {
                commands.push(delete(id));
            }
            self.sent.remove(&area);
            self.placed.remove(&area);
        }
        commands
    }

    /// The commands that take every shown picture off the screen; the state forgets them.
    pub fn drop_all(&mut self) -> Vec<String> {
        let ids: Vec<u32> = self.image.values().copied().collect();
        self.sent.clear();
        self.image.clear();
        self.placed.clear();
        ids.iter().map(|&id| delete(id)).collect()
    }
}

/// The first key event of a graphics reply: crossterm has no APC events, so the
/// terminal's `_G…` answer surfaces as `Alt+_`.
pub fn is_graphics_ack(ev: &Event) -> bool {
    matches!(
        ev,
        Event::Key(k) if k.modifiers.contains(KeyModifiers::ALT) && k.code == KeyCode::Char('_')
    )
}

/// Base64 of `data`: the standard alphabet, padded.
fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for c in data.chunks(3) {
        let n = u32::from(c[0]) << 16
            | u32::from(c.get(1).copied().unwrap_or(0)) << 8
            | u32::from(c.get(2).copied().unwrap_or(0));
        out.push(T[(n >> 18) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(if c.len() > 1 {
            T[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if c.len() > 2 {
            T[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// The commands that transmit `rgba` as the picture `id` without showing it: direct RGBA
/// in base64 chunks of at most 4096 bytes, no replies; showing goes through [`place`].
fn transmit(id: u32, px_w: u32, px_h: u32, rgba: &[u8]) -> Vec<String> {
    let b64 = base64(rgba);
    let total = b64.len().div_ceil(CHUNK).max(1);
    (0..total)
        .map(|k| {
            let part = &b64[k * CHUNK..((k + 1) * CHUNK).min(b64.len())];
            // only the first chunk carries the image keys; m=1 marks all but the last
            let keys = if k == 0 {
                format!(
                    "a=t,f=32,t=d,i={id},s={px_w},v={px_h},q=2,C=1,z=-1,m={}",
                    u8::from(k + 1 != total)
                )
            } else {
                format!("m={},q=2", u8::from(k + 1 != total))
            };
            format!("\x1b_G{keys};{part}\x1b\\")
        })
        .collect()
}

/// The commands that put the picture `id` with its top-left corner at that cell: the
/// cursor walks there and comes back (`x=`/`y=` in a show command would crop the source,
/// not place it), the placement is numbered, so a repeat with the same id and placement
/// moves the old picture instead of adding a copy.
fn place(id: u32, area: Rect) -> String {
    format!(
        "\x1b7\x1b[{};{}H\x1b_Ga=p,i={id},p=1,q=2,C=1,z=-1\x1b\\\x1b8",
        area.y + 1,
        area.x + 1
    )
}

/// The command that takes the picture `id` and everything shown of it off the screen.
fn delete(id: u32) -> String {
    format!("\x1b_Ga=d,d=i,i={id},q=2\x1b\\")
}

/// The RGBA picture for `img` at `cell_px` pixels per cell: every pixel column takes the
/// peak of its dot column, mirrored around the middle, in its cell column's colour; the
/// rest is transparent. Like the Braille wave, silence still reads as a line.
fn rasterise(img: &WaveImage, cell_px: (u16, u16)) -> (u32, u32, Vec<u8>) {
    let px_w = u32::from(img.area.width) * u32::from(cell_px.0);
    let px_h = u32::from(img.area.height) * u32::from(cell_px.1);
    let mut rgba = vec![0u8; px_w as usize * px_h as usize * 4];
    if px_w == 0 || px_h == 0 || img.peaks.is_empty() || img.colors.is_empty() {
        return (px_w, px_h, rgba);
    }
    let mid = px_h as usize / 2;
    for x in 0..px_w as usize {
        let Some((r, g, b)) = rgb(img.colors[x * img.colors.len() / px_w as usize]) else {
            continue;
        };
        let peak = img.peaks[x * img.peaks.len() / px_w as usize];
        let half = ((peak * px_h as f32 / 2.0).round() as usize).clamp(1, mid.max(1));
        for y in mid.saturating_sub(half)..(mid + half).min(px_h as usize) {
            let i = (y * px_w as usize + x) * 4;
            rgba[i..i + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
    (px_w, px_h, rgba)
}

/// The RGB of a style colour; pictures only get `Rgb` colours, the rest stays blank.
fn rgb(c: Color) -> Option<(u8, u8, u8)> {
    match c {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encodes_the_standard_alphabet() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn transmit_chunks_at_4096_bytes() {
        // a small picture is one command with the image keys
        assert_eq!(
            transmit(7, 4, 2, &[0u8; 12]),
            vec!["\x1b_Ga=t,f=32,t=d,i=7,s=4,v=2,q=2,C=1,z=-1,m=0;AAAAAAAAAAAAAAAA\x1b\\"]
        );
        // 3100 bytes of RGBA are 4136 base64 bytes: two chunks, every one at most 4096
        let rgba = vec![0xAA; 3100];
        let cmds = transmit(9, 10, 10, &rgba);
        assert_eq!(cmds.len(), 2);
        assert!(cmds[0].starts_with("\x1b_Ga=t,f=32,t=d,i=9,s=10,v=10,q=2,C=1,z=-1,m=1;"));
        // transmission never shows: a=T would put a second copy at the cursor
        assert!(cmds.iter().all(|c| !c.contains("a=T")));
        assert_eq!(payload(&cmds[0]).len(), 4096);
        assert!(cmds[1].starts_with("\x1b_Gm=0,q=2;"));
        assert_eq!(payload(&cmds[1]).len(), 40);
        let mut joined = payload(&cmds[0]).to_owned();
        joined.push_str(payload(&cmds[1]));
        assert_eq!(joined, base64(&rgba));
    }

    /// The base64 payload of a graphics command: between the first `;` and the terminator.
    fn payload(cmd: &str) -> &str {
        cmd.split(';')
            .nth(1)
            .unwrap()
            .strip_suffix("\x1b\\")
            .unwrap()
    }

    #[test]
    fn place_and_delete_name_the_id_and_the_cells() {
        assert_eq!(
            place(3, Rect::new(10, 5, 4, 2)),
            "\x1b7\x1b[6;11H\x1b_Ga=p,i=3,p=1,q=2,C=1,z=-1\x1b\\\x1b8"
        );
        assert_eq!(delete(3), "\x1b_Ga=d,d=i,i=3,q=2\x1b\\");
        assert!(Shown::default().drop_all().is_empty());
    }

    #[test]
    fn rasterise_turns_peaks_into_pixel_columns() {
        let img = WaveImage {
            id: 1,
            area: Rect::new(0, 0, 2, 1),
            peaks: vec![1.0, 1.0, 0.5, 0.0],
            colors: vec![Color::Rgb(10, 20, 30), Color::Rgb(40, 50, 60)],
        };
        let (w, h, rgba) = rasterise(&img, (4, 8));
        assert_eq!((w, h), (8, 8));
        let px = |x: usize, y: usize| {
            let i = (y * 8 + x) * 4;
            (rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3])
        };
        // the loud column fills the height in its colour
        assert_eq!(px(0, 0), (10, 20, 30, 255));
        assert_eq!(px(0, 7), (10, 20, 30, 255));
        // the half column keeps the middle rows only
        assert_eq!(px(4, 1), (0, 0, 0, 0));
        assert_eq!(px(4, 3), (40, 50, 60, 255));
        // silence keeps just the middle line
        assert_eq!(px(6, 0), (0, 0, 0, 0));
        assert_eq!(px(6, 3), (40, 50, 60, 255));
        assert_eq!(px(6, 5), (0, 0, 0, 0));
    }

    fn img(id: u32, peak: f32, y: u16) -> WaveImage {
        WaveImage {
            id,
            area: Rect::new(0, y, 4, 2),
            peaks: vec![peak; 8],
            colors: vec![Color::Rgb(1, 2, 3); 4],
        }
    }

    /// Scenario "The picture does not blink" (chopping).
    #[test]
    fn update_sends_only_what_changed() {
        let cell = (4, 8);
        let mut shown = Shown::default();
        // the first frame transmits and places under the area's first id (area 1: 2)
        let cmds = shown.update(&[img(1, 1.0, 3)], cell);
        assert_eq!(cmds.len(), 2);
        assert!(cmds[0].contains("a=t,f=32,t=d,i=2"));
        assert!(cmds[1].contains("\x1b7\x1b[4;1H"));
        assert!(cmds[1].contains("a=p,i=2,p=1"));
        assert!(cmds[1].ends_with("\x1b8"));
        assert!(!cmds[1].contains("x=") && !cmds[1].contains("y="));
        // the same picture again: the terminal already shows it
        assert!(shown.update(&[img(1, 1.0, 3)], cell).is_empty());
        // the picture moved: placed again, nothing transmitted, nothing deleted
        let cmds = shown.update(&[img(1, 1.0, 5)], cell);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("\x1b[6;1H") && cmds[0].contains("a=p,i=2,p=1"));
        // 04b "The picture does not blink": the new picture goes out under the spare id
        // and is placed before the old one is deleted
        let cmds = shown.update(&[img(1, 0.5, 5)], cell);
        assert_eq!(cmds.len(), 3);
        assert!(cmds[0].contains("a=t,f=32,t=d,i=3"));
        assert!(cmds[1].contains("a=p,i=3,p=1"));
        assert!(cmds[2].contains("a=d,d=i,i=2"), "the old one goes last");
        // and back to the first id on the next change
        let cmds = shown.update(&[img(1, 0.7, 5)], cell);
        assert!(cmds[0].contains("i=2") && cmds[2].contains("a=d,d=i,i=3"));
        // the area is gone: its picture is deleted
        let cmds = shown.update(&[], cell);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("a=d,d=i,i=2"));
        // back on the screen: transmitted again
        assert!(
            shown
                .update(&[img(1, 0.5, 5)], cell)
                .iter()
                .any(|c| c.contains("a=t,f=32"))
        );
        // a zero cell size means nothing is known about pixels: the picture is taken off
        let cmds = shown.update(&[img(1, 0.5, 5)], (0, 8));
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("a=d,d=i"));
    }

    #[test]
    fn overlays_remove_only_intersecting_images_before_text() {
        let mut shown = Shown::default();
        shown.update(&[img(1, 1.0, 3), img(2, 0.5, 8)], (4, 8));
        assert!(
            shown.suspend(&[Rect::new(4, 3, 2, 2)]).is_empty(),
            "touching edges do not overlap"
        );
        assert!(shown.suspend(&[Rect::new(2, 3, 0, 2)]).is_empty());
        let hidden = shown.suspend(&[Rect::new(3, 4, 2, 2), Rect::new(0, 3, 4, 1)]);
        assert_eq!(
            hidden,
            vec![delete(2)],
            "remove the whole intersecting image once"
        );
        assert!(shown.suspend(&[Rect::new(0, 3, 4, 2)]).is_empty());
        assert!(
            shown.update(&[img(2, 0.5, 8)], (4, 8)).is_empty(),
            "the other image never blinks"
        );
        let restored = shown.update(&[img(1, 0.7, 3), img(2, 0.5, 8)], (4, 8));
        assert!(restored[0].contains("a=t,f=32,t=d,i=2"));
        assert!(restored[1].contains("a=p,i=2"));
        assert_eq!(
            restored.len(),
            2,
            "restore current data without touching the other image"
        );
    }

    #[test]
    fn the_graphics_answer_surfaces_as_alt_underscore() {
        let alt_underscore = Event::Key(ratatui::crossterm::event::KeyEvent::new(
            KeyCode::Char('_'),
            KeyModifiers::ALT,
        ));
        assert!(is_graphics_ack(&alt_underscore));
        let plain = Event::Key(ratatui::crossterm::event::KeyEvent::new(
            KeyCode::Char('g'),
            KeyModifiers::NONE,
        ));
        assert!(!is_graphics_ack(&plain));
        let no_alt = Event::Key(ratatui::crossterm::event::KeyEvent::new(
            KeyCode::Char('_'),
            KeyModifiers::NONE,
        ));
        assert!(!is_graphics_ack(&no_alt));
    }
}
