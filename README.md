# [SQL Rustico](https://taller-1-fiuba-rust.github.io/proyecto/24C2/ejercicio_individual.html)<br />[Taller de Programación I](https://taller-1-fiuba-rust.github.io/inicio.html)<br />Cátedra Deymonnaz - 2C2024  

## Resumen
El trabajo consiste en basicamente realizar un motor sencillo de bases de datos, basados en SQL.
Debe soportar consultas del tipo SELECT, INSERT, UPDATE, DELETE con posibilidad de tener
consultas con condiciones (= , !=, operadores de mayor y menor), y permitir el ordenamiento si es
que se desea.
> No se da soporte a operadores tipo LENGHT o JOINs

## Ejecución:
El formato de ejecución del trabajo practico está dado por 
```
cargo run -- <ruta_a_directorio_con_tablas> <CONSULTA>
```
Donde:
* Las consultas tipo SELECT, serán mostradas por la terminal, y su contenido puede ser redireccionable.
    Ejemplo: 

    ```
    cargo run -- ./tables "SELECT * FROM clientes WHERE Edad >= 45;"
    # o bien
    cargo run -- ./tables "SELECT * FROM clientes WHERE Edad>=45;"
    ```

    Mostrará por terminal

    ```
    Nombre,Apellido,Edad,Correo electronico,Profesion
    Carlos,Sánchez,45,csanchez@gmail.com,ingeniero
    ```

    Si se desea redireccionar su contenido:

    ```
    cargo run -- ./tables "SELECT * FROM clientes WHERE Edad >= 45;" > result_select.csv
    ```

    Se generará un archivo result_select del tipo CSV con los resultados de la búsqueda.

* Las consultas UPDATE, INSERT, DELETE son ejecutadas sobre el archivo que se está trabajando
    Ejemplos
    ```
    ## esta consulta generará un update sobre todos los clientes con Edad = 31
    cargo run -- ./tables "UPDATE clientes SET Nombre = 'Github', Edad = 45 WHERE Edad=31;"

    ## esta consulta eliminará por completo la database 
    cargo run -- ./tables "DELETE FROM clientes;" 

    ## Esta consulta insertará (Juan, 20), y el resto de los espacios serán NULL (blank)
    cargo run -- ./tables "INSERT INTO clientes (Nombre, Edad) VALUES ('Juan', 20);" #Se inserta un valor en la base de datos

    ## Esta consulta insertara (Juan, 20) y (Marcelo, 20) y para ambas el resto de los espacios seran NULL
    cargo run -- ./tables "INSERT INTO clientes (Nombre, Edad) VALUES ('Juan', 20), ('Marcelo', 20);

    ## Esta consulta insertara por completo (55, Lucas, nodox, 80, test@gmail.com, informatico)
    cargo run -- ./tables "INSERT into clientes VALUES (55, Lucas, nodox, 80, test@gmail.com, informatico)
    ```

    > Aclaración: ***Por defecto, hay 2 archivos los cuales puede usar adentro de ./tables, usá el que mas te gusta!***

## Pruebas

> [!NOTE]
> No tocar el archivo database.csv que está en ./tests/test_tables dado que los tests unitarios
> lo utilizan para las pruebas.

Para correr **TODAS** las pruebas, se deberá correr el comando
`
cargo test
`

Para ***solo*** correr las pruebas de integración, se deberá correr el comando
`
cargo test --test '*'
`

## Correcciones realizadas

- [x] Cambio de lectura de archivos a ruta de carpeta
- [x] Cada estructura tiene su propia carpeta creada.
- [x] Limpieza de comentarios inecesarios
- [x] Refactor [run](https://github.com/lucas794/20242c_tpindividual_taller/blob/ee3c77fc37aa7f4291fd7df8e4af1758ee08b7e7/src/main.rs#L30) en ./src para hacer bloques de código mas cortos
- [x] Soporte de paréntesis en condicionales
- [x] Error con condiciones pegadas, programa quedaba colgado, ahora se lanza un error correspondiente.
- [x] Caso particular con operacion AND a veces fallaba.
- [x] Agregado por mi parte (no es parte de la corrección): Se agregó un struct Sorter.
- [x] SELECT: Las consultas ahora se devuelven en el orden ingresado de las columnas.
- [x] SELECT: Las consultas ahora permiten condicionales que no estén ingresadas en la query (Ejemplo SELECT Edad FROM table WHERE Nombre = 'Luis')
- [x] INSERT: Solucionado problema de ingreso de un valor nuevo
- [x] DELETE: Solucionado problema de la eliminación de un elemento de la tabla.
- [x] TODAS LAS CONSULTAS: Fix en columnas que tenian nombre espaciado.
- [x] Test agregados para estas nuevas features mencionadas previamente
- [x] Separación de integration_test a clases para cada tipo de consulta.
- [x] Rework completo sobre los tests, utilizando ahora un mock para evitar lecturas de archivo directas y evitar problemas a la hora de correr tests
- [x] Fix condiciones sin terminar dejaban el programa colgado
- [x] ORDER BY por columnas que no estaban en la query fallaba (agregados tests para esto)
- [x] Soporte para condiciones pegadas (Generé varios test para esto, y mixeando condiciones no pegadas y pegadas)
- [x] Fix usos de unwrap en algunos casos bordes y uso de std::process::exit
- [x] Arreglado error en returns de Table y FolderTables para matchear la funcion de retorno de creacion en vez de crear un nuevo error
- [x] IMPORTANTE: fix con comparaciones con espacios en los strings (agregadas pruebas tambien)
- [x] IMPORTANTE: Agregado soporte de operaciones con columnas como condiciones (SELECT * FROM clientes WHERE Id > Edad por ejemplo, agregado test también)