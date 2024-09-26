use std::fs::{File, OpenOptions};
use std::io::Write; // Add this line to import the Write trait

const CSV_DATA: &str = r#"Nombre,Apellido,Edad,Correo electronico,Profesion
Juan,Perez,32,jperez@gmail.com,medico
Maria,Gomez,28,mgomez@gmail.com,abogado
Carlos,Sánchez,45,csanchez@gmail.com,ingeniero
Ana,Ruiz,36,aruiz@gmail.com,arquitecta
Luis,Martínez,29,lmartinez@gmail.com,profesor
Laura,Domínguez,41,ldominguez@gmail.com,enfermera
Pedro,Fernández,33,pfernandez@gmail.com,diseñador
Lucía,Ramos,26,lramos@gmail.com,psicóloga
Diego,Navarro,39,dnavarro@gmail.com,empresario
Paula,Hernández,31,phernandez@gmail.com,publicista
"#;

pub fn setup(file_full_path: &str) {
    let _ = File::create(&file_full_path).unwrap();

    for line in CSV_DATA.lines() {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&file_full_path)
            .unwrap();
        writeln!(file, "{}", line).unwrap();
    }
}
