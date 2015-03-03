#![feature(plugin)]
#![feature(core)]
#![feature(old_io)]
#![feature(old_path)]

extern crate regex;
extern crate sdl2;

use std::old_io::File;
use std::old_io::BufferedReader;
use std::old_io::BufferedWriter;

use regex::Regex;

#[derive(Clone)]
struct Rule {
	match_as: Regex,
	replace_with: String
}

enum Token {
	Literal(String),
	Regex(String),
	Placeholder,
	ArrowOperator
}

fn tokenize(data: &str) -> Vec<Token> {
	let mut in_quote = false;
	let mut ret = vec!["".to_string()];
	for c in data.chars() {
		let mut top = ret.pop().unwrap();
		match c {
			c if c.is_whitespace() => {
				if in_quote {
					top.push(c);
				} else {
					if top.as_slice() != "" {
						ret.push(top);
						ret.push("".to_string());
						continue
					}
				}
			},
			'"' => {
				in_quote = !in_quote;
			},
			_ => {
				top.push(c);
			}
		}
		ret.push(top);
	}
	let mut ret_toks: Vec<Token> = Vec::new();
	for tok in ret {
		ret_toks.push(match tok.as_slice() {
			"=>" => Token::ArrowOperator,
			"_" => Token::Placeholder,
			t => {
				if t.contains("^") {
					Token::Regex(t.to_string().slice_from(1).to_string())
				} else {
					Token::Literal(t.to_string())
				}
			}
		});
	}
	ret_toks
}


fn replace_rule(data: &str, rule: &Rule) -> String {
	rule.match_as.replace_all(data, rule.replace_with.as_slice())
}

fn parse_rule(data: &str) -> Option<Rule> {
	let tokens = tokenize(data);
	let mut new_regex = String::new();
	let mut replacement = String::new();
	let mut after_arrow = false;
	{
		for tok in tokens {
			let mut current_string = if !after_arrow {
				& mut new_regex
			} else {
				& mut replacement
			};
			match tok {
				Token::Literal(s) => {
					if !after_arrow {
						current_string.push_str(regex::quote(s.as_slice()).as_slice())
					} else {
						current_string.push_str(s.as_slice())
					}
				},
				Token::Regex(s) => {
					current_string.push_str(s.as_slice())
				}
				Token::ArrowOperator => after_arrow = true,
				Token::Placeholder => current_string.push_str("([^、]+?)"),
			}
		}
	}

	let regex = match Regex::new(new_regex.as_slice()) {
		Ok(re) => re,
		Err(_) => return None
	};


	Some(Rule { match_as : regex, replace_with: replacement })
}

fn main() {
	let sdl_context = sdl2::init(sdl2::INIT_EVERYTHING).unwrap();

	let regex = match Regex::new(r"\\U(.+?)\\E") {
		Ok(re) => re,
		Err(_) => panic!("Regex compilation fail!")
	};



	'main_loop: loop {
		/*let data_path = Path::new("test_data.txt");
		let mut file = BufferedReader::new(File::open(&data_path));
		let mut data = String::new();
		for line in file.lines() {
			data.push_str(line.unwrap().as_slice());
		}*/

		let clipboard = sdl2::clipboard::get_clipboard_text();
		
		let data = match clipboard {
			Ok(s) => s,
			Err(s) => panic!(s),
		};

		let rule_path = Path::new("rules.txt");
		let mut rule_file = BufferedReader::new(File::open(&rule_path));

		let mut data_array = vec!(String::new());

		for c in data.clone().chars() {
			let mut val = data_array.pop().unwrap();
			match c {
				'〕' | '〔' | '］' | '［' => {
					data_array.push(val);
					data_array.push(c.to_string());
					data_array.push("".to_string());
					continue
				}
				'。' | '、'
				=> {
					data_array.push(val);
					val = String::new();
				}
				_ => ()
			}
			val.push(c);
			/*match c {
				'〔' => {
					data_array.push(val);
					data_array.push("〔".to_string());
					data_array.push(String::new());
					continue
				},
				_ => ()
			}
			match c {
				'。' | '〕' | '、' 
				=> {
					data_array.push(val);
					val = String::new();
				}
				_ => ()
			}*/


			data_array.push(val);
		}

		let mut new_data: Vec<String> = Vec::new();
		for line in rule_file.lines() {

			let new_rule = parse_rule(line.unwrap().as_slice()).unwrap();
			for mut t in data_array {
				t = replace_rule(t.as_slice(), &new_rule);
				t = regex.replace_all(t.as_slice(), |caps: &regex::Captures| {
				    caps.at(1).unwrap().chars().map(|c| c.to_uppercase()).collect::<String>()
				});
				new_data.push(t);
			}
			data_array = new_data;
			new_data = Vec::new();
		}
		println!("");
		println!("New Card:");

		let out_file = Path::new("translated.txt");
		let mut out_file_writer = BufferedWriter::new(File::create(&out_file));

		for t in data_array {
			write!(& mut out_file_writer, "{}", t);
			print!("{}", t);
		}

		out_file_writer.flush().unwrap();
		println!("");

		break;

		'wait_loop: loop {
			let new_clipboard = sdl2::clipboard::get_clipboard_text();
			let new_data = match new_clipboard {
				Ok(s) => s,
				Err(_) => continue 'wait_loop,
			};
			if new_data.as_slice() != data.as_slice() {
				continue 'main_loop
			}

		}
	}


}
