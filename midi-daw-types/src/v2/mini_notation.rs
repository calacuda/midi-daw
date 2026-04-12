use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

type MnNote = String;

#[derive(
    Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug,
)]
pub enum MnStepType {
    Chord(Vec<MnNote>),
    Note(MnNote),
    Rest,
}

#[derive(
    Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug,
)]
enum MnTokenType {
    Alternator(usize, Vec<MnTokenType>, bool),
    Chord(Vec<MnTokenType>),
    Note(String),
    Rest,
}

#[derive(
    Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug,
)]
pub struct Parser {
    full_text: String,
    full_i: (usize, usize),
    notes: Vec<MnTokenType>,
    i: usize,
    total_steps: usize,
}

impl Parser {
    pub fn new(full_text: impl ToString) -> Self {
        let full_text = full_text.to_string();

        Self {
            full_i: (0, full_text.len()),
            full_text,
            notes: Vec::new(),
            i: 0,
            total_steps: 0,
        }
    }

    fn do_parse_pattern(
        &mut self,
        close: &str,
        to_parse: &str,
        depth: usize,
    ) -> (Vec<MnTokenType>, String) {
        let mut tokens = Vec::new();

        let mut yet_to_parse = to_parse.to_string();

        loop {
            // println!("do_parse_pattern 1 => {token}, {to_parse}");

            let (mut token, mut the_rest) = yet_to_parse
                .split_once(" ")
                .map(|(tok, rest)| (tok.to_string(), Some(rest.into())))
                .unwrap_or((yet_to_parse.to_string(), None));
            // token = tmp_token;
            trace!("[depth = {depth}]: do_parse_pattern 2 => {token}, {the_rest:?}");

            if token.starts_with("<") {
                yet_to_parse = yet_to_parse.replacen("<", "", 1);
                trace!("[depth = {depth}]: do_parse_pattern before alternator");
                trace!("[depth = {depth}]: yet_to_parse = {yet_to_parse}");
                let (loc_tokens, loc_the_rest) = self.do_parse_pattern(
                    // "<",
                    ">",
                    // |tokens| MnTokenType::Alternator(0, tokens),
                    // &mut loc_tokens,
                    // token.clone(),
                    // the_rest,
                    &yet_to_parse,
                    depth + 1,
                );

                // trace!("[depth = {depth}]: seq the rest {loc_the_rest}");
                // println!("[depth = {depth}]: seq the rest {loc_the_rest}, ytp: {yet_to_parse}");

                if let Some((n, loc_the_rest)) = loc_the_rest.strip_prefix("*").map(|s| {
                    s.split_once(" ")
                        .map(|(tok, rest)| (tok.to_string(), Some(rest.to_string())))
                        .unwrap_or((s.to_string(), None))
                }) {
                    let n = n.parse::<usize>().unwrap_or(1);

                    for _ in 0..n {
                        tokens.append(&mut loc_tokens.clone());
                    }

                    self.total_steps += loc_tokens
                        .iter()
                        .map(|token| match token.clone() {
                            MnTokenType::Rest => 1,
                            MnTokenType::Note(_) => 1,
                            MnTokenType::Chord(_) => 1,
                            MnTokenType::Alternator(_, notes, _) => notes.len(),
                        })
                        .sum::<usize>()
                        * (n - 1);

                    the_rest = loc_the_rest;
                } else {
                    // self.total_steps += loc_tokens.len();
                    tokens.push(MnTokenType::Alternator(0, loc_tokens.clone(), false));
                    self.total_steps += loc_tokens
                        .iter()
                        .map(|token| match token.clone() {
                            MnTokenType::Rest => 1,
                            MnTokenType::Note(_) => 1,
                            MnTokenType::Chord(_) => 1,
                            MnTokenType::Alternator(_, notes, _) => notes.len(),
                        })
                        .sum::<usize>();
                }
                // the_rest = Some(loc_the_rest);
                yet_to_parse = loc_the_rest;
                trace!(
                    "[depth = {depth}]: do_parse_pattern after Alternator => ({tokens:?}, {yet_to_parse:?})"
                );
                // break;

                // } else if token.ends_with(">") {
            } else if token.starts_with("[") {
                // if token.contains("]*") {
                // } else if token.ends_with("]") {
                // }
                trace!("[depth = {depth}]: do_parse_pattern before chord");
                yet_to_parse = yet_to_parse.replacen("[", "", 1);
                let (loc_tokens, loc_the_rest) = self.do_parse_pattern(
                    // "[",
                    "]",
                    // MnTokenType::Chord,
                    // &mut loc_tokens,
                    // token.clone(),
                    // the_rest,
                    &yet_to_parse,
                    depth + 1,
                );
                // loc_tokens.append(&mut tokens);
                // token = loc_token.clone();
                // tokens.push(MnTokenType::Chord(loc_tokens.clone()));

                if let Some((n, loc_the_rest)) = loc_the_rest.strip_prefix("*").map(|s| {
                    s.split_once(" ")
                        .map(|(tok, rest)| (tok.to_string(), Some(rest.to_string())))
                        .unwrap_or((s.to_string(), None))
                }) {
                    let n = n.parse::<usize>().unwrap_or(1);

                    for _ in 0..n {
                        // tokens.append(&mut loc_tokens.clone());
                        tokens.push(MnTokenType::Chord(loc_tokens.clone()));
                    }

                    // self.total_steps += loc_tokens
                    //     .iter()
                    //     .map(|token| match token.clone() {
                    //         MnTokenType::Rest => 1,
                    //         MnTokenType::Note(_) => 1,
                    //         MnTokenType::Chord(_) => 1,
                    //         MnTokenType::Alternator(_, notes) => notes.len(),
                    //     })
                    //     .sum::<usize>()
                    //     * (n - 1);

                    the_rest = loc_the_rest;
                } else {
                    // self.total_steps += loc_tokens.len();
                    tokens.push(MnTokenType::Chord(loc_tokens.clone()));
                    // self.total_steps += loc_tokens
                    //     .iter()
                    //     .map(|token| match token.clone() {
                    //         MnTokenType::Rest => 1,
                    //         MnTokenType::Note(_) => 1,
                    //         MnTokenType::Chord(_) => 1,
                    //         MnTokenType::Alternator(_, notes) => notes.len(),
                    //     })
                    //     .sum::<usize>();
                }

                // self.total_steps += 1;
                // the_rest = Some(loc_the_rest);
                yet_to_parse = loc_the_rest;
                trace!(
                    "[depth = {depth}]: do_parse_pattern 2 (after chord) => ({tokens:?}, {yet_to_parse:?})"
                );
                trace!("[depth = {depth}]: loc_tokens :  {loc_tokens:?}");
                // break;
            } else if token.contains(" ") {
                // let mut parsed_token = self.do_parse(token.as_str());
                // tokens.append(&mut parsed_token);
                trace!("[depth = {depth}]: test print {token}");
                // let mut tmp_tokens = Vec::new();
                // self.do_parse(token.as_str(), &mut tmp_tokens);
                // loc_tokens.append(&mut tmp_tokens);
                let (mut loc_tokens, loc_the_rest) =
                    self.do_parse_pattern(close, token.as_str(), depth + 1);
                trace!("[depth = {depth}]: cleaned token = {close} | {token} | {the_rest:?}");
                the_rest = Some(loc_the_rest.clone());
                tokens.append(&mut loc_tokens);
                yet_to_parse = loc_the_rest;
            } else if token.contains(close) && !close.is_empty() {
                // token = token.replacen(close, "", 1);
                let old_the_rest = the_rest.clone();
                (token, the_rest) = token
                    .split_once(close)
                    .map(|(tok, rest)| {
                        trace!("[depth = {depth}]: tmp_rest: {rest:?}");
                        (tok.to_string(), Some(rest.to_string()))
                    })
                    .unwrap_or((token.replacen(close, "", 1), None));
                trace!(
                    "[depth = {depth}]: cleaned token = {close} | {token} | {the_rest:?} | {old_the_rest:?}"
                );
                // println!(
                //     "[depth = {depth}]: cleaned token = {token} | {close} | {the_rest:?} | {old_the_rest:?}"
                // );
                // loc_tokens.append(&mut self.do_parse(token.as_str()));
                // self.do_parse(token.as_str(), &mut loc_tokens);

                // loc_tokens.append(&mut self.do_parse(token.as_str()));
                // loc_tokens.push(mk_token(loc_tokens.clone()));

                let (mut loc_tokens, loc_the_rest) =
                    self.do_parse_pattern(close, token.as_str(), depth + 1);
                trace!("[depth = {depth}]: close not empty: {loc_the_rest} vs {the_rest:?}");
                // println!("[depth = {depth}]: close not empty: {loc_the_rest} vs {the_rest:?}");
                // println!("[depth = {depth}]: close not empty: {loc_the_rest} vs {the_rest:?}");
                // the_rest = Some(loc_the_rest.clone());
                tokens.append(&mut loc_tokens);

                // self.total_steps += loc_tokens
                //     .iter()
                //     .map(|token| match token.clone() {
                //         MnTokenType::Rest => 1,
                //         MnTokenType::Note(_) => 1,
                //         MnTokenType::Chord(_) => 1,
                //         MnTokenType::Alternator(_, notes) => notes.len(),
                //     })
                //     .sum::<usize>();

                // yet_to_parse = loc_the_rest;
                // yet_to_parse = the_rest.unwrap_or(String::new());
                yet_to_parse = old_the_rest.unwrap_or(the_rest.unwrap_or(String::new()));

                break;
            } else if token.is_empty() {
            } else if token.contains("*") {
                // let rest_bak = the_rest.clone();
                let (loc_token, loc_the_rest) = token
                    // .clone()
                    .split_once("*")
                    .map(|(tok, rest)| (tok.to_string(), Some(rest.to_string())))
                    .unwrap_or((to_parse.to_string(), None));
                token = loc_token;
                // loc_tokens.append(&mut self.do_parse(token.as_str()));
                // self.do_parse(token.as_str(), &mut loc_tokens);
                trace!("[depth = {depth}]: note token {token} the_rest: {the_rest:?}");

                let n = if let Some(Ok(n)) = loc_the_rest
                    .as_ref()
                    .map(|the_rest| the_rest.parse::<usize>())
                //   i  .clone()
                //     .map(|token| token.split_once("*").map(|(_, n)| n.parse::<usize>()))
                {
                    n
                } else {
                    1
                };

                trace!("[depth = {depth}]: note token {token} mulitplied by {n}");
                let (loc_tokens, _loc_the_rest) = self.do_parse_pattern(close, &token, depth + 1);
                trace!("[depth = {depth}]: note token {token} the_rest: {_loc_the_rest:?}");
                // self.total_steps += loc_tokens
                //     .iter()
                //     .map(|token| match token.clone() {
                //         MnTokenType::Rest => 1,
                //         MnTokenType::Note(_) => 1,
                //         MnTokenType::Chord(_) => 1,
                //         MnTokenType::Alternator(_, notes) => notes.len(),
                //     })
                //     .sum::<usize>()
                //     * (n - 1);

                for _ in 0..n {
                    // self.total_steps += loc_tokens.len();
                    tokens.append(&mut loc_tokens.clone());
                }

                // the_rest = rest_bak.clone();

                // if let Some(tr) = rest_bak.clone() {
                //     (_, the_rest) = tr
                //         .split_once(" ")
                //         .map(|(tok, rest)| (tok.to_string(), Some(rest.to_string())))
                //         .unwrap_or((to_parse.to_string(), None));
                // }
                trace!(
                    "[depth = {depth}]: note token {token}, after if-let  the_rest: {the_rest:?}"
                );

                yet_to_parse = the_rest.clone().unwrap_or(String::new());
            } else if token == "~" {
                tokens.push(MnTokenType::Rest);
                yet_to_parse = the_rest.clone().unwrap_or(String::new());
            } else {
                trace!("[depth = {depth}]: pushing note: {token}, {the_rest:?}");
                tokens.push(MnTokenType::Note(token));
                yet_to_parse = the_rest.clone().unwrap_or(String::new());
                trace!("[depth = {depth}]: yet_to_parse: {yet_to_parse}.");
                // break;
            }

            if yet_to_parse.is_empty() {
                break;
            }

            match the_rest.as_ref() {
                Some(ytp) => {
                    trace!("[depth = {depth}]: ytp: {ytp} {close}");
                    // yet_to_parse = ytp.to_string();
                    // break;
                }
                None => break,
            }
        }

        // if close == "]" {
        //     self.total_steps += 1;
        // } else {
        //     self.total_steps += tokens.len();
        // }

        trace!("[depth = {depth}]: do_parse_pattern 3 => ({tokens:?}, {yet_to_parse:?})");
        // println!("[depth = {depth}]: do_parse_pattern 3 => ({tokens:?}, {yet_to_parse:?})");

        (tokens.clone(), yet_to_parse)
    }

    // fn do_parse(&mut self, to_parse: &str, tokens: &mut Vec<MnTokenType>) {
    //     // let tok_len = tokens.len();
    //     // while the_rest.is_some() || tokens.len() == tok_len {
    //     loop {
    //         let (token, mut the_rest) = to_parse
    //             .split_once(" ")
    //             .map(|(tok, rest)| (tok.to_string(), Some(rest.to_string())))
    //             .unwrap_or((to_parse.to_string(), None));
    //
    //         println!("do_parse => {token}");
    //
    //         if token.starts_with("<") {
    //             println!("before alternator");
    //             let (mut token, tmp_the_rest) = self.do_parse_pattern(
    //                 "<",
    //                 ">",
    //                 |tokens| MnTokenType::Alternator(0, tokens),
    //                 // tokens,
    //                 token.clone(),
    //                 the_rest,
    //             );
    //             the_rest = tmp_the_rest.clone();
    //             tokens.append(&mut token);
    //             println!("do_parse after Alternator => ({tokens:?}, {the_rest:?})");
    //             // break;
    //
    //             // } else if token.ends_with(">") {
    //         } else if token.contains("[") {
    //             // if token.con-- --nocapture --test-threads=1tains("]*") {
    //             // } else if token.ends_with("]") {
    //             // }
    //             println!("before chord");
    //             let (mut token, tmp_the_rest) = self.do_parse_pattern(
    //                 "[",
    //                 "]",
    //                 MnTokenType::Chord,
    //                 // tokens,
    //                 token.clone(),
    //                 the_rest,
    //             );
    //             // token = loc_token.clone();
    //             tokens.append(&mut token);
    //             the_rest = tmp_the_rest.clone();
    //             println!("do_parse 2 (after chord) => ({tokens:?}, {the_rest:?})");
    //             // break;
    //         } else if token.contains("*") {
    //             let (new_token, n) = if let Some((new_token, Ok(n))) = token
    //                 .split_once("*")
    //                 .map(|(token, n)| (token.to_string(), n.parse::<usize>()))
    //             {
    //                 (new_token, n)
    //             } else {
    //                 (token.clone(), 1)
    //             };
    //
    //             let mut tmp_tokens = Vec::new();
    //             self.do_parse(new_token.as_str(), &mut tmp_tokens);
    //
    //             for _ in 0..n {
    //                 tokens.append(&mut tmp_tokens.clone());
    //             }
    //         } else if token == "~" {
    //             tokens.push(MnTokenType::Rest)
    //         } else {
    //             println!("token = {token}");
    //             tokens.push(MnTokenType::Note(token.clone()))
    //         }
    //
    //         if the_rest.is_none() {
    //             break;
    //         }
    //
    //         // the_rest = Some(to_parse);
    //     }
    //
    //     // tokens
    // }

    pub fn parse(&mut self) {
        // let mut notes =
        // let mut tokens = Vec::new();
        // self.do_parse(self.full_text.clone().as_str(), &mut tokens);
        (self.notes, _) = self.do_parse_pattern("", self.full_text.clone().as_str(), 0);
        let mut notes_bak = self.notes.clone();
        // println!("notes =>  {notes_bak:?}");

        if !self.notes.is_empty() && self.has_alternator(&notes_bak) {
            let mut count = 0;
            let mut old_count = 1;
            let mut not_done = true;

            while not_done && old_count != count {
                old_count = count;
                for token in notes_bak.iter_mut() {
                    Self::do_count_steps(
                        token,
                        &mut count,
                        matches!(token, MnTokenType::Chord(_)),
                        &mut not_done,
                    );
                }

                // println!("count = {count}");
            }

            self.total_steps = count;
        } else {
            self.total_steps = self.notes.len();
        }

        debug!("tokens: {:?}", self.notes);
        debug!("total_steps: {:?}", self.total_steps);

        // self.notes = tokens;
    }

    fn has_alternator(&self, notes: &Vec<MnTokenType>) -> bool {
        let mut seen = false;

        for note in notes {
            match note {
                MnTokenType::Alternator(_, _, _) => {
                    seen = true;
                    break;
                }
                MnTokenType::Chord(notes) => {
                    seen = seen || self.has_alternator(notes);
                }
                _ => {}
            }
        }

        seen
    }

    fn do_get_next(note: &mut MnTokenType) -> Option<MnStepType> {
        match note {
            MnTokenType::Alternator(i, notes, _) => {
                let len = notes.len();
                let note = if len > 1 {
                    notes.get_mut(*i % len)?
                } else {
                    notes.get_mut(0)?
                };
                *i = i.wrapping_add(1);

                Self::do_get_next(note)
            }
            MnTokenType::Chord(notes) => {
                trace!("notes: {notes:?}");

                Some(MnStepType::Chord(
                    notes
                        .iter_mut()
                        .filter_map(|note| {
                            let res = Self::do_get_next(note);
                            trace!("res for note: {note:?} is {res:?}");

                            match res {
                                Some(MnStepType::Note(name)) => Some(name),
                                _ => None,
                            }
                        })
                        .collect(),
                ))
            }
            MnTokenType::Note(name) => Some(MnStepType::Note(name.to_string())),
            MnTokenType::Rest => Some(MnStepType::Rest),
        }
    }

    fn do_count_steps(
        note: &mut MnTokenType,
        count: &mut usize,
        in_chord: bool,
        not_done: &mut bool,
    ) {
        match note {
            MnTokenType::Alternator(i, notes, completed) => {
                // if *i < notes.len() {
                // if !*all_done {
                let len = notes.len();

                // for note in notes.iter_mut() {
                if let Some(note) = notes.get_mut(*i % len) {
                    Self::do_count_steps(note, count, false, not_done);
                }
                // }
                // }

                // println!("notes => {notes:?}");

                *i += 1;

                if !*completed {
                    *completed = *i >= len;
                }

                *not_done = *not_done && !*completed;

                // println!("not_done {not_done}");
            }
            MnTokenType::Chord(notes) => {
                let old_count = *count;

                for note in notes {
                    Self::do_count_steps(note, count, true, not_done);
                }

                if *count == old_count && !in_chord {
                    *count += 1;
                }
            }
            MnTokenType::Note(_name) => {
                if !in_chord {
                    *count += 1;
                }
            }
            MnTokenType::Rest => {
                if !in_chord {
                    *count += 1;
                }
            }
        }
    }
    pub fn get_next(&mut self) -> Option<Vec<String>> {
        let len = self.notes.len();

        if self.i == self.total_steps {
            return None;
        }

        let note = if len > 1 {
            self.notes.get_mut(self.i % len).unwrap()
        } else {
            self.notes.get_mut(0).unwrap()
        };

        let res = match Self::do_get_next(note) {
            Some(MnStepType::Chord(notes)) => notes,
            Some(MnStepType::Note(note)) => vec![note],
            Some(MnStepType::Rest) => vec!["~".into()],
            None => Vec::with_capacity(0),
        };

        self.i += 1;

        Some(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_note() {
        let mut parser = Parser::new("f3");
        parser.parse();
        let next = parser.get_next().unwrap();
        // let note = note_from_str("f3".into()).expect("should not be reachable");
        let note = "f3";

        assert_eq!(next, vec![note], "{next:?} != {:?}", vec![note])
    }

    #[test]
    fn single_chord() {
        let mut parser = Parser::new("[f3 c4]");
        parser.parse();
        assert_eq!(parser.total_steps, 1, "wrong number of notes registered");

        // for (i, note) in ["f3", "c4"].iter().enumerate() {
        let next = parser.get_next().unwrap();
        // let n_1 = note_from_str("f3".into()).expect("should not be reachable");
        // let n_2 = note_from_str("c4".into()).expect("should not be reachable");
        let n_1 = "f3";
        let n_2 = "c4";

        assert_eq!(next, vec![n_1, n_2], "{next:?} != {:?}", vec![n_1, n_2])
        // }
    }

    #[test]
    fn single_seq() {
        let mut parser = Parser::new("<f3 c4>");
        parser.parse();
        assert_eq!(parser.total_steps, 2, "wrong number of notes registered");

        for (i, note) in ["f3", "c4"].iter().enumerate() {
            let next = parser.get_next().unwrap();
            // let note = note_from_str((*note).into()).expect("should not be reachable");
            let note = note.to_string();

            assert_eq!(
                next,
                vec![note.clone()],
                "[note {i}]: {next:?} != {:?}",
                vec![note]
            )
        }

        // panic!("manual fail");
    }

    #[test]
    fn three_long_seq() {
        let mut parser = Parser::new("<f3 a3 c4>");
        parser.parse();
        assert_eq!(parser.total_steps, 3, "wrong number of notes registered");

        for (i, note_name) in ["f3", "a3", "c4"].iter().enumerate() {
            let next = parser.get_next().unwrap();
            // let note = note_from_str((*note_name).into()).expect("should not be reachable");
            let note_name = note_name.to_string();

            assert_eq!(
                next,
                vec![note_name.clone()],
                "[note {i}]: {next:?} != {note_name}"
            )
        }

        // panic!("manual fail");
    }

    #[test]
    fn chord_wrapping_seq() {
        let mut parser = Parser::new("[f3 <a3 c4>]");
        parser.parse();
        assert_eq!(
            parser.total_steps, 2,
            "wrong number of notes registered, {:?}",
            parser.notes
        );

        for (i, note_names) in [["f3", "a3"], ["f3", "c4"]].iter().enumerate() {
            let next = parser.get_next().unwrap();
            println!("next => {next:?}");

            for (j, note) in note_names.iter().enumerate() {
                // let note = note_from_str((*note_name).into()).expect("should not be reachable");
                let note = note.to_string();

                assert_eq!(next[j], note, "[note {i}:{j}]: {} != {}", next[j], note,)
            }
        }

        // panic!("manual fail");
    }

    #[test]
    fn seq_wrapping_chord() {
        let mut parser = Parser::new("<f3 [a3 c4]>");
        parser.parse();
        assert_eq!(parser.total_steps, 2, "wrong number of notes registered");

        for (i, note_names) in [vec!["f3"], vec!["a3", "c4"]].iter().enumerate() {
            let next = parser.get_next().unwrap();
            println!("next => {next:?}");

            for (j, note_name) in note_names.iter().enumerate() {
                // let note = note_from_str((*note_name).into()).expect("should not be reachable");
                let note = note_name.to_string();

                assert_eq!(
                    next[j], note,
                    "[note {i}:{j}]: {} != {}",
                    next[j], note_name,
                )
            }
        }

        // panic!("manual fail");
    }

    #[test]
    fn long_seq() {
        let mut parser = Parser::new("<f3 a3 c4>");
        parser.parse();
        assert_eq!(parser.total_steps, 3, "wrong number of notes registered");

        for (i, note_names) in [vec!["f3"], vec!["a3"], vec!["c4"]].iter().enumerate() {
            let next = parser.get_next().unwrap();
            println!("next => {next:?}");

            for (j, note_name) in note_names.iter().enumerate() {
                // let note = note_from_str((*note_name).into()).expect("should not be reachable");
                let note = note_name.to_string();

                assert_eq!(
                    next[j], note,
                    "[note {i}:{j}]: {} != {}",
                    next[j], note_name,
                )
            }
        }

        // panic!("manual fail");
    }

    #[test]
    fn multiply_note_in_seq() {
        let mut parser = Parser::new("<f3*3 c4>");
        parser.parse();
        assert_eq!(parser.total_steps, 4, "wrong number of notes registered");

        for (_i, note) in ["f3", "f3", "f3", "c4"].iter().enumerate() {
            let next = parser.get_next().unwrap();
            // let note = note_from_str(note.to_string()).expect("should not be reachable");
            let note = note.to_string();

            assert_eq!(next, vec![note.clone()], "{next:?} != {:?}", vec![note])
        }
    }

    #[test]
    fn multiply_a_seq() {
        let mut parser = Parser::new("<f3 c4>*3");
        parser.parse();
        assert_eq!(
            parser.total_steps, 6,
            "wrong number of notes registered, was: {}, notes: {:?}",
            parser.total_steps, parser.notes,
        );
        assert_eq!(parser.notes.len(), 6, "wrong length token tree");

        for _ in 0..3 {
            for (i, note_name) in ["f3", "c4"].iter().enumerate() {
                let next = parser.get_next().unwrap();
                println!("next => {next:?}");

                // for (j, note_name) in note_names.iter().enumerate() {
                // let note = note_from_str((*note_name).into()).expect("should not be reachable");
                let note = note_name.to_string();

                assert_eq!(
                    next,
                    vec![note.clone()],
                    "[note {i}]: {:?} != {}:{:?}",
                    next,
                    note_name,
                    vec![note]
                )
            }
        }
    }

    #[test]
    fn nested_seq() {
        let to_parse = "c4 <e4*2 b4> <c4*2 g4> g4";
        let mut parser = Parser::new(to_parse);
        parser.parse();
        for i in 0..parser.total_steps {
            println!("{i}: note => {:?}", parser.get_next());
        }

        assert_eq!(
            parser.total_steps, 12,
            "wrong number of notes registered, was: {}, {to_parse}, notes: {:?}",
            parser.total_steps, parser.notes,
        );
    }
}
