use std::fs::File;
use std::io::{ self, BufReader, Write, BufRead };
use std::net::TcpStream;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct Config {
    database_ip: String,
    database_port: u16,
}

pub fn establish_connection() -> io::Result<TcpStream> {
    // Чтение конфигурации из config.json
    let config: Config = {
        let file = File::open("/home/kali/Desktop/VSCode_files/STUDY/prak_3/config.json")?;
        serde_json::from_reader(file)?
    };

    // Установка TCP-соединения
    let address = format!("{}:{}", config.database_ip, config.database_port);
    TcpStream::connect(address)
}

pub async fn execute_query(query: String) -> io::Result<String> {
    let mut stream = establish_connection().unwrap();

    // Устанавливаем тайм-аут для операций чтения и записи
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    // Отправка запроса
    stream.write_all(query.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    println!("Sent query: {:?}", query);

    // Чтение ответа
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    let mut first_line = String::new();

    // Читаем первую строку
    reader.read_line(&mut first_line)?;
    println!("Received first line: {:?}", first_line.trim());

    // Читаем оставшуюся часть ответа построчно
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 || line.trim() == "END" {
            break; // Конец ответа
        }
        response.push_str(&line);
    }
    println!("Received response: {:?}", response);

    if first_line.trim() == "SUCCES" {
        Ok(response)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, response))
    }
}
