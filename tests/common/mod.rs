const CSV_DATA: &str = "Id,Nombre,Apellido,Edad,Correo electronico,Profesion\n\
1,Juan,Perez,32,jperez@gmail.com,medico\n\
2,Maria,Gomez,28,mgomez@gmail.com,abogado\n\
3,Carlos,Sánchez,45,csanchez@gmail.com,ingeniero\n\
4,Ana,Ruiz,36,aruiz@gmail.com,arquitecta\n\
5,Luis,Martínez,29,lmartinez@gmail.com,profesor\n\
6,Laura,Domínguez,41,ldominguez@gmail.com,enfermera\n\
7,Pedro,Fernández,33,pfernandez@gmail.com,diseñador\n\
8,Lucía,Ramos,26,lramos@gmail.com,psicóloga\n\
9,Diego,Navarro,39,dnavarro@gmail.com,empresario\n\
10,Paula,Hernández,31,phernandez@gmail.com,publicista\n\
";

pub fn csv_data_as_bytes() -> &'static [u8] {
    CSV_DATA.as_bytes()
}

pub fn return_iterator_from_mock_csv_file() -> impl Iterator<Item = String> {
    CSV_DATA
        .as_bytes()
        .split(|byte| *byte == b'\n')
        .map(|line| String::from_utf8(line.to_vec()).unwrap())
}
