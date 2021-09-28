use std::io;

fn main() {
    println!("");
    println!("Bienvenido al ahorcado de FIUBA!");
    println!("");

    let mut attempts = 5;

    // load word
    let mut word_to_guess = Vec::new();
    word_to_guess.push('p');
    word_to_guess.push('e');
    word_to_guess.push('r');
    word_to_guess.push('r');
    word_to_guess.push('o');

    let mut word_to_show = Vec::new();
    
    for _ in word_to_guess.iter() {
        word_to_show.push('_');
    }
    
    loop {
        put_char(&mut attempts, &word_to_guess, &mut word_to_show);
        show_word(attempts, &word_to_show);

        let is_end = check_end_game(attempts, &word_to_guess, &word_to_show);
        if is_end == true {
            break;
        }
    }
}

fn put_char(attempts: &mut u8, word_to_guess: &Vec<char>, word_to_show: &mut Vec<char>)
{
    println!("Ingresa una letra: ");
    let mut input = String::new();
    let char_vec: Vec<char>;

    io::stdin()
        .read_line(&mut input)
        .expect("Error leyendo la linea.");

    // toma un string, lo separa en chars y collect arma un vector con esos chars
    char_vec = input.chars().collect();

    let mut find = false;

    for i in 0..word_to_guess.len()
    {
        if word_to_guess[i] == char_vec[0] {
            word_to_show[i] = char_vec[0];
            find = true;
        }
    }

    if !find {
        *attempts = *attempts - 1;
    }
}

fn show_word(attempts: u8, word_to_show: &Vec<char>) {
    print!("La palabra hasta el momento es:");
    for element in word_to_show {
        print!(" {} ", element);
    }
    println!("");

    print!("Adivinaste las siguientes letras:");
    for element in word_to_show {
        if *element != '_' {
            print!(" {} ", element.to_string());    
        }
    }

    println!();
    println!("Te quedan {} intentos.", attempts);
}

fn check_end_game(attempts: u8, word_to_guess: &Vec<char>, word_to_show: &Vec<char>) -> bool {
    if attempts == 0 {
        println!("Game over.");
        return true;
    }

    let word_to_guess_string: String = word_to_guess.into_iter().collect();
    let word_to_show_string: String = word_to_show.into_iter().collect();

    if word_to_guess_string == word_to_show_string {
        println!("You win.");
        return true;
    }

    return false;
}
