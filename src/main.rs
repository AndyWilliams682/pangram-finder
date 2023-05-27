use std::collections::HashMap;

const ALL_WORDS: &str = include_str!("words.txt");
const MAX_SOLUTION_SIZE: i32 = 4; // Maximum number of words to use for finding pangrams

#[derive(Debug, PartialEq, Clone)]
struct SanitizedString(String);

impl SanitizedString {
    fn sanitize(string: &str) -> SanitizedString {
        let output = string
            .trim()
            .to_uppercase()
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect();

        Self(output)
    }

    fn get_unique_letters(&self) -> String {
        let mut output: Vec<char> = self.0.chars().collect();
        output.sort_by(|a, b| a.cmp(&b));
        output.dedup();
        return output.iter().collect::<String>()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Word {
    name: String,
    letters_present: u32
}

impl Word {
    fn parse_string(s: &SanitizedString, order_of_letters: &Vec<char>) -> Word {
        let mut letters_in_word = order_of_letters
            .to_owned()
            .into_iter()
            .fold(0, |acc: u32, letter| (acc << 1) + s.0.contains(letter) as u32);
        letters_in_word <<= 32 - order_of_letters.len();
        return Word { name: s.0.to_owned(), letters_present: letters_in_word }
    }
}

#[derive(Debug)]
struct WordsWithLetter {
    words: Vec<Word>
}

impl WordsWithLetter {
    fn new() -> WordsWithLetter {
        return WordsWithLetter { words: vec![] }
    }
}

#[derive(Debug)]
struct SearchStructure {
    search_structure: Vec<WordsWithLetter>
}

impl SearchStructure {
    fn build(number_of_letters: usize, words: Vec<Word>) -> SearchStructure {
        let mut output = vec![];
        for _letter in 0..number_of_letters {
            output.push(WordsWithLetter::new())
        }

        for word in words {
            let mut letters_remaining = word.letters_present;
            while letters_remaining != 0 {
                let next_letter_index = letters_remaining.leading_zeros() as usize;
                output[next_letter_index].words.push(word.clone());
                letters_remaining -= 1 << (31 - next_letter_index)
            }
        }

        return SearchStructure { search_structure: output }
    }

    fn find_pangrams(&self, current_pangram: Pangram, mut pangrams: Vec<Pangram>) -> Vec<Pangram> {
        for new_word in &self.search_structure[current_pangram.next_missing_letter()].words {
            match current_pangram.add_new_word(new_word.clone()) {
                PangramState::CompletePangram(solution) => {
                    // Duplicate CompletePangrams are possible and need to be filtered out
                    if !pangrams.contains(&solution) {
                        pangrams.push(solution)
                    }
                    continue
                },
                PangramState::FailedPangram(_) => continue,
                PangramState::PotentialPangram(potential_solution) => {
                    pangrams = self.find_pangrams(potential_solution, pangrams)
                }
            }
        }
        return pangrams
    }
}

#[derive(Debug, PartialEq)]
struct Pangram {
    // Pangram, in this context, refers to a group of Words that captures one of each letter
    // Trivial Example: [ABCDE, FGHIJ, KLMNO, PQRST, UVWXYZ]
    selected_words: Vec<Word>,
    selected_letters: u32
}

impl Pangram {
    fn new() -> Pangram {
        return Pangram { selected_words: vec![], selected_letters: 0 }
    }

    fn add_new_word(&self, new_word: Word) -> PangramState {
        let new_selected_words = &mut vec![new_word.clone()];
        new_selected_words.extend_from_slice(&self.selected_words);

        let new_selected_letters = self.selected_letters | new_word.letters_present;

        let new_pangram = Pangram { selected_words: new_selected_words.to_vec(), selected_letters: new_selected_letters };

        return new_pangram.check()
    }

    fn check(mut self) -> PangramState {
        if self.selected_letters.leading_ones() >= 26 {
            return PangramState::CompletePangram(self.sort_words())
        } else if self.selected_words.len() >= MAX_SOLUTION_SIZE as usize {
            return PangramState::FailedPangram(self)
        } else {
            return PangramState::PotentialPangram(self)
        }
    }

    fn next_missing_letter(&self) -> usize {
        return self.selected_letters.leading_ones() as usize
    }

    fn sort_words(&mut self) -> Pangram {
        let words_in_solution = &mut self.selected_words;
        words_in_solution.sort_by(|a, b| a.letters_present.cmp(&b.letters_present));
        return Pangram { selected_words: words_in_solution.to_vec(), selected_letters: self.selected_letters }
    }
}

#[derive(Debug)]
enum PangramState {
    PotentialPangram(Pangram),
    FailedPangram(Pangram),
    CompletePangram(Pangram)
}

fn main() -> () {
    let mut sanitized_strings: Vec<SanitizedString> = ALL_WORDS
        .split("\n")
        .map(SanitizedString::sanitize)
        .filter(|line| line.0.len() > 0)
        .collect();
    sanitized_strings.sort_by(|s1, s2| s1.0.cmp(&s2.0));
    sanitized_strings.dedup();

    let occurences_of_each_letter: HashMap<char, u32> = sanitized_strings
        .iter()
        .map(|s| s.get_unique_letters())
        .collect::<String>()
        .chars()
        .fold(HashMap::new(), |mut map, letter| {
            *map.entry(letter).or_insert(0) += 1;
            map
        });

    let mut letters_sorted_by_rarity: Vec<char> =
        occurences_of_each_letter.keys().copied().collect();
    letters_sorted_by_rarity
        .sort_by(|a, b| occurences_of_each_letter[a].cmp(&occurences_of_each_letter[b]));

    let word_list: Vec<Word> = sanitized_strings
        .iter()
        .map(|s| Word::parse_string(s, &letters_sorted_by_rarity))
        .collect();
    
    let search_structure = SearchStructure::build(letters_sorted_by_rarity.len(),
                                                  word_list);

    let all_pangrams = search_structure.find_pangrams(Pangram::new(), vec![]);
    
    println!("{:?}", all_pangrams.len());
}
