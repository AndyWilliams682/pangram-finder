use std::{collections::HashMap};
use std::time::Instant;



const ALL_WORDS: &str = include_str!("words.txt");
const ALL_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug)]
struct BitsByLetter {
    word_data: Vec<LetterData>,
}

impl BitsByLetter {
    fn find_solutions(&self, next_missing_letter: usize, words_picked: &mut Vec<u32>, bits_picked: u32, max_words: usize, mut solutions: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
        for new_word in &self.word_data[next_missing_letter].letter_words {
            let new_words_picked = &mut vec![*new_word];
            new_words_picked.extend_from_slice(&words_picked);

            let mut new_missing_letter = 0;
            new_missing_letter += next_missing_letter;

            let new_bits_picked = bits_picked | new_word;

            while (1 << self.word_data[new_missing_letter].alphabet_index) & (new_bits_picked) > 0 {
                new_missing_letter += 1;
                if new_missing_letter == 26 {
                    // Every letter is present
                    let mut new_solution = new_words_picked.to_vec();
                    new_solution.sort(); // Some duplicate solutions can be found, this prevents them from being stored

                    if !solutions.contains(&new_solution) {
                        solutions.push(new_solution);
                    }
                    break
                }
            }
            if new_words_picked.len() >= max_words {
                // Failed to find a solution before selecting the maximum number of words
                continue
            }
            solutions = BitsByLetter::find_solutions(self, new_missing_letter, new_words_picked, new_bits_picked, max_words, solutions);
        }
        return solutions
    }
}

#[derive(Debug)]
struct LetterData {
    alphabet_index: usize, // Where the letter sits in the alphabet
    letter_words: Vec<u32> // List of words containing the letter, encoded as u32
}

fn sanitize_word(word: &str) -> String {
    word.trim()
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect()
}

fn encode_word(word: &String) -> u32 {
    ALL_LETTERS
        .chars()
        .fold(0, |acc, c| (acc << 1) + word.contains(c) as u32)
}

fn word_list() -> Vec<String> {
    let mut output: Vec<String> = ALL_WORDS
        .split("\n")
        .map(sanitize_word)
        .filter(|line| line.len() > 0)
        .collect();
    output.sort();
    output.dedup();
    output
}

fn find_encoded_non_subsets(encoded_word_vec: &Vec<u32>) -> Vec<bool> {
    let mut keep: Vec<bool> = vec![];
    for smaller_value_index in 0..encoded_word_vec.len() {
        let mut is_subset = true;
        for larger_value_index in 0..smaller_value_index {
            // If A | B == B, then A is a subset of B
            if encoded_word_vec[larger_value_index] | encoded_word_vec[smaller_value_index] == encoded_word_vec[larger_value_index] {
                is_subset = false;
                break
            }
        }
        keep.push(is_subset)
    }
    keep
}

fn get_bits_by_letter(encoded_word_vec: Vec<u32>) -> BitsByLetter {
    let mut word_data: Vec<LetterData> = vec![];
    for letter_index in 0..26 {
        word_data.push(LetterData { alphabet_index: letter_index, letter_words: vec![] })
    }

    for bits in encoded_word_vec {
        let mut int_bits = bits as i32;
        // Write function that gets lowest bit
        while int_bits != 0 {
            let lowest_set_bit = int_bits & -int_bits;
            let letter_index = 31 - lowest_set_bit.leading_zeros() as usize;
            word_data[letter_index].letter_words.push(bits);
            int_bits -= lowest_set_bit;
        }
    }

    // Each index represents a letter, and it's sorted from least to most frequent
    word_data.sort_by(|a, b| a.letter_words.len().cmp(&b.letter_words.len()));
    BitsByLetter { word_data }
}

fn main() {
    let max_words = 5; // Maximum number of words to use for finding pangrams, does not work if solutions exist below max_words
    let exhaustive_search = false; // If true, subset words will be removed ("ALL" is a subset of "BALL")

    let start = Instant::now();

    let mut all_words = word_list();

    let mut all_bits: Vec<u32> = all_words
        .iter()
        .map(encode_word)
        .collect();
    
    let mut bits_to_index: HashMap<u32, usize> = HashMap::new();
    for bits_index in 0..all_bits.len() {
        if bits_to_index.contains_key(&all_bits[bits_index]) {
            // If two words have the same letters, they get mapped to the same index and joined with a /
            // Ex: "MATE" and "TEAM" have the same letters (and bits), so the bits will point to "MATE / TEAM"
            let alternate_word = &all_words[bits_index].to_owned();
            all_words[bits_to_index[&all_bits[bits_index]]].push_str(&format!(" / {}", alternate_word));
        }
        bits_to_index.entry(all_bits[bits_index]).or_insert(bits_index);
    }

    all_bits.sort();
    all_bits.dedup();
    all_bits.reverse();

    if exhaustive_search {
        let non_subsets = find_encoded_non_subsets(&all_bits);
        let mut iter = non_subsets.iter();
        all_bits.retain(|_| *iter.next().unwrap());
    }

    let final_structure = get_bits_by_letter(all_bits);
    let final_solutions = final_structure.find_solutions(
        0,
        &mut vec![],
        0,
        max_words,
        vec![]
    );
    
    let mut total_solutions = 0;
    
    for solution in final_solutions {
        let written_solution: Vec<&String> = solution
            .into_iter()
            .map(|x| &all_words[bits_to_index[&x]])
            .collect();

        total_solutions += written_solution
            .iter()
            .fold(1, |acc: i32, s| acc * (s.split("/").count() as i32));
    }

    println!("Total Solutions for {:?} words is: {:?}", max_words, total_solutions);
    
    let duration = start.elapsed();
    println!("Time Elapsed is: {:?}", duration);
}
