use crate::makepadstorage::*;
use std::collections::HashMap;
use zaplib::*;
use zaplib_components::*;

#[derive(Clone, Default)]
pub struct SearchIndex {
    identifiers: TextIndex,
}

// search ordering
//
impl SearchIndex {
    pub fn new_rust_token(&mut self, mtb: &MakepadTextBuffer) {
        // pass it to the textindex
        if mtb.text_buffer.token_chunks.len() <= 1 {
            return;
        }
        let chunk_id = mtb.text_buffer.token_chunks.len() - 2;
        // lets figure out if its a decl, an impl or a use
        match mtb.text_buffer.token_chunks[chunk_id].token_type {
            TokenType::Identifier | TokenType::Call | TokenType::Macro | TokenType::TypeName => {
                let prev_tt = {
                    let mut i = if chunk_id > 0 { chunk_id - 1 } else { 0 };
                    loop {
                        let tt = mtb.text_buffer.token_chunks[i].token_type;
                        if i == 0 || !tt.should_ignore() {
                            break tt;
                        }
                        i -= 1;
                    }
                };
                let (next_tt, next_char) = {
                    let mut i = chunk_id + 1;
                    loop {
                        if i >= mtb.text_buffer.token_chunks.len() {
                            break (TokenType::Unexpected, '\0');
                        }
                        let tt = mtb.text_buffer.token_chunks[i].token_type;
                        if !tt.should_ignore() {
                            break (tt, mtb.text_buffer.flat_text[mtb.text_buffer.token_chunks[i].offset]);
                        }
                        i += 1;
                    }
                };
                let offset = mtb.text_buffer.token_chunks[chunk_id].offset;
                let len = mtb.text_buffer.token_chunks[chunk_id].len;
                let chars = &mtb.text_buffer.flat_text[offset..(offset + len)];
                let mut_id = (mtb.text_buffer.mutation_id & 0xffff) as u16;

                let prio = match mtb.text_buffer.token_chunks[chunk_id].token_type {
                    TokenType::Identifier => match prev_tt {
                        TokenType::Keyword => 1,
                        _ => 5,
                    },
                    TokenType::Call | TokenType::Macro => match prev_tt {
                        TokenType::Fn => 1,
                        _ => 5,
                    },
                    TokenType::TypeName => {
                        match prev_tt {
                            TokenType::TypeDef => 1,
                            TokenType::Impl => 2,
                            _ => {
                                // look at the next token
                                if next_tt == TokenType::Operator && next_char == '<' {
                                    3
                                } else if next_tt == TokenType::Namespace && next_char == ':' {
                                    5
                                } else {
                                    4
                                }
                            }
                        }
                    }
                    _ => 4,
                };

                self.identifiers.write(chars, mtb.text_buffer_id, mut_id, prio, chunk_id as u32);
            }
            _ => (),
        }
    }

    pub fn clear_markers(&mut self, cx: &mut Cx, storage: &mut MakepadStorage) {
        for atb in &mut storage.text_buffers {
            if !atb.text_buffer.markers.search_cursors.is_empty() {
                cx.send_signal(atb.text_buffer.signal, TextBuffer::STATUS_SEARCH_UPDATE);
            }
            atb.text_buffer.markers.search_cursors.truncate(0);
        }
    }

    pub fn search(
        &mut self,
        what: &str,
        first_tbid: MakepadTextBufferId,
        cx: &mut Cx,
        makepad_storage: &mut MakepadStorage,
    ) -> Vec<SearchResult> {
        let mut out = Vec::new();

        self.clear_markers(cx, makepad_storage);

        self.identifiers.search(what, first_tbid, makepad_storage, &mut out);

        // sort it
        out.sort_by(|a, b| {
            let prio = a.prio.cmp(&b.prio);
            if let std::cmp::Ordering::Equal = prio {
                let tb = a.text_buffer_id.cmp(&b.text_buffer_id);
                if let std::cmp::Ordering::Equal = tb {
                    return a.token.cmp(&b.token);
                }
                return tb;
            }
            prio
        });

        for atb in &mut makepad_storage.text_buffers {
            atb.text_buffer.markers.search_cursors.sort_by(|a, b| a.tail.cmp(&b.tail));
            if !atb.text_buffer.markers.search_cursors.is_empty() {
                cx.send_signal(atb.text_buffer.signal, TextBuffer::STATUS_SEARCH_UPDATE);
            }
        }

        out
    }
}

#[derive(Clone, Default)]
pub struct SearchResult {
    pub text_buffer_id: MakepadTextBufferId,
    prio: u16,
    pub token: u32,
}

#[derive(Clone, Default)]
struct TextIndexEntry {
    mut_id: u16,
    prio: u16,
}

#[derive(Clone, Default)]
struct TextIndexNode {
    stem: [char; 6],
    used: usize,
    map: HashMap<char, usize>,
    end: HashMap<(MakepadTextBufferId, u32), TextIndexEntry>,
}

#[derive(Clone)]
struct TextIndex {
    nodes: Vec<TextIndexNode>,
}

impl Default for TextIndex {
    fn default() -> Self {
        Self { nodes: vec![TextIndexNode::default()] }
    }
}

impl TextIndex {
    fn write(&mut self, what: &[char], text_buffer_id: MakepadTextBufferId, mut_id: u16, prio: u16, token: u32) {
        let mut o = 0;
        let mut id = 0;
        loop {
            if self.nodes[id].used > 0 {
                // we have a stem to compare/split
                for s in 0..self.nodes[id].used {
                    // read stem
                    let sc = self.nodes[id].stem[s];
                    if o >= what.len() || sc != what[o] {
                        // split
                        // lets make a new node and swap in our end/map
                        let new_id = self.nodes.len();
                        let mut new_end = HashMap::new();
                        let mut new_map = HashMap::new();
                        std::mem::swap(&mut new_end, &mut self.nodes[id].end);
                        std::mem::swap(&mut new_map, &mut self.nodes[id].map);
                        self.nodes.push(TextIndexNode { end: new_end, map: new_map, ..TextIndexNode::default() });
                        // copy whatever stem is left
                        for s2 in (s + 1)..self.nodes[id].used {
                            self.nodes[new_id].stem[s2 - s - 1] = self.nodes[id].stem[s2];
                        }
                        self.nodes[new_id].used = self.nodes[id].used - (s + 1);
                        self.nodes[id].used = s;
                        self.nodes[id].map.insert(sc, new_id);
                        break;
                    }
                    o += 1;
                }
            } else if self.nodes[id].end.is_empty() && self.nodes[id].map.is_empty() {
                // write stem
                for s in 0..self.nodes[id].stem.len() {
                    if o >= what.len() {
                        // we are done
                        break;
                    }
                    self.nodes[id].stem[s] = what[o];
                    self.nodes[id].used = s + 1;
                    o += 1;
                }
            }
            if o >= what.len() {
                // what is consumed
                break;
            }
            // jump/insert next node
            id = if let Some(next_id) = self.nodes[id].map.get(&what[o]) {
                o += 1;
                *next_id
            } else {
                let new_id = self.nodes.len();
                self.nodes.push(TextIndexNode::default());
                self.nodes[id].map.insert(what[o], new_id);
                o += 1;
                new_id
            };
        }

        self.nodes[id].end.insert((text_buffer_id, token), TextIndexEntry { mut_id, prio });
    }

    fn search(
        &mut self,
        what: &str,
        first_tbid: MakepadTextBufferId,
        makepad_storage: &mut MakepadStorage,
        out: &mut Vec<SearchResult>,
    ) {
        // ok so if i type a beginning of a word, i'd want all the endpoints

        let mut node_id = 0;
        let mut stem_eat = 0;
        let mut exact_only = true;
        for c in what.chars() {
            if c == '*' {
                exact_only = false;
                continue;
            }

            // first eat whats in the stem
            if stem_eat < self.nodes[node_id].used {
                if c != self.nodes[node_id].stem[stem_eat] {
                    return;
                }
                stem_eat += 1;
            } else {
                node_id = if let Some(next_id) = self.nodes[node_id].map.get(&c) {
                    stem_eat = 0;
                    *next_id
                } else {
                    // nothing
                    return;
                };
            }
        }
        if exact_only && stem_eat < self.nodes[node_id].used {
            return;
        }

        let mut nexts = Vec::new();
        let mut cleanup = Vec::new();

        loop {
            for next in self.nodes[node_id].map.values() {
                nexts.push(*next);
            }
            cleanup.truncate(0);
            for ((text_buffer_id, token), entry) in &self.nodes[node_id].end {
                let tb = &mut makepad_storage.text_buffers[text_buffer_id.as_index()].text_buffer;
                if (tb.mutation_id & 0xffff) as u16 == entry.mut_id {
                    out.push(SearchResult {
                        text_buffer_id: *text_buffer_id,
                        token: *token,
                        prio: if entry.prio == 1 {
                            if *text_buffer_id == first_tbid {
                                0
                            } else {
                                1
                            }
                        } else {
                            entry.prio
                        },
                    });
                    // lets output a result cursor int he textbuffer
                    let tok = &tb.token_chunks[*token as usize];
                    tb.markers.search_cursors.push(TextCursor { head: tok.offset + tok.len, tail: tok.offset, max: 0 });
                } else {
                    cleanup.push((*text_buffer_id, *token));
                }
            }
            if exact_only {
                break;
            }
            for pair in &cleanup {
                self.nodes[node_id].end.remove(pair);
            }
            if nexts.is_empty() {
                break;
            }
            node_id = nexts.pop().unwrap();
        }
    }
}
