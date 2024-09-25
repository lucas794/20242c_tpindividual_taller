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
    cargo run -- ./tables "UPDATE clientes SET Nombre = 'Github', Edad = 45 WHERE Edad = 31;" # Se ejecuta un update
    cargo run -- ./tables "DELETE FROM clientes;" # Se ejecuta un delete, en este caso particular se borra todo el archivo
    cargo run -- ./tables "INSERT INTO clientes (Nombre, Edad) VALUES ('Juan', 20);" #Se inserta un valor en la base de datos
    ```

    > Aclaración: ***si se va a ejecutar algún comando, utilizar la carpeta ./tables como referencia***

## Pruebas
> [!NOTE]
> No tocar el archivo database.csv que está en ./tests/test_tables dado que los test se realizan
> haciendo una copia exacta de ese archivo y corriendo las pruebas 

Para correr **TODAS** las pruebas, se deberá correr el comando
`
cargo test
`

Para ***solo*** correr las pruebas de integración, se deberá correr el comando
`
cargo test --test '*'
`

> [!WARNING]
> En algunos casos, por alguna razón, los test NO corren por lectura/escritura de archivos simultaneamente. Correr cargo test nuevamente en esos casos, todas las entregas se dan con cargo test verificado.

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
- [x] Correccion README.exe
