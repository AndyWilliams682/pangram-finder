use std::collections::HashMap;
use itertools::Itertools;
use std::time::Instant;

const ALL_WORDS: &str = include_str!("words.txt");
const MAX_SOLUTION_SIZE: usize = 4; // Maximum number of words to use for finding pangrams

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    fn find_pangrams(&self, current_pangram: Pangram, mut pangrams: Vec<Solution>) -> Vec<Solution> {
        for new_word in &self.search_structure[current_pangram.next_missing_letter()].words {
            match current_pangram.check_with(new_word.clone()) {
                PangramState::CompletePangram(solution) => {
                    pangrams.push(solution);
                    continue
                },
                PangramState::FailedPangram() => continue,
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

    fn check_with(&self, new_word: Word) -> PangramState {
        let new_selected_letters = self.selected_letters | new_word.letters_present;
        if new_selected_letters.leading_ones() >= 26 {
            let new_selected_words = &mut vec![new_word.clone()];
            new_selected_words.extend_from_slice(&self.selected_words);
            new_selected_words.sort_by(|a, b| a.letters_present.cmp(&b.letters_present));
            return PangramState::CompletePangram(Solution { words: new_selected_words.to_vec() })
        } else if self.selected_words.len() + 1 >= MAX_SOLUTION_SIZE {
            return PangramState::FailedPangram()
        } else {
            let new_selected_words = &mut vec![new_word.clone()];
            new_selected_words.extend_from_slice(&self.selected_words);
            let new_pangram = Pangram { selected_words: new_selected_words.to_vec(), selected_letters: new_selected_letters };
            return PangramState::PotentialPangram(new_pangram)
        }
    }

    fn next_missing_letter(&self) -> usize {
        return self.selected_letters.leading_ones() as usize
    }
}

#[derive(Debug)]
enum PangramState {
    PotentialPangram(Pangram),
    FailedPangram(),
    CompletePangram(Solution)
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
struct Solution {
    words: Vec<Word>
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

    let start = Instant::now();

    let all_pangrams = search_structure.find_pangrams(Pangram::new(), vec![]);
    let no_dupes = all_pangrams.into_iter().unique();
    
    println!("{:?}", no_dupes.collect::<Vec<Solution>>().len());
    println!("Time elapsed is: {:?}", start.elapsed());
}
