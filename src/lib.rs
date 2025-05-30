use wasm_bindgen::prelude::*;
use web_sys::{console, Storage};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum PatternElement {
    Word { text: String },
    Gap { min_words: u32, max_words: Option<u32> },
    Reference { pattern_id: String },
    OneOf { options: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Pattern {
    Sequence {
        id: String,
        name: String,
        elements: Vec<PatternElement>,
    },
    Composite {
        id: String,
        name: String,
        operator: CompositeOperator,
        patterns: Vec<Pattern>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CompositeOperator {
    And,
    Or,
    Not,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelectionSpan {
    text: String,
    start_index: usize,
    end_index: usize,
    word_index: usize,
}

impl Pattern {
    pub fn to_regex(&self) -> String {
        match self {
            Pattern::Sequence { elements, .. } => {
                let mut parts = Vec::new();
                for element in elements {
                    match element {
                        PatternElement::Word { text } => {
                            // Check if it's a phrase (contains spaces)
                            if text.contains(' ') {
                                // For phrases, match the exact phrase with word boundaries
                                parts.push(format!(r"\b{}\b", regex::escape(text)));
                            } else {
                                // For single words
                                parts.push(format!(r"\b{}\b", regex::escape(text)));
                            }
                        }
                        PatternElement::Gap { min_words, max_words } => {
                            // For AND patterns (open-ended gaps), match anything
                            if *min_words == 0 && max_words.is_none() {
                                parts.push(String::from(r".*?"));  // Non-greedy match anything
                            } else {
                                let gap_pattern = match max_words {
                                    Some(max) => format!(r"(?:\W+\w+){{{},{}}}", min_words, max),
                                    None => format!(r"(?:\W+\w+){{{},}}", min_words),
                                };
                                parts.push(gap_pattern);
                            }
                        }
                        PatternElement::OneOf { options } => {
                            let escaped_options: Vec<String> = options
                                .iter()
                                .map(|opt| regex::escape(opt))
                                .collect();
                            parts.push(format!(r"\b(?:{})\b", escaped_options.join("|")));
                        }
                        PatternElement::Reference { .. } => {
                            // TODO: Implement pattern reference resolution
                            parts.push(String::from(".*"));
                        }
                    }
                }
                // Don't join with \W+ anymore, let the gaps handle the spacing
                parts.join("")
            }
            Pattern::Composite { operator, patterns, .. } => {
                match operator {
                    CompositeOperator::Or => {
                        let sub_patterns: Vec<String> = patterns
                            .iter()
                            .map(|p| format!("({})", p.to_regex()))
                            .collect();
                        sub_patterns.join("|")
                    }
                    CompositeOperator::And => {
                        // For AND, we need to use lookahead assertions
                        let sub_patterns: Vec<String> = patterns
                            .iter()
                            .map(|p| format!("(?=.*{})", p.to_regex()))
                            .collect();
                        format!("{}.*", sub_patterns.join(""))
                    }
                    CompositeOperator::Not => {
                        // NOT is implemented as negative lookahead
                        if let Some(pattern) = patterns.first() {
                            format!("(?!.*{})", pattern.to_regex())
                        } else {
                            String::new()
                        }
                    }
                }
            }
        }
    }

    pub fn get_id(&self) -> &str {
        match self {
            Pattern::Sequence { id, .. } => id,
            Pattern::Composite { id, .. } => id,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Pattern::Sequence { name, .. } => name,
            Pattern::Composite { name, .. } => name,
        }
    }
}

#[wasm_bindgen]
pub struct PatternBuilder {
    patterns: Vec<Pattern>,
    current_selections: Vec<SelectionSpan>,
}

#[wasm_bindgen]
impl PatternBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PatternBuilder {
        console::log_1(&"PatternBuilder initialized".into());
        
        let patterns = load_patterns_from_storage();
        
        PatternBuilder {
            patterns,
            current_selections: Vec::new(),
        }
    }

    pub fn add_selection(&mut self, text: String, start_index: usize, end_index: usize, word_index: usize) {
        let selection = SelectionSpan {
            text,
            start_index,
            end_index,
            word_index,
        };
        self.current_selections.push(selection);
    }

    pub fn clear_selections(&mut self) {
        self.current_selections.clear();
    }

    pub fn build_sequence_pattern(&mut self, name: String) -> Result<String, JsValue> {
        if self.current_selections.is_empty() {
            return Err(JsValue::from_str("No selections to build pattern from"));
        }

        // Sort selections by their position in the text
        self.current_selections.sort_by_key(|s| s.word_index);

        let mut elements = Vec::new();
        let mut i = 0;

        while i < self.current_selections.len() {
            let start_selection = &self.current_selections[i];
            let mut phrase_words = vec![start_selection.text.clone()];
            let mut j = i + 1;

            // Collect adjacent words into a phrase
            while j < self.current_selections.len() {
                let current = &self.current_selections[j - 1];
                let next = &self.current_selections[j];
                
                // Check if words are adjacent (consecutive word indices)
                if next.word_index == current.word_index + 1 {
                    phrase_words.push(next.text.clone());
                    j += 1;
                } else {
                    break;
                }
            }

            // Add the word or phrase element
            if phrase_words.len() == 1 {
                elements.push(PatternElement::Word {
                    text: phrase_words[0].clone(),
                });
            } else {
                // Join adjacent words with spaces to create a phrase
                elements.push(PatternElement::Word {
                    text: phrase_words.join(" "),
                });
            }

            // If there's a next selection, determine if we need a gap
            if j < self.current_selections.len() {
                // For non-adjacent selections, we use an open-ended gap
                // This creates an AND pattern - both parts must exist but with anything in between
                elements.push(PatternElement::Gap {
                    min_words: 0,
                    max_words: None, // No upper limit - matches any amount of text
                });
            }

            i = j;
        }

        let pattern = Pattern::Sequence {
            id: generate_id(),
            name: name.clone(),
            elements,
        };

        let regex = pattern.to_regex();
        self.patterns.push(pattern);
        
        save_patterns_to_storage(&self.patterns)?;
        self.clear_selections();
        
        Ok(regex)
    }

    pub fn get_patterns(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.patterns).unwrap()
    }

    pub fn get_pattern_preview(&self) -> JsValue {
        if self.current_selections.is_empty() {
            return JsValue::NULL;
        }

        let mut sorted_selections = self.current_selections.clone();
        sorted_selections.sort_by_key(|s| s.word_index);

        let mut preview_elements = Vec::new();
        let mut i = 0;

        while i < sorted_selections.len() {
            let mut phrase_words = vec![sorted_selections[i].text.clone()];
            let mut j = i + 1;

            // Collect adjacent words into a phrase
            while j < sorted_selections.len() {
                let current = &sorted_selections[j - 1];
                let next = &sorted_selections[j];
                
                if next.word_index == current.word_index + 1 {
                    phrase_words.push(next.text.clone());
                    j += 1;
                } else {
                    break;
                }
            }

            // Add phrase or word element
            if phrase_words.len() == 1 {
                preview_elements.push(serde_json::json!({
                    "type": "word",
                    "text": phrase_words[0]
                }));
            } else {
                preview_elements.push(serde_json::json!({
                    "type": "phrase",
                    "text": phrase_words.join(" ")
                }));
            }

            // If there's a next selection, show AND relationship
            if j < sorted_selections.len() {
                preview_elements.push(serde_json::json!({
                    "type": "and",
                    "text": "AND"
                }));
            }

            i = j;
        }

        serde_wasm_bindgen::to_value(&preview_elements).unwrap()
    }

    pub fn test_pattern(&self, pattern_index: usize, text: &str) -> JsValue {
        if let Some(pattern) = self.patterns.get(pattern_index) {
            let regex_str = pattern.to_regex();
            match regex::Regex::new(&regex_str) {
                Ok(re) => {
                    let matches: Vec<(usize, usize)> = re
                        .find_iter(text)
                        .map(|m| (m.start(), m.end()))
                        .collect();
                    
                    serde_wasm_bindgen::to_value(&matches).unwrap()
                }
                Err(_) => JsValue::NULL
            }
        } else {
            JsValue::NULL
        }
    }

    pub fn delete_pattern(&mut self, index: usize) -> Result<(), JsValue> {
        if index < self.patterns.len() {
            self.patterns.remove(index);
            save_patterns_to_storage(&self.patterns)?;
        }
        Ok(())
    }

    pub fn remove_selection(&mut self, index: usize) {
        if index < self.current_selections.len() {
            self.current_selections.remove(index);
        }
    }
}

fn get_local_storage() -> Result<Storage, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    window.local_storage()?.ok_or_else(|| JsValue::from_str("No local storage"))
}

fn save_patterns_to_storage(patterns: &[Pattern]) -> Result<(), JsValue> {
    let storage = get_local_storage()?;
    let json = serde_json::to_string(patterns).map_err(|e| JsValue::from_str(&e.to_string()))?;
    storage.set_item("regexgen_patterns", &json)?;
    Ok(())
}

fn load_patterns_from_storage() -> Vec<Pattern> {
    match get_local_storage() {
        Ok(storage) => {
            match storage.get_item("regexgen_patterns") {
                Ok(Some(json)) => {
                    serde_json::from_str(&json).unwrap_or_default()
                }
                _ => Vec::new()
            }
        }
        _ => Vec::new()
    }
}

fn generate_id() -> String {
    let timestamp = js_sys::Date::now() as u64;
    let random = (js_sys::Math::random() * 1000.0) as u64;
    format!("{}-{}", timestamp, random)
}

#[wasm_bindgen]
pub fn get_word_at_position(text: &str, position: usize) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    
    if position >= chars.len() {
        return None;
    }

    // Find word boundaries
    let mut start = position;
    let mut end = position;

    // Move start backwards to beginning of word
    while start > 0 && chars[start - 1].is_alphanumeric() {
        start -= 1;
    }

    // Move end forward to end of word
    while end < chars.len() && chars[end].is_alphanumeric() {
        end += 1;
    }

    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

#[derive(Serialize, Deserialize)]
pub struct WordInfo {
    text: String,
    start_index: usize,
    end_index: usize,
    word_index: usize,
}

#[wasm_bindgen]
pub fn get_words_from_text(text: &str) -> JsValue {
    let mut words = Vec::new();
    let mut word_index = 0;
    let mut chars = text.char_indices().peekable();
    
    while let Some((i, c)) = chars.next() {
        if c.is_alphanumeric() {
            let start = i;
            let mut end = i;
            let mut word = String::from(c);
            
            while let Some(&(j, next_c)) = chars.peek() {
                if next_c.is_alphanumeric() {
                    word.push(next_c);
                    end = j;
                    chars.next();
                } else {
                    break;
                }
            }
            
            words.push(WordInfo {
                text: word,
                start_index: start,
                end_index: end + 1,
                word_index,
            });
            word_index += 1;
        }
    }
    
    serde_wasm_bindgen::to_value(&words).unwrap()
}