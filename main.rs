use std::cmp;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead};
use std::io::{Error, ErrorKind};
use std::path::Path;

fn main() {
    for file_name in [
        "a_example.txt",
        "b_read_on.txt",
        "c_incunabula.txt",
        "d_tough_choices.txt",
        "e_so_many_books.txt",
        "f_libraries_of_the_world.txt",
    ]
    .iter()
    {
        println!("processing {}", file_name);
        let (libraries, books, days_left) = read_file(file_name.to_string()).unwrap();
        let signup_queue = main_loop(libraries, books, days_left);
        print_output(signup_queue, file_name.to_string());
    }
}

fn print_output(signup_queue: Vec<Library>, file_name: String) {
    let file_name = format!("{}.output", file_name);
    let mut file = match File::create(file_name) {
        Err(_) => panic!("couldn't create: "),
        Ok(file) => file,
    };
    file.write_all(format!("{}\n", signup_queue.len()).as_bytes()).unwrap();

    for lib in signup_queue {
        file.write_all(format!("{} {}\n", lib.id, lib.books.len()).as_bytes()).unwrap();

        for book in lib.books {
            file.write_all(format!("{} ", book.id).as_bytes()).unwrap();
        }
        file.write_all("\n".as_bytes()).unwrap();
    }
}

fn main_loop(
    mut libraries: HashMap<i64, Library>,
    books: HashMap<i64, Book>,
    mut days_left: i64,
) -> Vec<Library> {
    let mut signup_queue: Vec<Library> = Vec::new();

    while days_left > 0 {
        //get library with highest score
        if let Some(best_lib) = calc_best_lib_score(&libraries, &books, days_left) {
            //println!("Best lib: {:?}", best_lib);
            //add library to signup queue with all books in it
            signup_queue.push(best_lib.clone());

            //remove lib
            libraries.remove(&best_lib.id);

            //deduplicate books in all other libraries
            deduplicate(&mut libraries, &books, best_lib.books);

            //calculate new time left
            days_left -= best_lib.signup_time;
        } else {
            break;
        }
    }
    //println!("COMPLTED: {:#?}", signup_queue);
    signup_queue
}

fn deduplicate(
    libraries: &mut HashMap<i64, Library>,
    books: &HashMap<i64, Book>,
    books_to_dedup: Vec<Book>,
) {
    for book in books_to_dedup {
        let libraries_containing_book = books.get(&book.id).unwrap();
        for lib in &libraries_containing_book.libraries {
            if let Some(l) = libraries.get_mut(&lib) {
                l.books.retain(|x| x.id != book.id);
            }
        }
    }
    //println!("Libraries updated: {:#?}", libraries);
}

fn calc_best_lib_score(
    libraries: &HashMap<i64, Library>,
    _books: &HashMap<i64, Book>,
    days_left: i64,
) -> Option<Library> {
    //println!("days left: {}, libraries left {:#?}", days_left, libraries);
    let mut current_max_score = 0;
    let mut best_lib: Option<Library> = None;
    for (_key, lib) in libraries {
        let days_after_signup: i64 = days_left - lib.signup_time;
        let number_of_books_scannable_in_remaining_days = days_after_signup * lib.books_per_day;

        if days_after_signup < 0 {
            break;
        }

        let mut score_sum = 0;
        let mut i = 0;
        while i < cmp::min(
            lib.books.len(),
            number_of_books_scannable_in_remaining_days
                .try_into()
                .unwrap(),
        ) {
            score_sum += lib.books[i].score;
            i += 1;
        }

        if score_sum > current_max_score {
            current_max_score = score_sum;
            best_lib = Some(lib.clone());
        }
    }
    best_lib
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Book {
    id: i64,
    score: i64,
    libraries: Vec<i64>,
}
impl Book {
    pub fn new(id: i64, score: i64) -> Book {
        let libraries = Vec::new();
        Book {
            id: id,
            score: score,
            libraries: libraries,
        }
    }
}
#[derive(Clone, Debug)]
struct Library {
    id: i64,
    signup_time: i64,
    books_per_day: i64,
    books: Vec<Book>,
}
impl Library {
    pub fn new(id: i64, signup_time: i64, books_per_day: i64) -> Library {
        let books = Vec::new();
        Library {
            id: id,
            signup_time: signup_time,
            books_per_day: books_per_day,
            books: books,
        }
    }
}

fn read_file(
    file_name: String,
) -> Result<(HashMap<i64, Library>, HashMap<i64, Book>, i64), std::io::Error> {
    if let Ok(mut lines) = read_lines(file_name) {
        let mut books: HashMap<i64, Book> = HashMap::new();
        let mut libraries: HashMap<i64, Library> = HashMap::new();
        let mut max_days: i64 = 0;

        if let Ok(first_line) = lines.next().unwrap() {
            let mut first_line = first_line.split_whitespace();
            let _number_of_books: i64 = first_line.next().unwrap().parse().unwrap();
            let _number_of_libraries: i64 = first_line.next().unwrap().parse().unwrap();
            max_days = first_line.next().unwrap().parse().unwrap();
        }

        if let Ok(book_score_line) = lines.next().unwrap() {
            let book_score_line = book_score_line.split_whitespace();

            for (id, book_score) in book_score_line.enumerate() {
                let id: i64 = id.try_into().unwrap();
                let book = Book::new(id, book_score.parse().unwrap());
                books.insert(id, book);
            }
        }
        let mut i: i64 = 0;
        while let Some(first) = lines.next() {
            if let Some(second) = lines.next() {
                //first contains library meta data [number_of_books signup_time books_per_day]
                let first = first.unwrap();
                let mut first = first.split_whitespace();
                let _number_of_books: i64 = first.next().unwrap().parse().unwrap();
                let signup_time: i64 = first.next().unwrap().parse().unwrap();
                let books_per_day: i64 = first.next().unwrap().parse().unwrap();

                let mut lib = Library::new(i, signup_time, books_per_day);

                //second line contains the book ids
                let second = second.unwrap();
                let second = second.split_whitespace();

                for book_in_here in second {
                    let book_in_here: i64 = book_in_here.parse().unwrap();
                    let book = books.get(&book_in_here).unwrap();
                    lib.books.push(book.clone());
                    match books.get_mut(&book_in_here) {
                        Some(b) => b.libraries.push(i),
                        None => println!("Should not happen"),
                    }
                }
                lib.books.sort_by(|a, b| b.score.cmp(&a.score));

                libraries.insert(i, lib);
                i = i + 1;
            }
        }

        //println!("Books: {:#?}", books);
        //println!("Libraries: {:#?}", libraries);

        Ok((libraries, books, max_days))
    } else {
        Err(Error::new(ErrorKind::Other, "Parsing failed"))
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
