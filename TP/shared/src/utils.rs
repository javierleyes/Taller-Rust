use std::io::Read;

/// Reads utf8 String from a stream
/// Returns a result with a new ThreadPool
/// # Arguments
///
/// * `stream` - a readable object
///
/// # Examples
///
/// ```no_run
/// # use shared::utils::read_utf8_string;
/// # use std::io;
/// # use std::io::BufReader;
/// # let pointer = &Vec::new()[..];
/// # let mut my_stream = BufReader::new(pointer);
/// let content = read_utf8_string(&mut my_stream)?;
/// println!("{}", content);
/// # Ok::<(), io::Error>(())
/// ```
pub fn read_utf8_string(stream: &mut dyn Read) -> std::io::Result<String> {
    // Read 16 bit length first
    let mut num_buffer = [0u8; 2];
    stream.read_exact(&mut num_buffer)?;
    let size = u16::from_be_bytes(num_buffer);

    // Read content
    let mut buffer = vec![0; size as usize];
    stream.read_exact(&mut buffer)?;
    let result_str = std::str::from_utf8(&buffer).expect("Error al leer el campo");
    Ok(result_str.to_owned())
}
